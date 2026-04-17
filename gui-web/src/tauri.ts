import { invoke } from '@tauri-apps/api/core'
import type { CandleDto, Quote, Ticker, Timeframe, Trade } from './types/market'

export interface AppError {
  kind: 'sdk' | 'not_connected' | 'missing_api_key' | 'other'
  message?: string
}

function unwrap<T>(p: Promise<T>): Promise<T> {
  return p.catch((e) => {
    // Tauri rejects Result<_, AppError> with the serialized struct directly.
    console.error('[tauri command]', e)
    throw e as AppError
  })
}

export const api = {
  connect: (apiKey: string) => unwrap(invoke<void>('connect', { apiKey })),
  disconnect: () => unwrap(invoke<void>('disconnect')),
  subscribe: (symbol: string) => unwrap(invoke<void>('subscribe', { symbol })),
  unsubscribe: (symbol: string) => unwrap(invoke<void>('unsubscribe', { symbol })),
  fetchCandles: (symbol: string, timeframe: Timeframe) =>
    unwrap(invoke<CandleDto[]>('fetch_candles', { symbol, timeframe })),
  fetchTicker: (symbol: string) => unwrap(invoke<Ticker>('fetch_ticker', { symbol })),
  fetchTrades: (symbol: string) => unwrap(invoke<Trade[]>('fetch_trades', { symbol })),
  fetchQuote: (symbol: string) => unwrap(invoke<Quote>('fetch_quote', { symbol })),
}
