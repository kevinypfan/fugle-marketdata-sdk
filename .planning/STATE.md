# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2025-01-30)

**Core value:** API-compatible drop-in replacement for official Fugle SDKs
**Current focus:** Phase 2 Complete - Ready for Phase 3 (Node.js Binding)

## Current Position

Phase: 2 of 6 (Python Binding Enhancement) - COMPLETE
Plan: 5 of 5 in current phase (all plans complete)
Status: Phase 2 complete with full async Python binding and test suite
Last activity: 2026-01-31 — Completed 02-05-PLAN.md (integration tests)

Progress: [███░░░░░░░] 33% (2 of 6 phases complete)

## Performance Metrics

**Velocity:**
- Total plans completed: 8
- Average duration: 7 min
- Total execution time: 0.8 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 01-build-infrastructure | 3 | 11min | 4min |
| 02-python-binding | 5 | 38min | 8min |

**Recent Trend:**
- Last 5 plans: 02-01 (9min), 02-02 (6min), 02-03 (7min), 02-04 (8min), 02-05 (8min)
- Trend: Phase 2 plans consistent at ~7-8min average

*Updated after each plan completion*

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.
Recent decisions affecting current work:

- Priority order: Python → Node.js → C# (Python most mature, C# needs architectural rework)
- C# requires csbindgen (not UniFFI) — UniFFI targets mobile platforms, csbindgen is .NET-appropriate
- Phase 1 workspace migration unblocks parallel Python/Node.js work in Phases 2/3
- **01-01:** Use workspace resolver 2 to prevent feature unification pitfalls
- **01-01:** Bump version to 0.2.0 for workspace migration milestone
- **01-01:** Keep core-only deps (ureq, tokio-tungstenite, etc.) in core/Cargo.toml only
- **01-02:** Standardize package names: fugle-marketdata (Python), @fugle/marketdata (Node.js)
- **01-02:** Use --cargo-name flag in napi build scripts for workspace compatibility
- **01-02:** Separate dev/release targets for each binding language in Makefile
- **01-03:** Use path-based workflow triggering (dorny/paths-filter) to run only affected language workflows
- **01-03:** Test minimal language versions (Python 3.8, Node 18) on Linux only, current versions on all platforms
- **01-03:** Use Swatinem/rust-cache with workspace paths to prevent cache key collisions between bindings
- **02-01:** Use pyo3-async-runtimes (not deprecated pyo3-asyncio) for asyncio integration
- **02-01:** Map core errors to specific Python exception types (ApiError, AuthError, etc.) for better error handling
- **02-01:** Exception hierarchy with inheritance: RateLimitError extends ApiError extends MarketDataError
- **02-02:** Use spawn_blocking to wrap sync ureq HTTP calls (core uses blocking HTTP, not async)
- **02-02:** Type conversion via serde_json::to_value then custom Python dict converters
- **02-02:** Scope limited to intraday endpoints until core implements historical/snapshot
- **02-03:** Keep std::sync::mpsc for FFI compatibility, use spawn_blocking for async polling without holding GIL
- **02-03:** Dual API pattern: preserve callback (on/off) while adding async methods (connect_async, subscribe_async)
- **02-03:** Timeout-based deadlock detection in GIL safety tests (pytest-timeout 10-15s)
- **02-04:** Use python-source = '.' with module-name for maturin mixed layout
- **02-04:** Add pyo3 signature attributes to all methods with optional parameters
- **02-05:** Use pytest-asyncio auto mode for automatic async test discovery
- **02-05:** Skip integration tests automatically when FUGLE_API_KEY not set
- **02-05:** API compatibility tests verify structural parity without network calls

### Pending Todos

None yet.

### Blockers/Concerns

**Phase 1 (Workspace Migration):**
- ✅ RESOLVED: Workspace structure successfully migrated in 01-01
- ✅ RESOLVED: Python binding dev headers handled in Phase 2

**Phase 2 (Python):**
- ✅ RESOLVED: PyO3 0.27 upgrade completed with Bound API migration in 02-01
- ✅ RESOLVED: pyo3-async-runtimes 0.27 added and ready for async API in 02-02
- ✅ RESOLVED: REST async conversion complete for intraday endpoints in 02-02
- ✅ RESOLVED: WebSocket async iterator and async methods complete in 02-03
- ✅ RESOLVED: MutexGuard+await issues fixed (Arc<WebSocketClient> pattern)
- ✅ RESOLVED: Python::with_gil deprecation fixed (→ Python::attach)
- ✅ RESOLVED: Type stubs and PEP 561 compliance complete in 02-04
- ✅ RESOLVED: Integration tests and API compatibility verification complete in 02-05
- ⚠️ PENDING: Historical/snapshot endpoints blocked until core implementation available

**Phase 3 (Node.js):**
- napi-rs 2.16 → 3.6 has ThreadsafeFunction API changes requiring WebSocket callback refactoring
- Memory leak testing needed for Buffer/TypedArray handling

**Phase 4 (C#):**
- UniFFI → csbindgen migration requires complete API redesign from UDL to extern "C" FFI
- C# async bridging strategy needs prototype validation (Task.Run vs. spawn+poll)
- Research identified this as highest complexity phase

**Phase 5 (Distribution):**
- macOS code signing and universal2 builds require Apple Developer account configuration
- Alpine/musl builds need validation if musllinux wheels required beyond manylinux

**Phase 6 (Testing):**
- Need Fugle API staging environment credentials for integration tests
- Performance benchmarking methodology needs definition before claiming speedup

## Session Continuity

Last session: 2026-01-31
Stopped at: Completed 02-05-PLAN.md (integration tests) - Phase 2 complete
Resume file: N/A
Next: Execute Phase 3 (Node.js Binding Enhancement)
