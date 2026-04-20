// Tauri event-name + Taiwan symbol constants shared across the frontend.
// Mirror of Rust constants in `src-tauri/src/bridge.rs`. Strings must match
// exactly — drift here breaks the IPC bridge silently.

export const MARKET_BATCH_EVENT = 'market-batch'
export const CONN_STATE_EVENT = 'connection-state'

export const INDICES_PREFIX = 'IX'
export const TAIEX_SYMBOL = 'IX0001'
export const OTC_SYMBOL = 'IX0043'

/** Indices that the app auto-subscribes on connect and renders in IndicesBar. */
export const INDEX_SYMBOLS = [TAIEX_SYMBOL, OTC_SYMBOL] as const

export const INDEX_LABEL: Record<string, string> = {
  [TAIEX_SYMBOL]: '加權',
  [OTC_SYMBOL]: '櫃買',
}

export function isIndexSymbol(symbol: string): boolean {
  return symbol.startsWith(INDICES_PREFIX)
}
