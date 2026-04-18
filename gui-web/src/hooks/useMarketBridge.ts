import { useEffect } from 'react'
import { listen } from '@tauri-apps/api/event'
import { useAppStore } from '../store/app'
import { api } from '../tauri'
import { loadPersisted, saveWatchlist } from '../persist'
import {
  CONN_STATE_EVENT,
  INDICES_PREFIX,
  MARKET_BATCH_EVENT,
  TAIEX_SYMBOL,
} from '../types/events'
import type { ConnectionState, MarketEvent } from '../types/market'
import { DEFAULT_TIMEFRAME } from '../types/timeframe'

const SAVE_DEBOUNCE_MS = 200
let bootstrapped = false

/**
 * Mounts at App root. Wires Tauri events → Zustand, hydrates persisted
 * settings, drives the WebSocket connect lifecycle, and persists watchlist
 * changes. Idempotent in dev mode (StrictMode double-invoke).
 */
export function useMarketBridge() {
  useEffect(() => {
    let unlistenBatch: (() => void) | undefined
    let unlistenConn: (() => void) | undefined
    let unsubWatchlist: (() => void) | undefined
    let cancelled = false
    let saveTimer: ReturnType<typeof setTimeout> | undefined
    let pendingSave: string[] | undefined

    const flushSave = () => {
      if (saveTimer) {
        clearTimeout(saveTimer)
        saveTimer = undefined
      }
      if (pendingSave) {
        void saveWatchlist(pendingSave)
        pendingSave = undefined
      }
    }

    void (async () => {
      const [unlistenA, unlistenB] = await Promise.all([
        listen<MarketEvent[]>(MARKET_BATCH_EVENT, (ev) => {
          useAppStore.getState().applyEvents(ev.payload)
        }),
        listen<ConnectionState>(CONN_STATE_EVENT, (ev) => {
          useAppStore.getState().applyConn(ev.payload)
        }),
      ])
      if (cancelled) {
        unlistenA()
        unlistenB()
        return
      }
      unlistenBatch = unlistenA
      unlistenConn = unlistenB

      // One-shot bootstrap across StrictMode double-invoke.
      if (!bootstrapped) {
        bootstrapped = true
        const persisted = await loadPersisted()
        if (cancelled) return
        useAppStore.getState().hydrate(persisted)

        if (persisted.apiKey) {
          await connectAndResubscribe(persisted.apiKey, persisted.watchlist)
        }
      }

      // Persist watchlist on every change (post-hydrate), debounced so
      // rapid drag-reorder collapses to a single disk write.
      let lastWatchlist = useAppStore.getState().watchlist
      unsubWatchlist = useAppStore.subscribe(
        (s) => s.watchlist,
        (watchlist) => {
          if (watchlist === lastWatchlist) return
          lastWatchlist = watchlist
          pendingSave = watchlist
          if (saveTimer) clearTimeout(saveTimer)
          saveTimer = setTimeout(flushSave, SAVE_DEBOUNCE_MS)
        },
      )
    })()

    return () => {
      cancelled = true
      unlistenBatch?.()
      unlistenConn?.()
      unsubWatchlist?.()
      flushSave()
    }
  }, [])
}

/**
 * Connects, subscribes TAIEX + every watchlist symbol, and seeds REST data
 * for each. Used both on first-launch (after API-key submit) and on hydrate
 * (when restoring a persisted session).
 */
export async function connectAndResubscribe(apiKey: string, watchlist: string[]) {
  try {
    await api.connect(apiKey)
  } catch (e) {
    console.error('connect failed', e)
    return
  }

  // Always have TAIEX in the status bar; it lives outside the watchlist.
  api.subscribe(TAIEX_SYMBOL).catch((e) => console.warn('TAIEX subscribe failed', e))

  for (const symbol of watchlist) {
    api.subscribe(symbol).catch((e) => console.warn('subscribe failed', symbol, e))
    void seedSymbol(symbol)
  }
}

export async function seedSymbol(symbol: string) {
  if (symbol.startsWith(INDICES_PREFIX)) return
  try {
    const [ticker, trades, quote] = await Promise.all([
      api.fetchTicker(symbol),
      api.fetchTrades(symbol),
      api.fetchQuote(symbol),
    ])
    const store = useAppStore.getState()
    store.applyTicker(symbol, ticker)
    store.applyTradeHistory(symbol, trades)
    store.applyQuote(symbol, quote)
  } catch (e) {
    console.error('seed failed', symbol, e)
  }

  // Fire-and-forget: prime candles in background so switching to this symbol
  // shows a chart without waiting. Kept out of Promise.all above so ticker/
  // trades/quote don't block on a bulky candles payload.
  api
    .fetchCandles(symbol, DEFAULT_TIMEFRAME)
    .then((candles) =>
      useAppStore.getState().setCandles(symbol, candles, DEFAULT_TIMEFRAME),
    )
    .catch((e) => console.error('seed candles failed', symbol, e))
}
