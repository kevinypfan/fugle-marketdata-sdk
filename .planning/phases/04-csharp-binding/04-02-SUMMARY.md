---
phase: 04-csharp-binding
plan: 02
subsystem: api
tags: [csharp, ffi, rest, async, callbacks, csbindgen]

# Dependency graph
requires:
  - phase: 04-01
    provides: csbindgen foundation with error codes and panic recovery
provides:
  - REST client FFI handle with lifecycle management (new/free)
  - 8 async callback-based REST endpoints (5 stock, 3 futopt intraday)
  - JSON serialization of REST responses via serde_json
  - spawn_blocking bridge for blocking HTTP to async
  - usize pointer conversion pattern for Send compatibility
affects: [04-04, 04-05]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Callback-based async FFI with (user_data, result_json, error_code)"
    - "Convert function pointers to usize for Send trait compatibility"
    - "spawn_blocking wraps blocking HTTP (ureq) in async context"
    - "JSON serialization for all REST responses"

key-files:
  created:
    - cs/src/rest_client.rs
  modified:
    - cs/src/lib.rs

key-decisions:
  - "Callback pattern instead of polling: C# Task-based async more natural for callbacks"
  - "Convert callback/user_data pointers to usize: Required for Send trait across async boundaries"
  - "JSON strings for all responses: Simplifies FFI boundary, C# deserializes with System.Text.Json"

patterns-established:
  - "FFI async pattern: capture pointers as usize → spawn task → reconstruct pointers → invoke callback"
  - "Error handling: catch_panic wraps entire function, callback invoked even on panic"

# Metrics
duration: 4min
completed: 2026-01-31
---

# Phase 04 Plan 02: REST Client FFI Summary

**Callback-based async REST client FFI with 8 intraday endpoints (stock/futopt) using spawn_blocking for blocking HTTP**

## Performance

- **Duration:** 4 min
- **Started:** 2026-01-31T04:07:36Z
- **Completed:** 2026-01-31T04:11:39Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- REST client handle lifecycle (new with API key, free with panic safety)
- 5 stock intraday endpoints (quote, trades, ticker, candles, volumes)
- 3 futopt intraday endpoints (quote, ticker, products)
- Callback-based async pattern with usize conversion for Send compatibility
- spawn_blocking wraps blocking ureq HTTP calls in async context
- JSON serialization for all REST responses via serde_json

## Task Commits

Each task was committed atomically:

1. **Task 1: REST client handle management** - `e8f01d7` (feat)
2. **Task 2: Async FFI exports for all intraday endpoints** - `cae2097` (feat)

## Files Created/Modified
- `cs/src/rest_client.rs` - REST client FFI exports with 8 async endpoints
- `cs/src/lib.rs` - Module exports including rest_client

## Decisions Made

**1. Callback pattern instead of polling**
- Rationale: C# Task-based async model naturally handles callbacks via TaskCompletionSource
- Alternative considered: Polling with fugle_rest_poll_result()
- Selected approach aligns with Python/Node.js async patterns

**2. Convert callback/user_data pointers to usize**
- Rationale: Raw pointers (*mut c_void, function pointers) are not Send
- Solution: Cast to usize before moving into async task, reconstruct inside
- Safe because C# keeps delegates alive until callback completes

**3. JSON strings for all responses**
- Rationale: Simplifies FFI boundary, avoids complex struct marshaling
- C# deserializes with System.Text.Json.JsonSerializer
- Alternative: Direct struct marshaling rejected due to complexity and error-prone alignment

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Send trait not implemented for raw pointers**
- **Found during:** Task 2 (Initial implementation of async endpoints)
- **Issue:** Compiler error: future cannot be sent between threads safely (*mut c_void not Send)
- **Fix:** Convert callback and user_data to usize before moving into async task, reconstruct with transmute inside
- **Files modified:** cs/src/rest_client.rs
- **Verification:** cargo build succeeds, all 8 endpoints compile
- **Committed in:** cae2097 (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Fixing Send trait issue was essential for async execution. Pattern now established for all future FFI async functions. No scope creep.

## Issues Encountered
None - plan executed smoothly after fixing Send trait issue.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- REST client FFI complete, ready for 04-04 (C# wrapper with Task-based async)
- WebSocket FFI (04-03) already implemented in parallel
- Callback pattern established for future async endpoints

---
*Phase: 04-csharp-binding*
*Completed: 2026-01-31*
