import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { immer } from 'zustand/middleware/immer'
import type {
  AggregatesData,
  BooksData,
  CandleDto,
  ConnectionState,
  MarketEvent,
  Quote,
  StreamTrade,
  Ticker,
  Trade,
} from '../types/market'
import type { Timeframe } from '../types/timeframe'

const TAPE_LIMIT = 200
/** Hard safety cap once the user has opted into load-more — prevents runaway
 *  growth while still respecting the user's explicit request for more history. */
const HARD_TAPE_CEILING = 5000
const INDEX_HISTORY_LIMIT = 120

/** WS trades carry no server serial; assign monotonic negatives so React keys
 *  stay unique without colliding with server-side positive serials. */
let syntheticSerialCounter = -1

export interface Tick {
  price: number
  size: number
  bid?: number
  ask?: number
  time: number
  /** -1 = sell hit bid, +1 = buy lifted ask, 0 = unknown */
  direction: -1 | 0 | 1
  /** Server serial for REST trades, synthetic negative for WS ticks. Always
   *  present so `key={t.serial}` has a stable unique value. */
  serial: number
}

export interface SymbolState {
  symbol: string
  ticker?: Ticker
  agg?: AggregatesData
  book?: BooksData
  tape: Tick[]
  /** How many trades landed in the initial REST seed (after TAPE_LIMIT slice). */
  tapeSeedCount?: number
  /** How many additional older trades loaded via `appendOlderTrades`. The
   *  next pagination offset is `(tapeSeedCount ?? 0) + (tapeExtraCount ?? 0)`.
   *  Kept independent of `tape.length` because WS prepends inflate that. */
  tapeExtraCount?: number
  /** False once the server returns a partial page. UI hides load-more. */
  hasMoreTape?: boolean
  /** Trial-matching snapshot — independent of `tape`. No live stream (WS
   *  has no per-trade isTrial flag), but supports the same load-more
   *  pagination as `tape` via the counters below. */
  trialTape?: Tick[]
  trialSeedCount?: number
  trialExtraCount?: number
  hasMoreTrialTape?: boolean
  /** newest-last (REST sort=asc). Holds whichever timeframe caller last set. */
  candles?: CandleDto[]
  /** Required for streaming: SDK only pushes 1-min ticks, so CandleTick
   *  only applies when this is '1'. */
  candleTimeframe?: Timeframe
}

export interface AppStore {
  // ── data
  apiKey: string | null
  restBaseUrl: string | null
  wsUrl: string | null
  watchlist: string[]
  selected: string | null
  symbols: Record<string, SymbolState>
  conn: ConnectionState | null
  /** Live index values, keyed by symbol (e.g. 'IX0001', 'IX0043').
   *  `history` is a rolling buffer of recent values for the sparkline and
   *  doubles as a session-reference fallback when `previousClose` is not
   *  seeded (Fugle REST's `/stock/historical/candles` 404s for IX symbols). */
  indices: Record<
    string,
    { value: number; previousClose?: number; history: number[] }
  >

  // ── index seeding
  seedIndexHistory: (
    symbol: string,
    closes: number[],
    previousClose: number,
  ) => void

  // ── lifecycle
  hydrate: (snapshot: {
    apiKey?: string | null
    restBaseUrl?: string | null
    wsUrl?: string | null
    watchlist?: string[]
  }) => void
  setApiKey: (key: string) => void
  setEndpoints: (restBaseUrl: string, wsUrl: string) => void

  // ── watchlist
  setSelected: (symbol: string) => void
  addToWatchlist: (symbol: string) => boolean
  removeFromWatchlist: (symbol: string) => void
  moveToTop: (symbol: string) => void
  moveToBottom: (symbol: string) => void

  // ── ingestion
  applyConn: (s: ConnectionState) => void
  applyEvents: (batch: MarketEvent[]) => void
  applyTicker: (symbol: string, ticker: Ticker) => void
  /** `fetchedLimit` is the `limit` passed to `fetchTrades` for the seed;
   *  drives `hasMoreTape` (full page → assume more, partial → exhausted). */
  applyTradeHistory: (symbol: string, trades: Trade[], fetchedLimit: number) => void
  /** Append older trades loaded via `fetchTrades(offset, limit)`. Dedups by
   *  serial, pushes to the tail, bumps `tapeExtraCount`, updates `hasMoreTape`. */
  appendOlderTrades: (symbol: string, older: Trade[], limit: number) => void
  /** Replace trial snapshot wholesale; resets trial pagination counters. */
  setTrialTape: (symbol: string, trades: Trade[], fetchedLimit: number) => void
  /** Append older trial trades; mirror of `appendOlderTrades` for `trialTape`. */
  appendOlderTrialTrades: (symbol: string, older: Trade[], limit: number) => void
  applyQuote: (symbol: string, quote: Quote) => void
  setCandles: (symbol: string, candles: CandleDto[], timeframe: Timeframe) => void
}

function ensureSymbol(symbols: Record<string, SymbolState>, symbol: string): SymbolState {
  let s = symbols[symbol]
  if (!s) {
    s = { symbol, tape: [] }
    symbols[symbol] = s
  }
  return s
}

function inferDirection(
  price: number,
  bid?: number,
  ask?: number,
  prevPrice?: number,
): -1 | 0 | 1 {
  if (bid !== undefined && ask !== undefined) {
    if (price >= ask) return 1
    if (price <= bid) return -1
  }
  if (prevPrice !== undefined) {
    if (price > prevPrice) return 1
    if (price < prevPrice) return -1
  }
  return 0
}

function tickFromStream(t: StreamTrade, time: number, prevPrice?: number): Tick {
  return {
    price: t.price,
    size: t.size,
    bid: t.bid,
    ask: t.ask,
    time,
    direction: inferDirection(t.price, t.bid, t.ask, prevPrice),
    serial: syntheticSerialCounter--,
  }
}

function tickFromRest(t: Trade): Tick {
  return {
    price: t.price,
    size: t.size,
    bid: t.bid,
    ask: t.ask,
    time: Math.floor(t.time / 1000),
    direction: inferDirection(t.price, t.bid, t.ask),
    serial: t.serial ?? syntheticSerialCounter--,
  }
}

export const useAppStore = create<AppStore>()(
  subscribeWithSelector(
    immer((set) => ({
      apiKey: null,
      restBaseUrl: null,
      wsUrl: null,
      watchlist: [],
      selected: null,
      symbols: {},
      conn: null,
      indices: {},

      hydrate: (snapshot) =>
        set((state) => {
          if (snapshot.apiKey !== undefined) state.apiKey = snapshot.apiKey
          if (snapshot.restBaseUrl !== undefined) state.restBaseUrl = snapshot.restBaseUrl
          if (snapshot.wsUrl !== undefined) state.wsUrl = snapshot.wsUrl
          if (snapshot.watchlist) {
            state.watchlist = snapshot.watchlist.slice()
            if (!state.selected && state.watchlist.length > 0) {
              state.selected = state.watchlist[0]
            }
          }
        }),

      setApiKey: (key) =>
        set((state) => {
          state.apiKey = key
        }),

      setEndpoints: (restBaseUrl, wsUrl) =>
        set((state) => {
          state.restBaseUrl = restBaseUrl
          state.wsUrl = wsUrl
        }),

      setSelected: (symbol) =>
        set((state) => {
          state.selected = symbol
        }),

      addToWatchlist: (symbol) => {
        let added = false
        set((state) => {
          if (!state.watchlist.includes(symbol)) {
            state.watchlist.push(symbol)
            ensureSymbol(state.symbols, symbol)
            added = true
            if (!state.selected) state.selected = symbol
          }
        })
        return added
      },

      removeFromWatchlist: (symbol) =>
        set((state) => {
          state.watchlist = state.watchlist.filter((s) => s !== symbol)
          delete state.symbols[symbol]
          if (state.selected === symbol) {
            state.selected = state.watchlist[0] ?? null
          }
        }),

      moveToTop: (symbol) =>
        set((state) => {
          const i = state.watchlist.indexOf(symbol)
          if (i > 0) {
            state.watchlist.splice(i, 1)
            state.watchlist.unshift(symbol)
          }
        }),

      moveToBottom: (symbol) =>
        set((state) => {
          const i = state.watchlist.indexOf(symbol)
          if (i >= 0 && i < state.watchlist.length - 1) {
            state.watchlist.splice(i, 1)
            state.watchlist.push(symbol)
          }
        }),

      applyConn: (s) =>
        set((state) => {
          if (
            state.conn?.state === s.state &&
            JSON.stringify(state.conn) === JSON.stringify(s)
          ) {
            return
          }
          state.conn = s
        }),

      applyEvents: (batch) =>
        set((state) => {
          for (const ev of batch) {
            // TEMP debug — remove after trade tape issue resolved
            if (ev.kind === 'TradeTick' || ev.kind === 'Indices') {
              // eslint-disable-next-line no-console
              console.log('[applyEvents]', ev.kind, ev)
            }
            switch (ev.kind) {
              case 'Indices': {
                if (ev.index !== undefined) {
                  const existing = state.indices[ev.symbol]
                  const history = existing?.history ?? []
                  // Dedup consecutive-same ticks: idle broadcasts repeat the
                  // same value, pushing would churn memo'd consumers for no gain.
                  const last = history[history.length - 1]
                  if (last !== ev.index) {
                    history.push(ev.index)
                    if (history.length > INDEX_HISTORY_LIMIT) {
                      history.splice(0, history.length - INDEX_HISTORY_LIMIT)
                    }
                  }
                  state.indices[ev.symbol] = {
                    value: ev.index,
                    previousClose: existing?.previousClose,
                    history,
                  }
                }
                break
              }
              case 'Aggregate': {
                const s = ensureSymbol(state.symbols, ev.symbol)
                s.agg = ev
                break
              }
              case 'BookSnap': {
                const s = ensureSymbol(state.symbols, ev.symbol)
                s.book = ev
                break
              }
              case 'TradeTick': {
                const s = ensureSymbol(state.symbols, ev.symbol)
                const time = ev.time ? Math.floor(ev.time / 1000) : Date.now()
                const newest: Tick[] = []
                let last = s.tape[0]?.price
                for (const t of ev.trades) {
                  newest.push(tickFromStream(t, time, last))
                  last = t.price
                }
                // ev.trades is oldest-first; tape[0] is newest, so reverse before prepending.
                newest.reverse()
                s.tape = newest.concat(s.tape)
                // Cap only when the user hasn't opted into load-more; otherwise
                // a WS reconnect replay would silently evict the loaded history
                // and invalidate `tapeExtraCount`/`hasMoreTape`.
                const cap = (s.tapeExtraCount ?? 0) > 0 ? HARD_TAPE_CEILING : TAPE_LIMIT
                if (s.tape.length > cap) s.tape = s.tape.slice(0, cap)
                break
              }
              case 'CandleHistory': {
                const s = ensureSymbol(state.symbols, ev.symbol)
                // ev.data items share shape with CandleDto (date, OHLCV).
                s.candles = ev.data
                s.candleTimeframe = (ev.timeframe as Timeframe) ?? '1'
                break
              }
              case 'CandleTick': {
                const s = ensureSymbol(state.symbols, ev.symbol)
                // SDK only streams 1-min ticks; skip when user picked a larger
                // timeframe to avoid polluting the daily/weekly/monthly array.
                if (s.candleTimeframe !== '1') break
                const tick: CandleDto = {
                  date: ev.date,
                  open: ev.open,
                  high: ev.high,
                  low: ev.low,
                  close: ev.close,
                  volume: ev.volume,
                }
                if (!s.candles) {
                  s.candles = [tick]
                } else {
                  const last = s.candles[s.candles.length - 1]
                  if (last.date === tick.date) {
                    s.candles[s.candles.length - 1] = tick
                  } else {
                    s.candles.push(tick)
                  }
                }
                break
              }
            }
          }
        }),

      applyTicker: (symbol, ticker) =>
        set((state) => {
          const s = ensureSymbol(state.symbols, symbol)
          s.ticker = ticker
        }),

      applyTradeHistory: (symbol, trades, fetchedLimit) =>
        set((state) => {
          const s = ensureSymbol(state.symbols, symbol)
          const sliced = trades.slice(0, TAPE_LIMIT)
          s.tape = sliced.map(tickFromRest)
          s.tapeSeedCount = sliced.length
          s.tapeExtraCount = 0
          // Full page back from server → assume more exists. Partial page →
          // exhausted. Compare against the actual fetched limit, not TAPE_LIMIT,
          // so this works regardless of seed page size.
          s.hasMoreTape = trades.length >= fetchedLimit
        }),

      appendOlderTrades: (symbol, older, limit) =>
        set((state) => {
          const s = ensureSymbol(state.symbols, symbol)
          // Boundary dedup: server pagination at offset N may overlap tape[N-1]
          // by one if a live tick landed between pages. Cheap filter via Set.
          const seenSerials = new Set<number>()
          for (const t of s.tape) seenSerials.add(t.serial)
          const filtered = older.filter(
            (t) => t.serial === undefined || !seenSerials.has(t.serial),
          )
          for (const t of filtered) s.tape.push(tickFromRest(t))
          s.tapeExtraCount = (s.tapeExtraCount ?? 0) + filtered.length
          // Server returning a full page → assume more exists. Partial page →
          // exhausted. Uses the un-filtered length because "server-returned
          // count" is what signals end-of-history, not our dedup count.
          s.hasMoreTape = older.length >= limit
        }),

      setTrialTape: (symbol, trades, fetchedLimit) =>
        set((state) => {
          const s = ensureSymbol(state.symbols, symbol)
          s.trialTape = trades.map(tickFromRest)
          s.trialSeedCount = trades.length
          s.trialExtraCount = 0
          s.hasMoreTrialTape = trades.length >= fetchedLimit
        }),

      appendOlderTrialTrades: (symbol, older, limit) =>
        set((state) => {
          const s = ensureSymbol(state.symbols, symbol)
          const existing = s.trialTape ?? []
          const seenSerials = new Set<number>()
          for (const t of existing) seenSerials.add(t.serial)
          const filtered = older.filter(
            (t) => t.serial === undefined || !seenSerials.has(t.serial),
          )
          s.trialTape = existing.concat(filtered.map(tickFromRest))
          s.trialExtraCount = (s.trialExtraCount ?? 0) + filtered.length
          s.hasMoreTrialTape = older.length >= limit
        }),

      applyQuote: (symbol, q) =>
        set((state) => {
          const s = ensureSymbol(state.symbols, symbol)
          s.book = { symbol, bids: q.bids ?? [], asks: q.asks ?? [] }
          s.agg = {
            symbol,
            lastPrice: q.lastPrice,
            openPrice: q.openPrice,
            highPrice: q.highPrice,
            lowPrice: q.lowPrice,
            closePrice: q.closePrice,
            bids: q.bids,
            asks: q.asks,
            total: q.total,
          }
        }),

      setCandles: (symbol, candles, timeframe) =>
        set((state) => {
          const s = ensureSymbol(state.symbols, symbol)
          s.candles = candles
          s.candleTimeframe = timeframe
        }),

      seedIndexHistory: (symbol, closes, previousClose) =>
        set((state) => {
          const existing = state.indices[symbol]
          // Merge seed with any live ticks that arrived during the REST
          // round-trip: seed goes first, then append live tail, deduping at
          // the boundary (seed's last close often matches the latest tick).
          const liveTail = existing?.history ?? []
          const combined = closes.slice()
          for (const v of liveTail) {
            if (combined[combined.length - 1] !== v) combined.push(v)
          }
          if (combined.length > INDEX_HISTORY_LIMIT) {
            combined.splice(0, combined.length - INDEX_HISTORY_LIMIT)
          }
          state.indices[symbol] = {
            value: existing?.value ?? combined[combined.length - 1] ?? 0,
            previousClose,
            history: combined,
          }
        }),
    })),
  ),
)
