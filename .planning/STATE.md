# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2025-01-30)

**Core value:** API-compatible drop-in replacement for official Fugle SDKs
**Current focus:** Phase 2 - Python Binding Enhancement

## Current Position

Phase: 2 of 6 (Python Binding Enhancement)
Plan: 1 of 3 in current phase
Status: In progress
Last activity: 2026-01-31 — Completed 02-01-PLAN.md (PyO3 0.27 upgrade)

Progress: [██░░░░░░░░] 17% (1.17 of 6 phases complete)

## Performance Metrics

**Velocity:**
- Total plans completed: 4
- Average duration: 5 min
- Total execution time: 0.33 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 01-build-infrastructure | 3 | 11min | 4min |
| 02-python-binding | 1 | 9min | 9min |

**Recent Trend:**
- Last 5 plans: 01-01 (4min), 01-02 (2min), 01-03 (5min), 02-01 (9min)
- Trend: Phase 2 plan (API migration) took longer than Phase 1 (config changes)

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

### Pending Todos

None yet.

### Blockers/Concerns

**Phase 1 (Workspace Migration):**
- ✅ RESOLVED: Workspace structure successfully migrated in 01-01
- Python binding requires dev headers (will address in Phase 2)

**Phase 2 (Python):**
- ✅ RESOLVED: PyO3 0.27 upgrade completed with Bound API migration in 02-01
- ✅ RESOLVED: pyo3-async-runtimes 0.27 added and ready for async API in 02-02

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
Stopped at: Completed 02-01-PLAN.md (PyO3 0.27 upgrade)
Resume file: .planning/phases/02-python-binding/02-01-SUMMARY.md
Next: Continue Phase 2 with Plan 02-02 (Async API) or `/gsd:plan-next`
