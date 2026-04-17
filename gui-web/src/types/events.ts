// Tauri event-name + Taiwan symbol constants shared across the frontend.
// Mirror of Rust constants in `src-tauri/src/bridge.rs`. Strings must match
// exactly — drift here breaks the IPC bridge silently.

export const MARKET_BATCH_EVENT = 'market-batch'
export const CONN_STATE_EVENT = 'connection-state'

export const INDICES_PREFIX = 'IX'
export const TAIEX_SYMBOL = 'IX0001'
