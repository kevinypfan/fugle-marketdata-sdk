import { useEffect } from 'react'
import { listen } from '@tauri-apps/api/event'
import { useAppStore } from '../store/app'
import { api } from '../tauri'
import { loadPersisted, saveWatchlist } from '../persist'
import type { ConnectionState, MarketEvent } from '../types/market'

const TAIEX_SYMBOL = 'IX0001'
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

    void (async () => {
      const [unlistenA, unlistenB] = await Promise.all([
        listen<MarketEvent[]>('market-batch', (ev) => {
          useAppStore.getState().applyEvents(ev.payload)
        }),
        listen<ConnectionState>('connection-state', (ev) => {
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

      // Persist watchlist on every change (post-hydrate).
      let lastWatchlist = useAppStore.getState().watchlist
      unsubWatchlist = useAppStore.subscribe(
        (s) => s.watchlist,
        (watchlist) => {
          if (watchlist === lastWatchlist) return
          lastWatchlist = watchlist
          void saveWatchlist(watchlist)
        },
      )
    })()

    return () => {
      cancelled = true
      unlistenBatch?.()
      unlistenConn?.()
      unsubWatchlist?.()
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
  if (symbol.startsWith('IX')) return
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
}
