import { invoke } from '@tauri-apps/api/core'
import type {
  CandleDto,
  FutOptQuote,
  FutOptTicker,
  Product,
  Quote,
  Ticker,
  Timeframe,
  Trade,
} from './types/market'

export interface AppError {
  kind: 'sdk' | 'not_connected' | 'missing_api_key' | 'other'
  message?: string
}

function unwrap<T>(p: Promise<T>): Promise<T> {
  return p.catch((e: unknown) => {
    // Tauri rejects Result<_, AppError> with the serialized struct directly.
    // Stringify so the error surfaces in console without needing to expand the
    // collapsed Object; fall back to raw for non-object rejections.
    const err = e as AppError | undefined
    if (err && typeof err === 'object') {
      console.error(`[tauri command] kind=${err.kind} message=${err.message ?? '(none)'}`)
    } else {
      console.error('[tauri command]', e)
    }
    throw e
  })
}

export interface FetchTradesOptions {
  offset?: number
  limit?: number
  isTrial?: boolean
}

export interface FetchFutOptTradesOptions {
  offset?: number
  limit?: number
  afterHours?: boolean
}

export const api = {
  connect: (apiKey: string, wsUrl: string | null) =>
    unwrap(invoke<void>('connect', { apiKey, wsUrl })),
  connectFutopt: (apiKey: string, wsUrl: string | null) =>
    unwrap(invoke<void>('connect_futopt', { apiKey, wsUrl })),
  disconnect: () => unwrap(invoke<void>('disconnect')),
  subscribe: (symbol: string) => unwrap(invoke<void>('subscribe', { symbol })),
  unsubscribe: (symbol: string) => unwrap(invoke<void>('unsubscribe', { symbol })),
  subscribeFutopt: (symbol: string, afterHours?: boolean) =>
    unwrap(invoke<void>('subscribe_futopt', { symbol, afterHours })),
  unsubscribeFutopt: (symbol: string, afterHours?: boolean) =>
    unwrap(invoke<void>('unsubscribe_futopt', { symbol, afterHours })),
  fetchCandles: (symbol: string, timeframe: Timeframe, restBaseUrl: string | null) =>
    unwrap(invoke<CandleDto[]>('fetch_candles', { symbol, timeframe, restBaseUrl })),
  fetchTicker: (symbol: string, restBaseUrl: string | null) =>
    unwrap(invoke<Ticker>('fetch_ticker', { symbol, restBaseUrl })),
  fetchTrades: (
    symbol: string,
    restBaseUrl: string | null,
    opts?: FetchTradesOptions,
  ) => unwrap(invoke<Trade[]>('fetch_trades', { symbol, restBaseUrl, ...opts })),
  fetchQuote: (symbol: string, restBaseUrl: string | null) =>
    unwrap(invoke<Quote>('fetch_quote', { symbol, restBaseUrl })),

  fetchFutoptTicker: (symbol: string, restBaseUrl: string | null) =>
    unwrap(invoke<FutOptTicker>('fetch_futopt_ticker', { symbol, restBaseUrl })),
  fetchFutoptQuote: (
    symbol: string,
    restBaseUrl: string | null,
    afterHours?: boolean,
  ) =>
    unwrap(
      invoke<FutOptQuote>('fetch_futopt_quote', { symbol, restBaseUrl, afterHours }),
    ),
  fetchFutoptTrades: (
    symbol: string,
    restBaseUrl: string | null,
    opts?: FetchFutOptTradesOptions,
  ) => unwrap(invoke<Trade[]>('fetch_futopt_trades', { symbol, restBaseUrl, ...opts })),
  fetchFutoptCandles: (
    symbol: string,
    timeframe: Timeframe,
    restBaseUrl: string | null,
    afterHours?: boolean,
  ) =>
    unwrap(
      invoke<CandleDto[]>('fetch_futopt_candles', {
        symbol,
        timeframe,
        restBaseUrl,
        afterHours,
      }),
    ),
  fetchFutoptProducts: (restBaseUrl: string | null) =>
    unwrap(invoke<Product[]>('fetch_futopt_products', { restBaseUrl })),
}
