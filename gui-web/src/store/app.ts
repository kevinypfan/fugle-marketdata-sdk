import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { immer } from 'zustand/middleware/immer'
import type {
  AggregatesData,
  BooksData,
  CandleDto,
  ConnectionState,
  FutOptQuote,
  FutOptTicker,
  Market,
  MarketEvent,
  PriceLevel,
  Quote,
  StreamTrade,
  Ticker,
  TotalStats,
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
  /** Stored at first ingestion so UI (Watchlist tab filter, chart fetch path,
   *  subscribe target) can route without re-deriving from symbol shape. */
  market: Market
  ticker?: Ticker
  /** FutOpt-specific contract metadata (expiry dates, type). Only set when
   *  `market === 'futopt'`. Kept separate from `ticker` because its shape is
   *  distinct enough that merging would lose information. */
  futoptTicker?: FutOptTicker
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
  /** FutOpt-only: the highest WS-level serial base (equivalent to
   *  `floor(restSerial / 100)`) covered by the REST seed + accepted WS
   *  TradeTicks so far. Used to drop whole WS events whose batch has
   *  already been fully covered by REST, since one WS event carries N
   *  trades that would otherwise all slip past a tape[0]-only check. */
  futoptTradeSerialHigh?: number
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
  /** Stock-market watchlist (legacy `watchlist` → renamed for clarity). */
  stockWatchlist: string[]
  /** FutOpt watchlist. Can hold either concrete month codes (`TXFD4`) or
   *  near-month aliases the server resolves (`TXF1!`). For aliases, the
   *  server echoes the resolved canonical symbol in WS events — we learn
   *  that mapping in `applyFutoptTicker` and redirect via `symbolAliases`. */
  futoptWatchlist: string[]
  /** Canonical → user-input redirect, populated when REST ticker resolves an
   *  alias (e.g. `TXF1!` → `TXFD4`). `applyEvents` reads this so incoming
   *  WS events for the canonical symbol land in the user-input slot instead
   *  of creating a parallel dead entry the UI can't see. */
  symbolAliases: Record<string, string>
  /** Which tab the Watchlist is currently showing. Drives filtering and
   *  which market new "add" actions go to. Not persisted; resets to 'stock'
   *  on launch. */
  activeMarket: Market
  /** Which FutOpt trading session the bridge is subscribed to right now.
   *  Server delivers disjoint streams for 日盤 (regular) vs 夜盤 (afterhours)
   *  — switching requires unsub all current futopt subs and resub with the
   *  new flag, plus re-seed REST with `session=afterhours`. Defaults by the
   *  caller based on wall-clock time. Not persisted. */
  futoptSession: 'regular' | 'afterhours'
  selected: string | null
  symbols: Record<string, SymbolState>
  /** Per-market connection state. Both markets have independent WSes so
   *  UI can show each market's lifecycle without one overwriting the other. */
  conn: {
    stock: ConnectionState | null
    futopt: ConnectionState | null
  }
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
    stockWatchlist?: string[]
    futoptWatchlist?: string[]
  }) => void
  setApiKey: (key: string) => void
  setEndpoints: (restBaseUrl: string, wsUrl: string) => void
  /** Switch between 股票 / 期貨 tab. Snaps `selected` to the first symbol of
   *  the new market so the chart/tape/book follow the visible tab. */
  setActiveMarket: (market: Market) => void
  /** Store-only update — does not trigger WS re-subscribe. Callers (see
   *  `useMarketBridge.applyFutoptSession`) must orchestrate the unsub/sub
   *  switchover so the bridge state stays aligned. */
  setFutoptSession: (session: 'regular' | 'afterhours') => void
  /** Wipe futopt-bucket SymbolState entries + their alias mappings. Used
   *  before a session switch so stale 日盤/夜盤 tape/book data doesn't
   *  linger until the new seed/WS events arrive. */
  clearFutoptSymbolData: () => void

  // ── watchlist
  setSelected: (symbol: string) => void
  /** Add to the given market's watchlist. Returns true if inserted (false if
   *  already present). */
  addToWatchlist: (market: Market, symbol: string) => boolean
  removeFromWatchlist: (market: Market, symbol: string) => void
  moveToTop: (market: Market, symbol: string) => void
  moveToBottom: (market: Market, symbol: string) => void

  // ── ingestion
  applyConn: (s: ConnectionState) => void
  applyEvents: (batch: MarketEvent[]) => void
  applyTicker: (symbol: string, ticker: Ticker) => void
  applyFutoptTicker: (symbol: string, ticker: FutOptTicker) => void
  applyFutoptQuote: (symbol: string, quote: FutOptQuote) => void
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

/** Pick the FutOpt session most likely active right now in Taipei time.
 *  日盤: 08:45–13:45, 夜盤: 15:00–05:00 (next day). Outside both windows
 *  (13:45–15:00, weekends) default to 'regular' — a single toggle click
 *  flips it. */
function defaultFutoptSession(): 'regular' | 'afterhours' {
  const now = new Date()
  // Convert to Taipei time regardless of machine TZ. Intl.DateTimeFormat
  // is on every browser + Tauri's webview.
  const fmt = new Intl.DateTimeFormat('en-GB', {
    timeZone: 'Asia/Taipei',
    hour: '2-digit',
    minute: '2-digit',
    hour12: false,
  })
  const [hh, mm] = fmt.format(now).split(':').map(Number)
  const m = hh * 60 + mm
  // 15:00 (900) .. 05:00 next day (300) wraps midnight.
  if (m >= 15 * 60 || m < 5 * 60) return 'afterhours'
  return 'regular'
}

function ensureSymbol(
  symbols: Record<string, SymbolState>,
  symbol: string,
  market: Market,
): SymbolState {
  let s = symbols[symbol]
  if (!s) {
    s = { symbol, market, tape: [] }
    symbols[symbol] = s
  }
  return s
}

/** Resolve a symbol's market from store state, defaulting to 'stock' for
 *  unknown symbols. REST seed/apply actions use this to preserve an already-
 *  tagged market when an entry exists, without forcing every caller to thread
 *  a market argument it doesn't know. */
function marketOf(symbols: Record<string, SymbolState>, symbol: string): Market {
  return symbols[symbol]?.market ?? 'stock'
}

/** Build the shared AggregatesData slot from a REST quote response. Stock and
 *  FutOpt quotes share most fields; callers pre-map the `total` sub-shape
 *  (FutOpt carries only `tradeVolume`). Keeping the shape in one place avoids
 *  field-name drift between the two apply paths. */
function buildAggFromQuote(
  symbol: string,
  q: {
    lastPrice?: number
    openPrice?: number
    highPrice?: number
    lowPrice?: number
    closePrice?: number
    previousClose?: number
    bids?: PriceLevel[]
    asks?: PriceLevel[]
    total?: TotalStats
  },
): AggregatesData {
  return {
    symbol,
    lastPrice: q.lastPrice,
    openPrice: q.openPrice,
    highPrice: q.highPrice,
    lowPrice: q.lowPrice,
    closePrice: q.closePrice,
    previousClose: q.previousClose,
    bids: q.bids,
    asks: q.asks,
    total: q.total,
  }
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
      stockWatchlist: [],
      futoptWatchlist: [],
      symbolAliases: {},
      activeMarket: 'stock',
      // Default to the session most likely active right now in Taipei time:
      // 日盤 8:45-13:45, 夜盤 15:00-next-day 05:00. Outside both windows
      // (13:45-15:00, weekends) fall back to 'regular' — user can flip.
      futoptSession: defaultFutoptSession(),
      selected: null,
      symbols: {},
      conn: { stock: null, futopt: null },
      indices: {},

      hydrate: (snapshot) =>
        set((state) => {
          if (snapshot.apiKey !== undefined) state.apiKey = snapshot.apiKey
          if (snapshot.restBaseUrl !== undefined) state.restBaseUrl = snapshot.restBaseUrl
          if (snapshot.wsUrl !== undefined) state.wsUrl = snapshot.wsUrl
          if (snapshot.stockWatchlist) {
            state.stockWatchlist = snapshot.stockWatchlist.slice()
            for (const s of state.stockWatchlist) ensureSymbol(state.symbols, s, 'stock')
          }
          if (snapshot.futoptWatchlist) {
            state.futoptWatchlist = snapshot.futoptWatchlist.slice()
            for (const s of state.futoptWatchlist) ensureSymbol(state.symbols, s, 'futopt')
          }
          if (!state.selected) {
            state.selected =
              state.activeMarket === 'stock'
                ? (state.stockWatchlist[0] ?? state.futoptWatchlist[0] ?? null)
                : (state.futoptWatchlist[0] ?? state.stockWatchlist[0] ?? null)
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

      setActiveMarket: (market) =>
        set((state) => {
          if (state.activeMarket === market) return
          state.activeMarket = market
          // Snap selection to the new market's list so chart/tape/book follow
          // the visible tab. Empty list → null (UI shows empty state).
          const list = market === 'stock' ? state.stockWatchlist : state.futoptWatchlist
          if (!state.selected || !list.includes(state.selected)) {
            state.selected = list[0] ?? null
          }
        }),

      setFutoptSession: (session) =>
        set((state) => {
          state.futoptSession = session
        }),

      clearFutoptSymbolData: () =>
        set((state) => {
          // Wipe futopt entries (they carry stale session data) and any alias
          // entries pointing at them. Keeps stockWatchlist entries untouched.
          const keys = Object.keys(state.symbols)
          for (const k of keys) {
            if (state.symbols[k]?.market === 'futopt') {
              delete state.symbols[k]
            }
          }
          for (const canonical of Object.keys(state.symbolAliases)) {
            const userInput = state.symbolAliases[canonical]
            if (state.futoptWatchlist.includes(userInput)) {
              delete state.symbolAliases[canonical]
            }
          }
        }),

      setSelected: (symbol) =>
        set((state) => {
          state.selected = symbol
        }),

      addToWatchlist: (market, symbol) => {
        let added = false
        set((state) => {
          const list = market === 'stock' ? state.stockWatchlist : state.futoptWatchlist
          if (!list.includes(symbol)) {
            list.push(symbol)
            ensureSymbol(state.symbols, symbol, market)
            added = true
            if (!state.selected) state.selected = symbol
          }
        })
        return added
      },

      removeFromWatchlist: (market, symbol) =>
        set((state) => {
          if (market === 'stock') {
            state.stockWatchlist = state.stockWatchlist.filter((s) => s !== symbol)
          } else {
            state.futoptWatchlist = state.futoptWatchlist.filter((s) => s !== symbol)
          }
          delete state.symbols[symbol]
          // Drop any alias entries that pointed to this symbol — otherwise
          // a future subscribe using the same canonical symbol would keep
          // redirecting to a non-existent key.
          for (const canonical of Object.keys(state.symbolAliases)) {
            if (state.symbolAliases[canonical] === symbol) {
              delete state.symbolAliases[canonical]
            }
          }
          if (state.selected === symbol) {
            const list = market === 'stock' ? state.stockWatchlist : state.futoptWatchlist
            state.selected = list[0] ?? null
          }
        }),

      moveToTop: (market, symbol) =>
        set((state) => {
          const list = market === 'stock' ? state.stockWatchlist : state.futoptWatchlist
          const i = list.indexOf(symbol)
          if (i > 0) {
            list.splice(i, 1)
            list.unshift(symbol)
          }
        }),

      moveToBottom: (market, symbol) =>
        set((state) => {
          const list = market === 'stock' ? state.stockWatchlist : state.futoptWatchlist
          const i = list.indexOf(symbol)
          if (i >= 0 && i < list.length - 1) {
            list.splice(i, 1)
            list.push(symbol)
          }
        }),

      applyConn: (s) =>
        set((state) => {
          // Route to the market's own slot so stock/futopt lifecycles are
          // independent — previously one would clobber the other's status.
          const prev = state.conn[s.market]
          if (
            prev?.state === s.state &&
            JSON.stringify(prev) === JSON.stringify(s)
          ) {
            return
          }
          state.conn[s.market] = s
        }),

      applyEvents: (batch) =>
        set((state) => {
          for (const ev of batch) {
            // Redirect canonical-symbol events to user-input key when an
            // alias is known (populated by applyFutoptTicker). If no alias
            // is recorded, ev.symbol is used as-is.
            const key = state.symbolAliases[ev.symbol] ?? ev.symbol
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
                const s = ensureSymbol(state.symbols, key, ev.marketSource)
                s.agg = ev
                break
              }
              case 'BookSnap': {
                const s = ensureSymbol(state.symbols, key, ev.marketSource)
                s.book = ev
                break
              }
              case 'TradeTick': {
                const s = ensureSymbol(state.symbols, key, ev.marketSource)
                // FutOpt serial dedup: REST serial = WS serial * 100 + seq.
                // One WS event represents a whole base batch; if that base
                // is already covered by REST seed / prior WS events, drop
                // the entire event (not just tape[0]). Runs before the
                // per-trade tape[0] heuristic to catch multi-trade batches.
                if (
                  ev.marketSource === 'futopt' &&
                  ev.serial != null &&
                  s.futoptTradeSerialHigh != null &&
                  ev.serial <= s.futoptTradeSerialHigh
                ) {
                  break
                }
                const time = ev.time ? Math.floor(ev.time / 1000) : Date.now()
                const newest: Tick[] = []
                let last = s.tape[0]?.price
                for (const t of ev.trades) {
                  // Boundary dedup: the first futopt-synthesized TradeTick
                  // after a REST seed echoes the newest seeded exec (server
                  // reads both from the same trade record, so price/size/
                  // ms-time line up exactly). Suppress matches against
                  // tape[0] — O(1) guard, handles the common case without a
                  // Set of all serials. Back-to-back identical executions
                  // would be false-positive-suppressed, but for the user
                  // they're visually indistinguishable anyway.
                  if (s.tape.length > 0) {
                    const top = s.tape[0]
                    if (
                      top.time === time &&
                      top.price === t.price &&
                      top.size === t.size
                    ) {
                      continue
                    }
                  }
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
                // Advance the futopt serial high-water mark so subsequent WS
                // events in the same base get dropped as duplicates.
                if (ev.marketSource === 'futopt' && ev.serial != null) {
                  s.futoptTradeSerialHigh = Math.max(
                    s.futoptTradeSerialHigh ?? 0,
                    ev.serial,
                  )
                }
                break
              }
              case 'CandleHistory': {
                const s = ensureSymbol(state.symbols, key, ev.marketSource)
                // ev.data items share shape with CandleDto (date, OHLCV).
                s.candles = ev.data
                s.candleTimeframe = (ev.timeframe as Timeframe) ?? '1'
                break
              }
              case 'CandleTick': {
                const s = ensureSymbol(state.symbols, key, ev.marketSource)
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
          const s = ensureSymbol(state.symbols, symbol, marketOf(state.symbols, symbol))
          s.ticker = ticker
        }),

      applyFutoptTicker: (symbol, ticker) =>
        set((state) => {
          // Learn alias if the server resolved a near-month input into a
          // canonical contract (e.g. TXF1! → TXFD4). WS events come back
          // tagged with the canonical symbol — without this map, they'd
          // create a parallel dead entry the UI never sees.
          if (ticker.symbol && ticker.symbol !== symbol) {
            state.symbolAliases[ticker.symbol] = symbol
            // If WS events already landed under the canonical key before
            // the alias was learned, fold that data into the user-input
            // entry so DepthBook/TradeTape catch up immediately.
            const orphan = state.symbols[ticker.symbol]
            if (orphan) {
              const live = ensureSymbol(state.symbols, symbol, 'futopt')
              if (orphan.book && !live.book) live.book = orphan.book
              if (orphan.agg && !live.agg) live.agg = orphan.agg
              if (orphan.tape.length > 0 && live.tape.length === 0) {
                live.tape = orphan.tape
              }
              delete state.symbols[ticker.symbol]
            }
          }
          const s = ensureSymbol(state.symbols, symbol, 'futopt')
          s.futoptTicker = ticker
          // Bridge a subset into the generic `ticker` slot so WatchlistRow
          // (which reads `ticker.name` / `ticker.previousClose`) works without
          // branching on market.
          s.ticker = {
            symbol,
            name: ticker.name,
            market: ticker.exchange,
            previousClose: ticker.referencePrice,
          }
        }),

      applyFutoptQuote: (symbol, q) =>
        set((state) => {
          const s = ensureSymbol(state.symbols, symbol, 'futopt')
          s.book = { symbol, bids: q.bids ?? [], asks: q.asks ?? [] }
          s.agg = buildAggFromQuote(symbol, {
            ...q,
            // FutOptTotalStats only carries tradeVolume; map to the shared
            // TotalStats shape so downstream volume readers work unchanged.
            total: q.total ? { tradeVolume: q.total.tradeVolume, tradeValue: 0 } : undefined,
          })
        }),

      applyTradeHistory: (symbol, trades, fetchedLimit) =>
        set((state) => {
          const s = ensureSymbol(state.symbols, symbol, marketOf(state.symbols, symbol))
          const sliced = trades.slice(0, TAPE_LIMIT)
          s.tape = sliced.map(tickFromRest)
          s.tapeSeedCount = sliced.length
          s.tapeExtraCount = 0
          // Full page back from server → assume more exists. Partial page →
          // exhausted. Compare against the actual fetched limit, not TAPE_LIMIT,
          // so this works regardless of seed page size.
          s.hasMoreTape = trades.length >= fetchedLimit
          // FutOpt: REST trade serial = wsSerial * 100 + seq. Track the base
          // of the newest seeded trade so incoming WS TradeTick events whose
          // serial is <= this base are recognized as already-covered.
          if (s.market === 'futopt') {
            const newest = trades[0]?.serial
            s.futoptTradeSerialHigh =
              newest != null ? Math.floor(newest / 100) : undefined
          }
        }),

      appendOlderTrades: (symbol, older, limit) =>
        set((state) => {
          const s = ensureSymbol(state.symbols, symbol, marketOf(state.symbols, symbol))
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
          const s = ensureSymbol(state.symbols, symbol, marketOf(state.symbols, symbol))
          s.trialTape = trades.map(tickFromRest)
          s.trialSeedCount = trades.length
          s.trialExtraCount = 0
          s.hasMoreTrialTape = trades.length >= fetchedLimit
        }),

      appendOlderTrialTrades: (symbol, older, limit) =>
        set((state) => {
          const s = ensureSymbol(state.symbols, symbol, marketOf(state.symbols, symbol))
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
          const s = ensureSymbol(state.symbols, symbol, marketOf(state.symbols, symbol))
          s.book = { symbol, bids: q.bids ?? [], asks: q.asks ?? [] }
          // Stock Quote has no previousClose; Ticker carries it separately
          // and WatchlistRow reads from there. buildAggFromQuote leaves it
          // undefined here, which is correct.
          s.agg = buildAggFromQuote(symbol, q)
        }),

      setCandles: (symbol, candles, timeframe) =>
        set((state) => {
          const s = ensureSymbol(state.symbols, symbol, marketOf(state.symbols, symbol))
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
