---
phase: 13-nodejs-config-exposure
plan: 02
subsystem: api
tags: [napi-rs, typescript, nodejs, rest, websocket, config]

# Dependency graph
requires:
  - phase: 13-01
    provides: RestClientOptions and WebSocketClientOptions napi structs
provides:
  - RestClient constructor accepts options object with exactly-one-auth validation
  - WebSocketClient constructor accepts options object with config validation
  - Breaking change: String constructor removed per Phase 13 CONTEXT.md
affects: [13-03-typescript-types]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Options object constructor pattern (matching official SDK)"
    - "Exactly-one-auth validation at construction time"
    - "Config validation delegation to core (ReconnectionConfig::new, HealthCheckConfig::new)"
    - "napi::Result<Self> for constructor error handling"

key-files:
  created: []
  modified: [js/src/client.rs, js/src/websocket.rs]

key-decisions:
  - "Breaking change: Removed string constructor, only options object accepted"
  - "Error message format matches Python: 'Provide exactly one of: apiKey, bearerToken, sdkToken'"
  - "WebSocketClient configs validated but stored as _ (ConnectionConfig doesn't accept them yet)"
  - "RestClient uses base_url() method (not with_base_url) per Phase 12-02 learning"

patterns-established:
  - "Constructor validation pattern: auth count → validate exactly one → build auth → create client"
  - "Config validation pattern: extract Option fields → apply defaults → call core::Config::new()"
  - "Unit test pattern: helper function for test options creation"

# Metrics
duration: 2min
completed: 2026-02-06
---

# Phase 13 Plan 02: Node.js Config Exposure Summary

**RestClient and WebSocketClient constructors accept options objects with exactly-one-auth validation and config delegation to core**

## Performance

- **Duration:** 2 min
- **Started:** 2026-02-06T02:47:04Z
- **Completed:** 2026-02-06T02:48:54Z
- **Tasks:** 3
- **Files modified:** 2

## Accomplishments
- RestClient constructor accepts RestClientOptions instead of String
- WebSocketClient constructor accepts WebSocketClientOptions with nested config validation
- Exactly-one-auth validation (apiKey, bearerToken, sdkToken) at construction time
- Config validation delegates to core::ReconnectionConfig::new() and core::HealthCheckConfig::new()
- Comprehensive unit tests for all constructor scenarios including error paths

## Task Commits

Each task was committed atomically:

1. **Task 1: Modify RestClient constructor to accept options object** - `026de78` (feat)
2. **Task 2: Modify WebSocketClient constructor to accept options object** - `726f3c5` (feat)
3. **Task 3: Update unit tests for new constructors** - `71bd92b` (test)

## Files Created/Modified
- `js/src/client.rs` - RestClient constructor accepts RestClientOptions with auth validation
- `js/src/websocket.rs` - WebSocketClient constructor accepts WebSocketClientOptions with config validation

## Decisions Made

**1. Breaking change: String constructor removal**
- Decision: Remove old `new RestClient("api-key")` constructor completely
- Rationale: Phase 13 CONTEXT.md specifies clean break for v0.3.0, no deprecation warnings
- Impact: All users must migrate to options object pattern

**2. Error message format consistency**
- Decision: Use "Provide exactly one of: apiKey, bearerToken, sdkToken"
- Rationale: Matches Python Phase 12-02 error message format for consistency
- Implementation: Same validation logic and error message across Python/Node.js

**3. Config validation at construction time**
- Decision: Validate ReconnectOptions and HealthCheckOptions via core::Config::new()
- Rationale: Fail-fast principle - catch invalid configs immediately, not at connect() time
- Pattern: Extract Option fields → apply defaults → call core validation → return Result

**4. Configs validated but not yet used**
- Decision: Store validated configs as `_reconnect_cfg` and `_health_check_cfg` (unused)
- Rationale: ConnectionConfig doesn't accept these configs yet (Phase 13-03 will wire them)
- Comment: Added TODO explaining future propagation to child clients

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None - plan executed smoothly.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

**Ready for Phase 13-03 (TypeScript type generation):**
- Constructors implemented and accept options objects
- Validation logic complete with proper error messages
- Unit tests cover all constructor scenarios
- TypeScript types will auto-generate from napi structs via napi-rs

**No blockers:** All constructor changes complete and tested.

## Self-Check: PASSED

All files and commits verified:
- js/src/client.rs modified
- js/src/websocket.rs modified
- Commit 026de78 exists (RestClient constructor)
- Commit 726f3c5 exists (WebSocketClient constructor)
- Commit 71bd92b exists (unit tests)

---
*Phase: 13-nodejs-config-exposure*
*Completed: 2026-02-06*
