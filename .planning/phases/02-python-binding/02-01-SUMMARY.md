---
phase: 02-python-binding
plan: 01
subsystem: python-bindings
tags: [pyo3, python, async, tokio, pyo3-async-runtimes]

# Dependency graph
requires:
  - phase: 01-build-infrastructure
    provides: Workspace structure with resolver 2 and core library
provides:
  - PyO3 0.27 Bound API throughout Python binding
  - pyo3-async-runtimes 0.27 for future asyncio support
  - 7-level exception hierarchy for Python error handling
  - Maturin 1.11+ build configuration
affects: [02-02-async-api, 05-distribution]

# Tech tracking
tech-stack:
  added:
    - pyo3 0.27 (upgraded from 0.22)
    - pyo3-async-runtimes 0.27
    - maturin 1.11+ (upgraded from 1.0)
  patterns:
    - Bound API for all PyO3 operations
    - Exception hierarchy with create_exception! macro
    - IntoPyObject trait for Python conversions

key-files:
  created: []
  modified:
    - Cargo.toml
    - py/Cargo.toml
    - py/pyproject.toml
    - py/src/lib.rs
    - py/src/errors.rs
    - py/src/callback.rs
    - py/src/websocket.rs
    - py/src/iterator.rs
    - py/src/types.rs

key-decisions:
  - "Use pyo3-async-runtimes (not pyo3-asyncio which is deprecated)"
  - "Map core errors to specific Python exception types for better error handling"
  - "Use Py<PyAny> instead of deprecated PyObject type alias"
  - "Use IntoPyObject trait for Python conversions in 0.27 API"

patterns-established:
  - "Exception hierarchy: Base → Specific (ApiError → RateLimitError)"
  - "Error mapping: HTTP 429 → RateLimitError, auth → AuthError, etc."
  - "Async foundation: docstrings show future asyncio patterns"

# Metrics
duration: 9min
completed: 2026-01-31
---

# Phase 2 Plan 01: PyO3 Foundation Upgrade Summary

**PyO3 0.27 with Bound API and pyo3-async-runtimes 0.27 enabling future native asyncio support**

## Performance

- **Duration:** 9 min
- **Started:** 2026-01-31T00:05:36Z
- **Completed:** 2026-01-31T00:14:36Z
- **Tasks:** 2
- **Files modified:** 9

## Accomplishments
- Upgraded PyO3 from 0.22 to 0.27 with full Bound API migration
- Added pyo3-async-runtimes 0.27 with tokio-runtime feature for future asyncio support
- Implemented 7-level exception hierarchy (MarketDataError → ApiError, AuthError, etc.)
- Migrated all Python binding code to PyO3 0.27 API (IntoPyObject, Bound<PyTuple>, etc.)
- Verified zero GIL Ref deprecation warnings

## Task Commits

Each task was committed atomically:

1. **Task 1: Upgrade workspace dependencies to PyO3 0.27** - `2c948a6` (feat)
2. **Task 2: Migrate lib.rs to Bound API and prepare async module** - `01a8b2f` (feat)

## Files Created/Modified

### Dependency Files
- `Cargo.toml` - Added pyo3 0.27 and pyo3-async-runtimes 0.27 workspace dependencies
- `py/Cargo.toml` - Added pyo3-async-runtimes workspace dependency
- `py/pyproject.toml` - Updated maturin to >=1.11, added pytest dev dependencies

### Source Files
- `py/src/lib.rs` - Updated module docstrings with async examples, registered 7 exception types
- `py/src/errors.rs` - Created exception hierarchy with specific error type mapping
- `py/src/callback.rs` - Migrated to Bound<PyTuple> and IntoPyObject API
- `py/src/websocket.rs` - Updated json_value_to_py to use IntoPyObject
- `py/src/iterator.rs` - Replaced deprecated PyObject with Py<PyAny>
- `py/src/types.rs` - Updated PyDict::new and PyList::empty to new API

## Decisions Made

1. **Use pyo3-async-runtimes over pyo3-asyncio**
   - Rationale: pyo3-asyncio is deprecated, pyo3-async-runtimes is the official successor for asyncio integration

2. **Map core errors to specific Python exception types**
   - Rationale: Better error handling for Python users - catch specific exceptions (RateLimitError, AuthError) instead of generic MarketDataError

3. **Exception hierarchy with inheritance**
   - Rationale: RateLimitError extends ApiError extends MarketDataError - allows catching at multiple levels

4. **Use IntoPyObject trait for conversions**
   - Rationale: PyO3 0.27 deprecated .into_py(), new API is .into_pyobject()?.to_owned().into_any().unbind()

## Deviations from Plan

None - plan executed exactly as written.

All API migration work was planned and completed successfully:
- PyDict::new_bound → PyDict::new (as expected in PyO3 0.27)
- PyList::empty_bound → PyList::empty (as expected in PyO3 0.27)
- .into_py() → IntoPyObject trait (documented migration path)
- PyObject → Py<PyAny> (documented deprecation)

## Issues Encountered

### PyO3 0.27 API changes
- **Issue:** PyO3 0.27 changed multiple API surface areas
- **Resolution:** Systematically migrated all files:
  - Constructor methods: new_bound → new, empty_bound → empty
  - Type conversions: .into_py() → .into_pyobject()
  - Type aliases: PyObject → Py<PyAny>
  - Callback invocation: tuple args → Bound<PyTuple> references

### Borrowed type lifetime management
- **Issue:** IntoPyObject returns Borrowed types that don't implement Copy
- **Resolution:** Used .to_owned().into_any().unbind() pattern for owned Py<PyAny>

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

**Ready for async API development (Plan 02-02):**
- ✅ pyo3-async-runtimes 0.27 available with tokio-runtime feature
- ✅ Exception hierarchy ready for async error handling
- ✅ Bound API throughout for consistency
- ✅ No GIL Ref deprecation warnings

**Foundation for distribution (Phase 05):**
- ✅ Maturin 1.11+ supports modern Python versions
- ✅ PyO3 0.27 supports Python 3.8-3.13

**No blockers identified.** Async API work can proceed immediately.

---
*Phase: 02-python-binding*
*Completed: 2026-01-31*
