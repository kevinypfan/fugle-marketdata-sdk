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

/** Mirrors Rust `MarketEventDto` (`#[serde(tag = "kind")]` — "type" collided
 *  with TradesData/AggregatesData/IndicesData's own `type` field). */
export type MarketEvent =
  | ({ kind: 'Aggregate' } & AggregatesData)
  | ({ kind: 'TradeTick' } & TradesData)
  | ({ kind: 'BookSnap' } & BooksData)
  | ({ kind: 'CandleTick' } & CandleData)
  | ({ kind: 'CandleHistory' } & CandlesSnapshot)
  | ({ kind: 'Indices' } & IndicesData)

/** Mirrors Rust `ConnectionStateDto` (`#[serde(tag = "state")]`). */
export type ConnectionState =
  | { state: 'connecting' }
  | { state: 'connected' }
  | { state: 'reconnecting'; attempt: number }
  | { state: 'disconnected'; reason: string }
  | { state: 'failed'; message: string }

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

export type { Timeframe }
