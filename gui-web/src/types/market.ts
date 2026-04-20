// TypeScript mirrors of Rust DTOs (gui-web/src-tauri/src/events.rs).
// Field names follow the SDK's camelCase serde renames.

import type { Timeframe } from './timeframe'

export interface PriceLevel {
  price: number
  size: number
}

export interface StreamTrade {
  price: number
  size: number
  bid?: number
  ask?: number
}

export interface TotalStats {
  tradeValue: number
  tradeVolume: number
  tradeVolumeAtBid?: number
  tradeVolumeAtAsk?: number
  transaction?: number
  time?: number
}

export interface TradesData {
  symbol: string
  type?: string
  exchange?: string
  market?: string
  trades: StreamTrade[]
  total?: TotalStats
  time?: number
  serial?: number
}

export interface BooksData {
  symbol: string
  bids: PriceLevel[]
  asks: PriceLevel[]
  time?: number
  serial?: number
}

export interface AggregatesData {
  symbol: string
  date?: string
  previousClose?: number
  openPrice?: number
  highPrice?: number
  lowPrice?: number
  closePrice?: number
  lastPrice?: number
  lastSize?: number
  bids?: PriceLevel[]
  asks?: PriceLevel[]
  total?: TotalStats
  time?: number
}

export interface CandleData {
  symbol: string
  date: string
  open: number
  high: number
  low: number
  close: number
  volume: number
  average?: number
}

export interface CandlesSnapshot {
  symbol: string
  date: string
  timeframe?: string
  data: Array<{
    date: string
    open: number
    high: number
    low: number
    close: number
    volume: number
  }>
}

export interface IndicesData {
  symbol: string
  index?: number
  time?: number
}

/** Which WS (and therefore which watchlist bucket) the event belongs to.
 *  The bridge tags each batched event with this at emit time so the store
 *  can route to the right SymbolState without a symbol-format heuristic. */
export type Market = 'stock' | 'futopt'

/** Mirrors Rust `TaggedMarketEvent` (market tag + flattened `MarketEventDto`).
 *  `kind` is the payload discriminator (`TradeTick`, `BookSnap`, …).
 *  Routing tag is `marketSource` — not `market` — because stock Aggregates/
 *  Trades already carry `market: string` ("TSE"/"OTC"), which would collide
 *  when the inner event is flattened next to the outer routing tag. */
export type MarketEvent =
  | ({ marketSource: Market; kind: 'Aggregate' } & AggregatesData)
  | ({ marketSource: Market; kind: 'TradeTick' } & TradesData)
  | ({ marketSource: Market; kind: 'BookSnap' } & BooksData)
  | ({ marketSource: Market; kind: 'CandleTick' } & CandleData)
  | ({ marketSource: Market; kind: 'CandleHistory' } & CandlesSnapshot)
  | ({ marketSource: Market; kind: 'Indices' } & IndicesData)

/** Mirrors Rust `MarketConnectionStateDto` — the `market` tag plus flattened
 *  `ConnectionStateDto`. Stock and futopt WSes have independent lifecycles,
 *  so the frontend needs to know which market a state update refers to. */
export type ConnectionState =
  | { market: Market; state: 'connecting' }
  | { market: Market; state: 'connected' }
  | { market: Market; state: 'reconnecting'; attempt: number }
  | { market: Market; state: 'disconnected'; reason: string }
  | { market: Market; state: 'failed'; message: string }

/** REST-seed return shapes. */
export interface CandleDto {
  date: string
  open: number
  high: number
  low: number
  close: number
  volume: number
}

export interface Trade {
  bid?: number
  ask?: number
  price: number
  size: number
  time: number
  serial?: number
  volume?: number
}

export interface Ticker {
  symbol: string
  name?: string
  market?: string
  exchange?: string
  previousClose?: number
  isAttention?: boolean
  isDisposition?: boolean
}

export interface Quote {
  symbol: string
  name?: string
  date: string
  openPrice?: number
  highPrice?: number
  lowPrice?: number
  closePrice?: number
  lastPrice?: number
  lastSize?: number
  avgPrice?: number
  change?: number
  changePercent?: number
  bids: PriceLevel[]
  asks: PriceLevel[]
  total?: TotalStats
}

// ── FutOpt REST seed shapes (separate from Stock; field overlap is partial) ──

export interface FutOptTotalStats {
  tradeVolume: number
  totalBidMatch?: number
  totalAskMatch?: number
}

export interface FutOptLastTrade {
  price: number
  size: number
  time: number
}

export interface FutOptQuote {
  symbol: string
  name?: string
  date: string
  /** "FUTURE" | "OPTION" */
  contractType?: string
  exchange?: string
  previousClose?: number
  openPrice?: number
  openTime?: number
  highPrice?: number
  highTime?: number
  lowPrice?: number
  lowTime?: number
  closePrice?: number
  closeTime?: number
  lastPrice?: number
  lastSize?: number
  avgPrice?: number
  change?: number
  changePercent?: number
  amplitude?: number
  bids: PriceLevel[]
  asks: PriceLevel[]
  total?: FutOptTotalStats
  lastTrade?: FutOptLastTrade
  lastUpdated?: number
}

export interface FutOptTicker {
  symbol: string
  name?: string
  date: string
  /** "FUTURE" | "OPTION" */
  contractType?: string
  exchange?: string
  referencePrice?: number
  startDate?: string
  endDate?: string
  settlementDate?: string
  /** "I"=Index, "S"=Stock, etc. (distinct from FUTURE/OPTION `contractType`). */
  contractSubType?: string
  isDynamicBanding?: boolean
  flowGroup?: number
}

/** FutOpt products list entry (from `/futopt/intraday/products`). Used for a
 *  future "pick a contract" picker; most fields optional so we can render a
 *  sparse row. */
export interface Product {
  symbol: string
  name?: string
  /** "FUTURE" | "OPTION" */
  type?: string
  exchange?: string
  underlyingSymbol?: string
  contractType?: string
  contractSize?: number
  underlyingType?: string
  statusCode?: string
  tradingCurrency?: string
  startDate?: string
  expiryType?: string
}

export type { Timeframe }
