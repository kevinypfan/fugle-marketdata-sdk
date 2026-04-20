import { useEffect } from 'react'
import { listen } from '@tauri-apps/api/event'
import { useAppStore } from '../store/app'
import { api } from '../tauri'
import { loadPersisted, saveWatchlist } from '../persist'
import {
  CONN_STATE_EVENT,
  INDEX_SYMBOLS,
  INDICES_PREFIX,
  MARKET_BATCH_EVENT,
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
          await connectAndResubscribe(
            persisted.apiKey,
            persisted.restBaseUrl,
            persisted.wsUrl,
            persisted.watchlist,
          )
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
 * Connects, subscribes the index bar symbols + every watchlist symbol, and
 * seeds REST data for each. Used both on first-launch (after API-key submit)
 * and on hydrate (when restoring a persisted session).
 */
export async function connectAndResubscribe(
  apiKey: string,
  restBaseUrl: string | null,
  wsUrl: string | null,
  watchlist: string[],
) {
  try {
    await api.connect(apiKey, wsUrl)
  } catch (e) {
    console.error('connect failed', e)
    return
  }

  // Indices live outside the watchlist — always subscribe + seed previousClose.
  for (const symbol of INDEX_SYMBOLS) {
    api.subscribe(symbol).catch((e) => console.warn('index subscribe failed', symbol, e))
    void seedIndex(symbol, restBaseUrl)
  }

  for (const symbol of watchlist) {
    api.subscribe(symbol).catch((e) => console.warn('subscribe failed', symbol, e))
    void seedSymbol(symbol, restBaseUrl)
  }
}

/**
 * Pre-populate the IndicesBar sparkline and change-% reference from Fugle's
 * `/stock/intraday/candles/{IX}` endpoint (which returns today's 1-min bars
 * for indices). First bar's `open` ≈ today's opening price — used as the
 * "change" reference. Fugle doesn't expose true yesterday-close for indices
 * to this SDK, so the shown change is "change since today's open" (off from
 * the exchange's official change by the overnight gap, typically <1%).
 *
 * Non-fatal: on failure we just rely on WS ticks (sparkline grows live,
 * change-% falls back to session-start reference via `history[0]`).
 */
async function seedIndex(symbol: string, restBaseUrl: string | null) {
  try {
    const candles = await api.fetchCandles(symbol, '1', restBaseUrl)
    if (candles.length === 0) return
    const closes = candles.map((c) => c.close)
    useAppStore.getState().seedIndexHistory(symbol, closes, candles[0].open)
  } catch (e) {
    console.warn('seed index failed', symbol, e)
  }
}

/** Seed tape size. Explicit so `hasMoreTape` can detect a full page — relying
 *  on the server's implicit default would make the "more available?" signal
 *  ambiguous (default varies by endpoint). */
const TRADE_SEED_LIMIT = 200

export async function seedSymbol(symbol: string, restBaseUrl: string | null) {
  if (symbol.startsWith(INDICES_PREFIX)) return
  try {
    const [ticker, trades, quote] = await Promise.all([
      api.fetchTicker(symbol, restBaseUrl),
      api.fetchTrades(symbol, restBaseUrl, { limit: TRADE_SEED_LIMIT }),
      api.fetchQuote(symbol, restBaseUrl),
    ])
    const store = useAppStore.getState()
    store.applyTicker(symbol, ticker)
    store.applyTradeHistory(symbol, trades, TRADE_SEED_LIMIT)
    store.applyQuote(symbol, quote)
  } catch (e) {
    console.error('seed failed', symbol, e)
  }

  // Fire-and-forget: prime candles in background so switching to this symbol
  // shows a chart without waiting. Kept out of Promise.all above so ticker/
  // trades/quote don't block on a bulky candles payload.
  api
    .fetchCandles(symbol, DEFAULT_TIMEFRAME, restBaseUrl)
    .then((candles) =>
      useAppStore.getState().setCandles(symbol, candles, DEFAULT_TIMEFRAME),
    )
    .catch((e) => console.error('seed candles failed', symbol, e))
}
