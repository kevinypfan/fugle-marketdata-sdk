# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2025-01-30)

**Core value:** API-compatible drop-in replacement for official Fugle SDKs
**Current focus:** Phase 4 Complete - Ready for Phase 4.1 (UniFFI Migration)

## Current Position

Phase: 4.1 of 7 (UniFFI Migration) - IN PROGRESS
Plan: 2 of 6 in current phase - COMPLETE
Status: Plan 04.1-02 complete - Typed REST client with async support
Last activity: 2026-01-31 - Completed 04.1-02-PLAN.md (typed REST client implementation)

Progress: [███████░░░] 70% (~19 of 27 plans complete)

## Performance Metrics

**Velocity:**
- Total plans completed: 17
- Average duration: 6 min
- Total execution time: ~1.8 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 01-build-infrastructure | 3 | 11min | 4min |
| 02-python-binding | 5 | 38min | 8min |
| 03-nodejs-binding | 4 | 32min | 8min |
| 04-csharp-binding | 5 | 19min | 4min |

**Recent Trend:**
- Last 5 plans: 04-01 (2min), 04-03 (3min), 04-02 (4min), 04-04 (6min), 04-05 (4min)
- Trend: Phase 4 consistently fast (4min avg) due to patterns from Python/Node.js

*Updated after each plan completion*

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.
Recent decisions affecting current work:

- Priority order: Python -> Node.js -> C# (Python most mature, C# needs architectural rework)
- C# requires csbindgen (not UniFFI) - UniFFI targets mobile platforms, csbindgen is .NET-appropriate
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
- **03-04:** Use isPromiseLike() helper for napi-rs Promise detection (cross-runtime compatibility)
- **03-04:** Integration tests use describe.skip pattern for CI-friendly conditional execution
- **04-01:** csbindgen (not UniFFI) for .NET-specific FFI generation with extern "C" approach
- **04-01:** Error codes use negative integers (SUCCESS=0, errors=-1 to -999) for C-style FFI
- **04-01:** catch_unwind wraps all FFI boundaries to prevent process abort on panic
- **04-01:** Global tokio RUNTIME (Lazy<Runtime>) for async operation bridging
- **04-02:** Callback pattern for async REST endpoints with (user_data, result_json, error_code)
- **04-02:** Convert callback/user_data pointers to usize for Send trait compatibility across async boundaries
- **04-02:** JSON serialization for all REST responses via serde_json
- **04-03:** Single generic subscribe/unsubscribe API for both stock and futopt (endpoint type selected at connect time)
- **04-03:** Message polling with MESSAGE_AVAILABLE/NO_MESSAGE status codes for non-blocking C# consumption
- **04-03:** State codes as c_int constants (DISCONNECTED=0, CONNECTING=1, CONNECTED=2, RECONNECTING=3)
- **04-03:** Tokio spawn task forwards messages from core MessageReceiver to mpsc::channel for C# polling
- **04-04:** Record types (not classes) for C# model immutability with JsonPropertyName attributes
- **04-04:** IsExternalInit polyfill enables records in netstandard2.0 target
- **04-04:** Dual async/sync method pattern: async primary, sync as GetAwaiter().GetResult() wrapper
- **04-05:** EventHandler<T> pattern (not callbacks) for .NET-idiomatic streaming
- **04-05:** 10ms polling interval for low latency message delivery
- **04-05:** Method-level unsafe (not class-level) for async/await compatibility
- **04-05:** Assert.Inconclusive for graceful skip when native library unavailable
- **04.1-01:** Proc-macro-only approach (uniffi::setup_scaffolding!) instead of UDL file to avoid duplicate type generation
- **04.1-01:** Constructors in separate non-exported impl blocks (UniFFI doesn't support associated functions in exports)
- **04.1-01:** tokio feature required for uniffi dependency when using async_runtime = "tokio"
- **04.1-02:** spawn_blocking wraps all core REST calls (core uses blocking ureq)
- **04.1-02:** Dual async/sync methods: get_quote() async, quote_sync() blocking for simple use cases
- **04.1-02:** tokio feature required in build-dependencies for async_compat module in proc-macro generation

### Roadmap Evolution

- Phase 4.1 inserted after Phase 4: UniFFI Migration (URGENT)
  - Rationale: Consolidate language-specific FFI (PyO3, napi-rs, csbindgen) into unified UniFFI architecture
  - Benefits: Single UDL source, easier Go/Swift support, reduced maintenance burden
  - Trade-off: Requires rewriting existing bindings, may lose some language-idiomatic optimizations

### Pending Todos

None yet.

### Blockers/Concerns

**Phase 1 (Workspace Migration):**
- COMPLETE: All plans executed successfully

**Phase 2 (Python):**
- COMPLETE: All plans executed successfully
- PENDING: Historical/snapshot endpoints blocked until core implementation available

**Phase 3 (Node.js):**
- COMPLETE: All 4 plans executed successfully
- 03-01: napi-rs 3.4 upgrade with Arc<ThreadsafeFunction>
- 03-02: Async REST methods with spawn_blocking
- 03-03: TypeScript type definitions (813 lines, no `any` types)
- 03-04: Jest test suite (45 structural + 15 conditional tests)
- PENDING: Memory leak testing for Buffer/TypedArray handling (deferred to Phase 6)

**Phase 4 (C#):**
- PHASE COMPLETE: All 5 plans executed successfully
- 04-01: csbindgen FFI foundation with error codes and panic recovery
- 04-02: REST client FFI with 8 async callback endpoints (stock/futopt intraday)
- 04-03: WebSocket FFI with polling-based message retrieval and generic subscription API
- 04-04: C# wrapper layer (RestClient with Task-based async, exception hierarchy, record models)
- 04-05: WebSocketClient with EventHandler streaming + MSTest suite (56 tests)
- Send trait issue resolved via usize pointer conversion pattern
- IsExternalInit polyfill enables records in netstandard2.0
- Method-level unsafe enables async/await in disposal patterns

**Phase 4.1 (UniFFI Migration):**
- IN PROGRESS: Plan 2 of 6 complete
- 04.1-01: Typed UniFFI interface with proc-macro approach (23 Record structs, flat error enum)
- 04.1-02: Typed REST client with async/sync methods (spawn_blocking for core calls)
- Deviation: Split impl blocks required (UniFFI doesn't support associated functions in exports)
- PENDING: Historical data (04.1-03), WebSocket (04.1-04), C# generation (04.1-05), testing (04.1-06)

**Phase 5 (Distribution):**
- macOS code signing and universal2 builds require Apple Developer account configuration
- Alpine/musl builds need validation if musllinux wheels required beyond manylinux

**Phase 6 (Testing):**
- Need Fugle API staging environment credentials for integration tests
- Performance benchmarking methodology needs definition before claiming speedup

## Session Continuity

Last session: 2026-01-31
Stopped at: Completed 04.1-02-PLAN.md (typed REST client with async support)
Resume file: N/A
Next: Phase 4.1 Plan 03 - Historical data methods
