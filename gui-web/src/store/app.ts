import { create } from 'zustand'
import { subscribeWithSelector } from 'zustand/middleware'
import { immer } from 'zustand/middleware/immer'
import type {
  AggregatesData,
  BooksData,
  ConnectionState,
  MarketEvent,
  Quote,
  StreamTrade,
  Ticker,
  Trade,
} from '../types/market'

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
}

function ensureSymbol(symbols: Record<string, SymbolState>, symbol: string): SymbolState {
  let s = symbols[symbol]
  if (!s) {
    s = { symbol, tape: [] }
    symbols[symbol] = s
  }
  return s
}

function tickFromStream(t: StreamTrade, time: number, prevPrice?: number): Tick {
  let direction: -1 | 0 | 1 = 0
  if (t.bid !== undefined && t.ask !== undefined) {
    if (t.price >= t.ask) direction = 1
    else if (t.price <= t.bid) direction = -1
  }
  if (direction === 0 && prevPrice !== undefined) {
    if (t.price > prevPrice) direction = 1
    else if (t.price < prevPrice) direction = -1
  }
  return {
    price: t.price,
    size: t.size,
    bid: t.bid,
    ask: t.ask,
    time,
    direction,
  }
}

function tickFromRest(t: Trade): Tick {
  let direction: -1 | 0 | 1 = 0
  if (t.bid !== undefined && t.ask !== undefined) {
    if (t.price >= t.ask) direction = 1
    else if (t.price <= t.bid) direction = -1
  }
  return {
    price: t.price,
    size: t.size,
    bid: t.bid,
    ask: t.ask,
    time: Math.floor(t.time / 1000),
    direction,
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
                if (ev.bids?.length || ev.asks?.length) {
                  s.book = {
                    symbol: ev.symbol,
                    bids: ev.bids ?? [],
                    asks: ev.asks ?? [],
                    time: ev.time,
                  }
                }
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
                let last = s.tape[0]?.price
                for (const t of ev.trades) {
                  const tick = tickFromStream(t, time, last)
                  s.tape.unshift(tick)
                  last = t.price
                }
                if (s.tape.length > TAPE_LIMIT) s.tape.length = TAPE_LIMIT
                break
              }
              case 'CandleTick':
              case 'CandleHistory':
                // chart wiring deferred to iteration 8
                break
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
          // REST returns newest-first with epoch microseconds; same shape as tape.
          s.tape = trades.slice(0, TAPE_LIMIT).map(tickFromRest)
        }),

      applyQuote: (symbol, q) =>
        set((state) => {
          const s = ensureSymbol(state.symbols, symbol)
          // Quote is the off-hours equivalent of the WS Aggregate stream:
          // populates depth, last price, and OHLC even when WS is silent.
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
    })),
  ),
)
