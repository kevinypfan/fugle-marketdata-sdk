---
phase: 12-python-config
plan: 02
subsystem: api
tags: [python, pyo3, kwargs, authentication, configuration]

# Dependency graph
requires:
  - phase: 12-01
    provides: ReconnectConfig and HealthCheckConfig PyClasses
provides:
  - RestClient with kwargs constructor supporting all three auth methods
  - WebSocketClient with kwargs constructor accepting config objects
  - Authentication validation (exactly one auth method required)
affects: [12-03, python-usage]

# Tech tracking
tech-stack:
  added: []
  patterns: [kwargs-only constructors, fail-fast validation, config object pattern]

key-files:
  created: []
  modified:
    - py/src/client.rs
    - py/src/websocket.rs

key-decisions:
  - "Use kwargs-only pattern (signature with *) matching official SDK"
  - "Fail-fast validation: check exactly one auth method before construction"
  - "Store configs in WebSocketClient for future propagation to core"
  - "Maintain backwards compatibility via static methods"

patterns-established:
  - "Kwargs constructors with Option<T> parameters and validation"
  - "ValueError for invalid auth combinations"
  - "Config objects passed as &Bound<'_, ConfigType> for extraction"

# Metrics
duration: 1.8min
completed: 2026-02-05
---

# Phase 12 Plan 02: Client Kwargs Constructors Summary

**RestClient and WebSocketClient accept kwargs for auth and config, matching official SDK API**

## Performance

- **Duration:** 1.8 min
- **Started:** 2026-02-05T16:04:32Z
- **Completed:** 2026-02-05T16:06:18Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- RestClient kwargs constructor with auth validation (api_key, bearer_token, sdk_token, base_url)
- WebSocketClient kwargs constructor with config params (auth + reconnect + health_check)
- Exactly-one-auth validation with ValueError on invalid combinations

## Task Commits

Each task was committed atomically:

1. **Task 1: Update RestClient with kwargs constructor** - `d72e90a` (feat)
2. **Task 2: Update WebSocketClient with kwargs and config params** - `63dd98f` (feat)

## Files Created/Modified
- `py/src/client.rs` - RestClient with kwargs constructor and auth validation
- `py/src/websocket.rs` - WebSocketClient with config storage and kwargs constructor

## Decisions Made

**Used `base_url()` method instead of `with_base_url()`**
- During compilation discovered core uses `base_url()` not `with_base_url()`
- Updated to match actual method name in marketdata-core

**Config storage strategy**
- Store configs in WebSocketClient struct for future use
- Added TODO comments for propagation to child clients (StockWebSocketClient, FutOptWebSocketClient)
- Configs will be wired to core once core WebSocket supports runtime configuration

**Backwards compatibility maintained**
- RestClient keeps `with_bearer_token()` and `with_sdk_token()` static methods
- Existing code using old API continues to work

## Deviations from Plan

None - plan executed exactly as written

## Issues Encountered

**Compilation method name mismatch**
- Plan specified `with_base_url()` but actual method is `base_url()`
- Fixed immediately during Task 1 compilation
- No impact on functionality

## Next Phase Readiness

Ready for Phase 12-03 (Integration Tests):
- Both clients accept kwargs with auth validation
- Configs stored and ready for use
- All validation working as specified

**Remaining work:**
- Future enhancement: Wire stored configs to core WebSocket (blocked on core support)
- Future enhancement: Propagate configs to child clients (StockWebSocketClient, FutOptWebSocketClient)

---
*Phase: 12-python-config*
*Completed: 2026-02-05*
