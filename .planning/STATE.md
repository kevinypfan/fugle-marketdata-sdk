# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2025-01-30)

**Core value:** API-compatible drop-in replacement for official Fugle SDKs
**Current focus:** Phase 5 (Cross-Platform Distribution)

## Current Position

Phase: 5 of 7 (Cross-Platform Distribution)
Plan: 2 of 6 in current phase
Status: In progress
Last activity: 2026-01-31 - Completed 05-02-PLAN.md (Node.js Build Workflow)

Progress: [████████░░] 90% (~28 of 31 plans complete)

## Performance Metrics

**Velocity:**
- Total plans completed: 28
- Average duration: 6 min
- Total execution time: ~2.6 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 01-build-infrastructure | 3 | 11min | 4min |
| 02-python-binding | 5 | 38min | 8min |
| 03-nodejs-binding | 4 | 32min | 8min |
| 04-csharp-binding | 5 | 19min | 4min |
| 04.1-uniffi-migration | 6 | 36min | 6min |
| 04.2-java-binding | 3 | 18min | 6min |
| 05-distribution | 2 | 6min | 3min |

**Recent Trend:**
- Last 5 plans: 04.2-02 (5min), 04.2-03 (5min), 05-01 (4min), 05-02 (2min)
- Trend: Fast execution for GitHub Actions workflow plans

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
- **04.1-03:** Proc-macro with_foreign pattern (not deprecated callback interface) for WebSocketListener trait
- **04.1-03:** receive_timeout with 100ms interval enables graceful shutdown via AtomicBool signal
- **04.1-03:** Arc<AtomicBool> for connected/shutdown state (AtomicBool not Clone, Arc enables sharing)
- **04.1-04:** uniffi-bindgen-cs --library mode extracts metadata from cdylib (no UDL file)
- **04.1-04:** Post-process generated bindings with sed to change internal to public visibility
- **04.1-04:** FugleRestClient wrapper provides GetQuoteAsync/GetQuote pattern matching FubonNeo
- **04.1-04:** Multi-target project (netstandard2.0, net6.0, net8.0) with IsExternalInit polyfill
- **04.1-05:** uniffi-bindgen-go --library mode for Go bindings (same pattern as C#)
- **04.1-05:** Go channel wrapper (StreamingClient) for idiomatic message consumption with range loops
- **04.1-05:** WebSocketListener interface implemented by channelListener for callback-to-channel bridge
- **04.1-06:** Test skipping strategy: structural tests pass without native library, integration tests require it
- **04.1-06:** CI uses matrix builds with artifact sharing for cross-platform testing
- **04.1-06:** csbindgen removed from workspace (cs/ deleted, uniffi is now sole C#/Go binding approach)
- **04.2-01:** uniffi.toml configures Java package name (tw.com.fugle.marketdata.generated)
- **04.2-01:** Java 21 required for pattern matching in switch expressions (UniFFI-generated code)
- **04.2-01:** Gradle wrapper 8.5 with JNA 5.14.0 for native library access
- **04.2-02:** Unchecked exceptions (RuntimeException) for modern Java API, matching C# pattern
- **04.2-02:** Builder pattern with apiKey()/bearerToken()/sdkToken() for flexible authentication
- **04.2-02:** Dual sync/async methods: getQuote() blocks, getQuoteAsync() returns CompletableFuture
- **04.2-02:** FugleException.from() for sync, unwrap() for async exception conversion
- **04.2-03:** Dual streaming patterns: callback (WebSocketListener) and pull-based (BlockingQueue)
- **04.2-03:** Reflection-based structural tests that pass without native library
- **04.2-03:** Separate structural and integration tests with @Tag annotation
- **05-01:** manylinux 2_17 for glibc compatibility (Rust 1.64+ requires glibc 2.17 minimum)
- **05-01:** Swatinem/rust-cache with shared-key python-release for cross-job caching
- **05-01:** workflow_dispatch alongside workflow_call for manual testing before release integration
- **05-02:** Cross-compilation for Linux ARM64 on ubuntu-latest with gcc-aarch64-linux-gnu
- **05-02:** Artifact naming: bindings-{target} pattern for napi-rs targets
- **05-02:** napi prepublish generates optionalDependencies packages

### Roadmap Evolution

- Phase 4.1 inserted after Phase 4: UniFFI Migration (COMPLETE)
  - Result: Successfully consolidated binding generation for C# and Go
  - PyO3 and napi-rs preserved for Python and Node.js (more mature, language-idiomatic)
  - csbindgen removed (replaced by UniFFI for C#)
- Phase 4.2 inserted after Phase 4.1: Java Binding via UniFFI (INSERTED)
  - uniffi-bindgen-java (IronCore Labs) provides Java binding generation
  - Uses CompletableFuture instead of Kotlin coroutines for async
  - Follows same pattern as C# and Go bindings
  - Inserted before Distribution to complete all UniFFI bindings together

### Pending Todos

1. **Fix WebSocket client shutdown blocking** (uniffi) - `uniffi/src/websocket.rs`
   - `client.Close()` hangs, requires timeout workaround in Go

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
- SUPERSEDED: Replaced by Phase 4.1 UniFFI approach
- Legacy csbindgen implementation deleted

**Phase 4.1 (UniFFI Migration):**
- PHASE COMPLETE: All 6 plans executed successfully
- 04.1-01: Typed UniFFI interface with proc-macro approach (23 Record structs, flat error enum)
- 04.1-02: Typed REST client with async/sync methods (spawn_blocking for core calls)
- 04.1-03: WebSocket client with WebSocketListener foreign trait and StreamMessage delivery
- 04.1-04: C# binding generation via uniffi-bindgen-cs with FubonNeo-compatible wrapper
- 04.1-05: Go binding generation via uniffi-bindgen-go with channel wrapper
- 04.1-06: Testing and cleanup (15 C# tests, 14 Go tests, CI workflow)

**Phase 4.2 (Java Binding):**
- PHASE COMPLETE: All 3 plans executed successfully
- 04.2-01: Java binding infrastructure (uniffi-bindgen-java, Gradle 8.5, JNA 5.14.0, 152 generated files)
- 04.2-02: Idiomatic Java wrapper (builder pattern, FugleException hierarchy, dual sync/async)
- 04.2-03: WebSocket wrapper with dual streaming patterns, JUnit 5 tests, GitHub Actions CI

**Phase 5 (Distribution):**
- macOS code signing and universal2 builds require Apple Developer account configuration
- Alpine/musl builds need validation if musllinux wheels required beyond manylinux

**Phase 6 (Testing):**
- Need Fugle API staging environment credentials for integration tests
- Performance benchmarking methodology needs definition before claiming speedup

## Session Continuity

Last session: 2026-01-31
Stopped at: Completed 05-02-PLAN.md (Node.js Build Workflow)
Resume file: N/A
Next: 05-03-PLAN.md (UniFFI Build Workflow)
