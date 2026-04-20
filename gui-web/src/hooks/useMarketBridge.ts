import { useEffect } from 'react'
import { listen } from '@tauri-apps/api/event'
import { useAppStore } from '../store/app'
import { api } from '../tauri'
import { loadPersisted, saveWatchlists } from '../persist'
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
    let unsubStock: (() => void) | undefined
    let unsubFutopt: (() => void) | undefined
    let cancelled = false
    let saveTimer: ReturnType<typeof setTimeout> | undefined
    let pendingStock: string[] | undefined
    let pendingFutopt: string[] | undefined

    const flushSave = () => {
      if (saveTimer) {
        clearTimeout(saveTimer)
        saveTimer = undefined
      }
      if (pendingStock !== undefined || pendingFutopt !== undefined) {
        const s = useAppStore.getState()
        void saveWatchlists(
          pendingStock ?? s.stockWatchlist,
          pendingFutopt ?? s.futoptWatchlist,
        )
        pendingStock = undefined
        pendingFutopt = undefined
      }
    }

    const scheduleSave = () => {
      if (saveTimer) clearTimeout(saveTimer)
      saveTimer = setTimeout(flushSave, SAVE_DEBOUNCE_MS)
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
            persisted.stockWatchlist,
            persisted.futoptWatchlist,
          )
        }
      }

      // Persist watchlists on every change (post-hydrate), debounced so
      // rapid drag-reorder collapses to a single disk write.
      let lastStock = useAppStore.getState().stockWatchlist
      unsubStock = useAppStore.subscribe(
        (s) => s.stockWatchlist,
        (w) => {
          if (w === lastStock) return
          lastStock = w
          pendingStock = w
          scheduleSave()
        },
      )
      let lastFutopt = useAppStore.getState().futoptWatchlist
      unsubFutopt = useAppStore.subscribe(
        (s) => s.futoptWatchlist,
        (w) => {
          if (w === lastFutopt) return
          lastFutopt = w
          pendingFutopt = w
          scheduleSave()
        },
      )
    })()

    return () => {
      cancelled = true
      unlistenBatch?.()
      unlistenConn?.()
      unsubStock?.()
      unsubFutopt?.()
      flushSave()
    }
  }, [])
}

/**
 * Connects both the stock and (if there are any futopt symbols) the futopt
 * WebSocket, subscribes index symbols + every watchlist symbol, and seeds REST
 * data for each. Used on first launch (post-API-key submit) and on hydrate
 * (restoring a persisted session). FutOpt WS is only connected when needed —
 * a stock-only user never hits the futopt endpoint.
 */
export async function connectAndResubscribe(
  apiKey: string,
  restBaseUrl: string | null,
  wsUrl: string | null,
  stockWatchlist: string[],
  futoptWatchlist: string[],
) {
  // Stock and futopt WS are independent — run connects in parallel to save
  // one handshake RTT on startup. Failures are isolated: if futopt rejects
  // (e.g. account lacks perm), stock still comes up.
  const needFutopt = futoptWatchlist.length > 0
  const results = await Promise.allSettled([
    api.connect(apiKey, wsUrl),
    needFutopt ? api.connectFutopt(apiKey, deriveFutoptWsUrl(wsUrl)) : Promise.resolve(),
  ])
  if (results[0].status === 'rejected') {
    console.error('connect (stock) failed', results[0].reason)
    return
  }
  if (needFutopt && results[1].status === 'rejected') {
    console.error('connect (futopt) failed', results[1].reason)
    // Fall through: stock is up, futopt subscribes below will throw
    // NotConnected individually — logged but non-fatal for stock.
  }

  // Indices live outside the watchlist — always subscribe + seed previousClose.
  for (const symbol of INDEX_SYMBOLS) {
    api.subscribe(symbol).catch((e) => console.warn('index subscribe failed', symbol, e))
    void seedIndex(symbol, restBaseUrl)
  }

  for (const symbol of stockWatchlist) {
    api.subscribe(symbol).catch((e) => console.warn('subscribe failed', symbol, e))
    void seedSymbol(symbol, restBaseUrl)
  }

  if (needFutopt) {
    const afterHours = useAppStore.getState().futoptSession === 'afterhours'
    for (const symbol of futoptWatchlist) {
      // Seed first so REST ticker populates the store's alias map before any
      // WS event with canonical symbol arrives. Subscribe with user-input
      // (aliases like TXF1! are accepted; server echoes canonical in data
      // events — alias map redirects events back to the user-input key).
      await seedFutoptSymbol(symbol, restBaseUrl, afterHours)
      api
        .subscribeFutopt(symbol, afterHours)
        .catch((e) => console.warn('futopt subscribe failed', symbol, e))
    }
  }
}

/** Switch the FutOpt trading session end-to-end:
 *  1. Unsubscribe every futopt symbol on the OLD session
 *  2. Clear stale SymbolState (old session's tape/book/agg linger otherwise)
 *  3. Set the new session in the store
 *  4. Re-seed each symbol with the new REST session, then resubscribe on WS
 *
 *  Uses `useAppStore.getState()` directly so the caller doesn't need to
 *  thread the store through props. Idempotent if called with the same
 *  session — still tears down and rebuilds, which is fine for a user-driven
 *  action. Call this from a UI handler, not a render. */
export async function applyFutoptSession(
  next: 'regular' | 'afterhours',
  apiKey: string | null,
  restBaseUrl: string | null,
) {
  const store = useAppStore.getState()
  const current = store.futoptSession
  const watchlist = store.futoptWatchlist
  const futoptAlive = store.conn.futopt?.state === 'connected'
  if (!apiKey || watchlist.length === 0 || !futoptAlive) {
    // No live connection to reconfigure — flip the flag so future adds /
    // reconnects use the new session. unsubscribeFutopt would throw
    // NotConnected here since the futopt client isn't up yet.
    store.setFutoptSession(next)
    return
  }

  const oldAfterHours = current === 'afterhours'
  for (const symbol of watchlist) {
    try {
      await api.unsubscribeFutopt(symbol, oldAfterHours)
    } catch (e) {
      console.warn('unsubscribe before session switch failed', symbol, e)
    }
  }

  store.clearFutoptSymbolData()
  store.setFutoptSession(next)

  const nextAfterHours = next === 'afterhours'
  for (const symbol of watchlist) {
    try {
      await seedFutoptSymbol(symbol, restBaseUrl, nextAfterHours)
      await api.subscribeFutopt(symbol, nextAfterHours)
    } catch (e) {
      console.error('session-switch resubscribe failed', symbol, e)
    }
  }
}

/** Derive the futopt WS URL from a user-configured stock WS URL by path swap.
 *  Dev vs prod differ only in host, not in the `/stock/streaming` ↔
 *  `/futopt/streaming` suffix, so a single override (stock wsUrl) is enough
 *  to route futopt to the same environment. Returns null when stock wsUrl
 *  isn't set (SDK falls back to its prod default). */
function deriveFutoptWsUrl(stockWsUrl: string | null): string | null {
  if (!stockWsUrl) return null
  if (stockWsUrl.includes('/stock/streaming')) {
    return stockWsUrl.replace('/stock/streaming', '/futopt/streaming')
  }
  // Unrecognized shape — bail to SDK default rather than guess. Warn so a
  // dev-env session doesn't silently hit the prod futopt endpoint with a
  // dev API key (the failure mode that triggered this whole fix).
  console.warn(
    `deriveFutoptWsUrl: stock wsUrl "${stockWsUrl}" has no "/stock/streaming" segment; futopt will fall back to the SDK default (prod). If you are on dev, set a stock wsUrl containing "/stock/streaming" so futopt can be derived.`,
  )
  return null
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

/** FutOpt symbol seeder — mirror of `seedSymbol` but routed to the futopt
 *  REST endpoints. Takes the current `afterHours` flag so REST pulls the
 *  matching session (server serves disjoint 日盤 / 夜盤 streams; wrong flag
 *  returns the other session's data silently). Candle fetch fire-and-forget.
 *
 *  Subscribe uses the user-input symbol directly (e.g. `TXF1!`). Server
 *  echoes canonical (e.g. `TXFE6`) in data events; `symbolAliases` redirects. */
export async function seedFutoptSymbol(
  symbol: string,
  restBaseUrl: string | null,
  afterHours: boolean,
) {
  try {
    const [ticker, trades, quote] = await Promise.all([
      api.fetchFutoptTicker(symbol, restBaseUrl),
      api.fetchFutoptTrades(symbol, restBaseUrl, {
        limit: TRADE_SEED_LIMIT,
        afterHours,
      }),
      api.fetchFutoptQuote(symbol, restBaseUrl, afterHours),
    ])
    const store = useAppStore.getState()
    store.applyFutoptTicker(symbol, ticker)
    store.applyTradeHistory(symbol, trades, TRADE_SEED_LIMIT)
    store.applyFutoptQuote(symbol, quote)
  } catch (e) {
    console.error('seed futopt failed', symbol, e)
  }

  api
    .fetchFutoptCandles(symbol, DEFAULT_TIMEFRAME, restBaseUrl, afterHours)
    .then((candles) =>
      useAppStore.getState().setCandles(symbol, candles, DEFAULT_TIMEFRAME),
    )
    .catch((e) => console.error('seed futopt candles failed', symbol, e))
}

/** Ensure the futopt WS is connected before subscribing the first futopt
 *  symbol. Idempotent: second call is a no-op in the bridge. Call this
 *  before `api.subscribeFutopt` in the add-to-watchlist flow. The futopt WS
 *  URL is derived from the stock wsUrl so dev/prod stay aligned without a
 *  second setting to configure. */
export async function ensureFutoptConnected(
  apiKey: string | null,
  stockWsUrl: string | null,
) {
  if (!apiKey) return
  try {
    await api.connectFutopt(apiKey, deriveFutoptWsUrl(stockWsUrl))
  } catch (e) {
    console.error('connect (futopt) failed', e)
  }
}
