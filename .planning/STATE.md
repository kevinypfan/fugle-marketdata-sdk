# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2025-01-30)

**Core value:** API-compatible drop-in replacement for official Fugle SDKs
**Current focus:** Phase 3 - Node.js Binding Enhancement (In Progress)

## Current Position

Phase: 3 of 6 (Node.js Binding Enhancement)
Plan: 3 of 4 in current phase
Status: Completed 03-03 (TypeScript Type Definitions)
Last activity: 2026-01-31 — Completed 03-03-PLAN.md (TypeScript type definitions)

Progress: [████░░░░░░] 44% (~11 of 25 plans complete)

## Performance Metrics

**Velocity:**
- Total plans completed: 11
- Average duration: 7 min
- Total execution time: ~1.3 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 01-build-infrastructure | 3 | 11min | 4min |
| 02-python-binding | 5 | 38min | 8min |
| 03-nodejs-binding | 3 | 28min | 9min |

**Recent Trend:**
- Last 5 plans: 02-04 (8min), 02-05 (8min), 03-01 (10min), 03-02 (10min), 03-03 (8min)
- Trend: Consistent execution time for Node.js binding plans

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
- **03-01:** napi-rs 3.4 pinned for Rust 1.87 compatibility (3.8+ requires Rust 1.88)
- **03-01:** ThreadsafeFunction wrapped in Arc for safe cross-thread callback access
- **03-01:** @napi-rs/cli upgraded to 3.5.1 for napi-rs 3.x compatibility
- **03-02:** All REST methods converted to async with spawn_blocking for non-blocking I/O
- **03-02:** tokio rt-multi-thread feature required for spawn_blocking
- **03-03:** Separate types.d.ts with postbuild script to prepend to generated index.d.ts
- **03-03:** Use #[napi(ts_return_type = "Promise<T>")] for explicit TypeScript return types
- **03-03:** Runtime validation via validate_types.js to verify TS matches Rust JSON

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
- ✅ RESOLVED: napi-rs 3.4 upgrade complete with Arc<ThreadsafeFunction> pattern in 03-01
- ✅ RESOLVED: @napi-rs/cli upgraded to 3.5.1 for build compatibility in 03-01
- ✅ RESOLVED: REST async conversion complete in 03-02
- ✅ RESOLVED: TypeScript type definitions complete in 03-03 (no 'any' types)
- ⚠️ PENDING: Memory leak testing for Buffer/TypedArray handling

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
Stopped at: Completed 03-03-PLAN.md (TypeScript type definitions)
Resume file: N/A
Next: Execute 03-04-PLAN.md (Documentation and examples)
