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

export interface Tick {
  price: number
  size: number
  bid?: number
  ask?: number
  time: number
  /** -1 = sell hit bid, +1 = buy lifted ask, 0 = unknown */
  direction: -1 | 0 | 1
}

export interface SymbolState {
  symbol: string
  ticker?: Ticker
  agg?: AggregatesData
  book?: BooksData
  tape: Tick[]
  /** newest-last (REST sort=asc). Holds whichever timeframe caller last set. */
  candles?: CandleDto[]
  /** Required for streaming: SDK only pushes 1-min ticks, so CandleTick
   *  only applies when this is '1'. */
  candleTimeframe?: Timeframe
}

export interface AppStore {
  // ── data
  apiKey: string | null
  watchlist: string[]
  selected: string | null
  symbols: Record<string, SymbolState>
  conn: ConnectionState | null
  taiex?: { value: number; previousClose?: number }

  // ── lifecycle
  hydrate: (snapshot: { apiKey?: string | null; watchlist?: string[] }) => void
  setApiKey: (key: string) => void

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
  applyTradeHistory: (symbol: string, trades: Trade[]) => void
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
  }
}

export const useAppStore = create<AppStore>()(
  subscribeWithSelector(
    immer((set) => ({
      apiKey: null,
      watchlist: [],
      selected: null,
      symbols: {},
      conn: null,

      hydrate: (snapshot) =>
        set((state) => {
          if (snapshot.apiKey !== undefined) state.apiKey = snapshot.apiKey
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
            switch (ev.type) {
              case 'Indices': {
                if (ev.symbol === 'IX0001' && ev.index !== undefined) {
                  state.taiex = {
                    value: ev.index,
                    previousClose: state.taiex?.previousClose,
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
                s.tape = newest.concat(s.tape).slice(0, TAPE_LIMIT)
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

      applyTradeHistory: (symbol, trades) =>
        set((state) => {
          const s = ensureSymbol(state.symbols, symbol)
          s.tape = trades.slice(0, TAPE_LIMIT).map(tickFromRest)
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
    })),
  ),
)
