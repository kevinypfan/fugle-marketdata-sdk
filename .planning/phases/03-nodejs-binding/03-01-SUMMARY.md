---
phase: 03-nodejs-binding
plan: 01
subsystem: node-binding
tags: [napi-rs, websocket, threadsafe-function, upgrade]

dependency-graph:
  requires: [01-build-infrastructure]
  provides: [napi-rs-3x-upgrade, arc-threadsafe-pattern]
  affects: [03-02, 03-03]

tech-stack:
  added:
    - napi 3.4.0 (from 2.16)
    - napi-derive 3.3.0 (from 2.16)
    - napi-build 2.2.4 (from 2.1.3)
    - "@napi-rs/cli" 3.5.1 (from 2.18.4)
  patterns:
    - Arc<ThreadsafeFunction> for thread-safe callback sharing
    - ThreadsafeFunctionCallMode::NonBlocking for event loop safety

key-files:
  created:
    - js/test_event_loop.js
  modified:
    - Cargo.toml
    - Cargo.lock
    - js/Cargo.toml
    - js/src/websocket.rs
    - js/package.json
    - js/package-lock.json

decisions:
  - id: napi-version-pin
    choice: "Pin napi 3.4.0 (MSRV 1.82) for Rust 1.87 compatibility"
    rationale: "napi 3.5+ requires Rust 1.88 which is not yet released"
  - id: arc-threadsafe-pattern
    choice: "Use Arc<ThreadsafeFunction> instead of raw ThreadsafeFunction"
    rationale: "napi-rs 3.x removed .clone() on ThreadsafeFunction; Arc provides thread-safe sharing"
  - id: napi-cli-upgrade
    choice: "Upgrade @napi-rs/cli to 3.5.1"
    rationale: "CLI 2.x is incompatible with napi-rs 3.x macros and build process"

metrics:
  duration: 10min
  completed: 2026-01-31
---

# Phase 03 Plan 01: napi-rs 3.x Upgrade with ThreadsafeFunction Refactoring

**One-liner:** Upgrade napi-rs 2.16 to 3.4 with Arc-wrapped ThreadsafeFunction pattern for thread-safe WebSocket callbacks.

## What Was Done

### Task 1: Upgrade napi-rs workspace dependencies to 3.4
- Updated workspace Cargo.toml with napi = "=3.4.0", napi-derive = "=3.3.0", napi-build = "=2.2.4"
- Pinned exact versions to avoid resolution to 3.5+ requiring Rust 1.88
- Pinned transitive deps (napi-sys 3.0.1, napi-derive-backend 3.0.0, libloading 0.8.9) for MSRV compatibility

### Task 2: Refactor WebSocket ThreadsafeFunction to use Arc pattern
- Removed ErrorStrategy import (not in napi-rs 3.x threadsafe_function module)
- Changed JsCallback type alias to Arc<ThreadsafeFunction<String>>
- Updated on() methods to wrap callbacks in Arc before storing
- Updated fire_callback to use Arc::clone for thread-safe access
- API compatible with both napi 2.x and 3.x (call(Ok(data), mode) works for both)

### Task 3: Verify WebSocket functionality and event loop integration
- Upgraded @napi-rs/cli from 2.18.4 to 3.5.1
- Updated napi config: name -> binaryName, triples -> targets
- Updated build scripts: --cargo-name -> -p (package)
- Created event loop blocking test (test_event_loop.js)
- Verified WebSocketClient instantiation and all methods accessible

## Key Files Changed

| File | Change |
|------|--------|
| Cargo.toml | napi-rs version upgrade with exact pinning |
| Cargo.lock | Dependency resolution with pinned transitive deps |
| js/src/websocket.rs | Arc<ThreadsafeFunction> pattern implementation |
| js/package.json | @napi-rs/cli upgrade and napi config migration |
| js/test_event_loop.js | Event loop blocking test for JS-02 verification |

## Verification Results

```
cargo build -p marketdata-js  # Compiles without errors
npm run build:debug           # Produces .node file
node -e "..."                 # WebSocketClient instantiation works
                              # Methods (connect, on, subscribe, disconnect) accessible
test_event_loop.js            # Skips gracefully without API key
```

## API Pattern

```rust
// Before (napi 2.x)
pub type JsCallback = ThreadsafeFunction<String, ErrorStrategy::CalleeHandled>;

// After (napi 3.x compatible)
pub type JsCallback = Arc<ThreadsafeFunction<String>>;

// on() method wraps callback in Arc
pub fn on(&self, event: String, callback: ThreadsafeFunction<String>) -> napi::Result<()> {
    let arc_callback = Arc::new(callback);
    // ... store arc_callback
}

// fire_callback uses Arc::clone
fn fire_callback(callbacks: &Arc<Mutex<EventCallbacks>>, event: &str, data: String) {
    if let Some(callback) = callback {
        let callback_ref = Arc::clone(callback);
        callback_ref.call(Ok(data), ThreadsafeFunctionCallMode::NonBlocking);
    }
}
```

## Deviations from Plan

### [Rule 3 - Blocking] @napi-rs/cli version incompatibility
- **Found during:** Task 3
- **Issue:** @napi-rs/cli 2.x doesn't support napi-rs 3.x macros, build fails with "missing environment variables"
- **Fix:** Upgraded @napi-rs/cli to 3.5.1, updated napi config (binaryName, targets) and build scripts (-p flag)
- **Files modified:** js/package.json, js/package-lock.json

## Next Phase Readiness

- [x] napi-rs 3.4 upgrade complete
- [x] ThreadsafeFunction Arc pattern implemented
- [x] @napi-rs/cli upgraded for 3.x compatibility
- [x] WebSocket client verified working
- [ ] Live API test requires FUGLE_API_KEY (test infrastructure ready)

**Ready for:** 03-02-PLAN.md (REST client Promise API upgrade)
