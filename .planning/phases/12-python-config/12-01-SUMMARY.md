---
phase: 12-python-config
plan: 01
subsystem: bindings
tags: [pyo3, python, config, validation, ffi]

# Dependency graph
requires:
  - phase: 08-validation
    provides: Core validation logic (HealthCheckConfig, ReconnectionConfig)
  - phase: 09-python-foundation
    provides: PyO3 0.27 async foundation, ReconnectConfig partial implementation
provides:
  - HealthCheckConfig PyClass with kwargs constructor and core validation
  - ReconnectConfig aligned with core field names (max_attempts, initial_delay_ms)
  - Config validation at construction time via core's validation logic
  - Both configs are importable from marketdata_py module
affects: [13-nodejs-config, 14-csharp-config, 12-02, 12-03]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Config PyClass pattern: kwargs-only constructor with #[pyo3(signature)]"
    - "Core validation delegation: call core::Config::new() to validate, fail with PyValueError"
    - "FFI conversion: to_core() method converts validated Python config to core config"
    - "Field exposure: #[pyo3(get)] for read-only config fields"

key-files:
  created: []
  modified:
    - py/src/websocket.rs
    - py/src/lib.rs
    - py/marketdata_py/__init__.py

key-decisions:
  - "ReconnectConfig field rename is breaking change (acceptable per CONTEXT.md)"
  - "Config fields are immutable after construction (#[pyo3(get)] only, no set)"
  - "Validation happens at construction time (fail-fast principle)"
  - "to_core() uses .expect() since validation already passed"

patterns-established:
  - "Config validation pattern: Python layer calls core validation, maps errors to PyValueError"
  - "Millisecond FFI boundary: store ms as u64 in Python, convert to Duration for core"
  - "Default constants alignment: use core's DEFAULT_* constants for consistency"

# Metrics
duration: 12min
completed: 2026-02-05
---

# Phase 12 Plan 01: Python Config Exposure Summary

**HealthCheckConfig and ReconnectConfig PyClasses with kwargs constructors delegating validation to Phase 8 core logic**

## Performance

- **Duration:** 12 min
- **Started:** 2026-02-05T12:46:22Z
- **Completed:** 2026-02-05T12:58:39Z
- **Tasks:** 3
- **Files modified:** 3

## Accomplishments
- HealthCheckConfig PyClass with enabled, interval_ms, max_missed_pongs fields
- ReconnectConfig renamed fields (max_retries→max_attempts, base_delay_ms→initial_delay_ms)
- Both configs use kwargs-only constructors with sensible defaults
- Validation via core's validation logic with fail-fast ValueError
- Both configs registered and importable from marketdata_py

## Task Commits

Each task was committed atomically:

1. **Task 1: Add HealthCheckConfig PyClass** - `e2e200a` (feat)
   - Includes Task 2 changes since both modified py/src/websocket.rs
2. **Task 2: Update ReconnectConfig field names** - (included in e2e200a)
3. **Task 3: Register HealthCheckConfig in module** - (included in e2e200a)

**Plan metadata:** Not created (all changes in single commit)

## Files Created/Modified
- `py/src/websocket.rs` - Added HealthCheckConfig PyClass, updated ReconnectConfig field names and validation
- `py/src/lib.rs` - Registered HealthCheckConfig in module
- `py/marketdata_py/__init__.py` - Exported HealthCheckConfig from package

## Decisions Made
- ReconnectConfig field rename is a breaking change (acceptable per 12-CONTEXT.md: "No deprecation needed — SDK not formally released")
- Config fields use #[pyo3(get)] only (no set) for immutability after construction
- Validation happens at construction time (fail-fast Python convention)
- to_core() method uses .expect() since validation already happened in __new__

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None - straightforward implementation following Phase 9 PyO3 patterns and Phase 8 validation logic.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Ready for Plan 12-02 (RestClient options constructor):
- Config classes are complete and validated
- Pattern established for config validation via core
- Ready to be used as parameters in client constructors

Ready for Phase 13 (Node.js config):
- Can follow same validation pattern (delegate to core)
- Millisecond FFI boundary pattern established

No blockers or concerns.

---
*Phase: 12-python-config*
*Completed: 2026-02-05*
