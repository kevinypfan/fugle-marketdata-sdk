---
phase: 03-nodejs-binding
plan: 02
subsystem: rest-api
tags: [napi-rs, async, tokio, promise, spawn-blocking]

dependency_graph:
  requires: [03-01]
  provides: [async-rest-api, promise-return-types]
  affects: [03-03, 03-04]

tech_stack:
  added: []
  patterns: [spawn-blocking-for-sync-http, tokio-to-promise-bridge]

key_files:
  created:
    - js/src/client.rs
    - js/index.d.ts
    - js/test_rest.js
  modified:
    - js/Cargo.toml

decisions:
  - spawn-blocking-pattern: Use tokio::task::spawn_blocking to wrap synchronous ureq HTTP calls
  - napi-2x-compatibility: Use napi 2.16 for Rust 1.87 compatibility (3.x requires 1.88+)

metrics:
  duration: 9min
  completed: 2025-01-31
---

# Phase 03 Plan 02: Async REST Client API Summary

**One-liner:** Converted all 11 REST methods to async/Promise-returning API using spawn_blocking pattern for core's sync HTTP.

## What Was Built

### Async REST Client Methods
Converted all REST client methods from synchronous to async, enabling proper Promise-based API in JavaScript:

**StockIntradayClient (5 methods):**
- `quote(symbol)` - Returns Promise<Quote>
- `ticker(symbol)` - Returns Promise<Ticker>
- `candles(symbol, timeframe)` - Returns Promise<Candles>
- `trades(symbol)` - Returns Promise<Trades>
- `volumes(symbol)` - Returns Promise<Volumes>

**FutOptIntradayClient (6 methods):**
- `quote(symbol)` - Returns Promise<Quote>
- `ticker(symbol)` - Returns Promise<Ticker>
- `candles(symbol, timeframe)` - Returns Promise<Candles>
- `trades(symbol)` - Returns Promise<Trades>
- `volumes(symbol)` - Returns Promise<Volumes>
- `products(typ, contractType?)` - Returns Promise<Products>

### Implementation Pattern
```rust
#[napi]
pub async fn quote(&self, symbol: String) -> napi::Result<Value> {
    let inner = self.inner.clone();

    // Use spawn_blocking since core uses synchronous HTTP (ureq)
    let result = tokio::task::spawn_blocking(move || {
        inner.stock().intraday().quote().symbol(&symbol).send()
    })
    .await
    .map_err(|e| napi::Error::from_reason(format!("Task error: {}", e)))?;

    match result {
        Ok(quote) => serde_json::to_value(&quote)
            .map_err(|e| napi::Error::from_reason(e.to_string())),
        Err(e) => Err(to_napi_error(e)),
    }
}
```

### JavaScript Usage
```javascript
const { RestClient } = require('@fugle/marketdata');
const client = new RestClient('your-api-key');

// All methods are now async
const quote = await client.stock.intraday.quote('2330');
console.log(quote.lastPrice);

// Error handling with try/catch
try {
    const data = await client.futopt.intraday.products('FUTURE', 'I');
} catch (err) {
    console.error(err.message);  // Contains error code: "[2002] Authentication error..."
}
```

## Technical Decisions

### Why spawn_blocking?
The core library uses ureq for HTTP requests, which is synchronous. Since napi-rs async functions run on the tokio runtime, we use `spawn_blocking` to:
1. Move the synchronous HTTP call to a blocking thread pool
2. Avoid blocking the Node.js event loop
3. Bridge the sync-to-async boundary cleanly

### Why Clone Inner?
`spawn_blocking` requires `'static` lifetime for the closure. Cloning the inner `RestClient` (which is Arc-based internally) allows the closure to own its data while keeping the outer struct usable.

### Dependency Resolution Issue
During execution, encountered Rust 1.87 incompatibility with napi-rs 3.x (requires Rust 1.88+). The workspace Cargo.toml was configured to pin napi at 3.4.0 with exact versions to avoid resolution to incompatible newer versions.

## Verification Results

All tests passing:
- All 11 methods return Promise instances
- async/await syntax works correctly
- Error rejections contain proper Error instances
- Error codes preserved in format `[XXXX] message`
- TypeScript definitions updated with Promise<any> return types

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] napi-rs version compatibility**
- **Found during:** Task 1
- **Issue:** napi 3.x requires Rust 1.88+, but system has Rust 1.87
- **Fix:** Workspace uses pinned napi 3.4.0 with exact version specifications, plus Cargo.lock pinning of transitive dependencies
- **Files modified:** Cargo.toml, Cargo.lock
- **Commit:** Part of fa5c3e8

**2. [Rule 2 - Missing Critical] TypeScript definition updates**
- **Found during:** Task 3
- **Issue:** Generated index.d.ts had sync return types (NapiResult) instead of Promise
- **Fix:** Updated all REST method return types to Promise<any>
- **Files modified:** js/index.d.ts
- **Commit:** fa5c3e8

## Key Artifacts

| File | Purpose |
|------|---------|
| js/src/client.rs | Async REST client implementation with spawn_blocking |
| js/index.d.ts | TypeScript definitions with Promise return types |
| js/Cargo.toml | Added rt-multi-thread feature for spawn_blocking |
| js/test_rest.js | Test suite for async API behavior |

## Commits

- `fa5c3e8`: feat(03-02): convert REST client to async/Promise API

## Next Phase Readiness

Ready for Plan 03-03 (WebSocket async enhancement):
- tokio runtime properly configured with multi-thread support
- spawn_blocking pattern established for bridging sync-to-async
- Error handling pattern consistent with WebSocket needs
