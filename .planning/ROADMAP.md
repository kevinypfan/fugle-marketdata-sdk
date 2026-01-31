# Roadmap: Fugle Market Data SDK

## Overview

Transform the fugle-marketdata-sdk from an extracted prototype into a production-ready, multi-language SDK with API-compatible bindings for Python, Node.js, and C#. The journey begins by modernizing build infrastructure with a Cargo workspace, then enhances Python and Node.js bindings in parallel (the lowest-risk paths), replaces the C# binding architecture with the appropriate tooling, ensures cross-platform distribution, and validates production readiness through comprehensive testing.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [x] **Phase 1: Build Infrastructure Modernization** - Unified workspace and CI/CD foundation ✓
- [x] **Phase 2: Python Binding Enhancement** - Upgrade to PyO3 0.27+ with full async support ✓
- [x] **Phase 3: Node.js Binding Enhancement** - Upgrade to napi-rs 3.6+ with TypeScript improvements ✓
- [x] **Phase 4: C# Binding Replacement** - Migrate from UniFFI to csbindgen with Task async ✓
- [x] **Phase 4.1: UniFFI Migration** - Unified FFI architecture for multi-language maintenance (INSERTED) ✓
- [x] **Phase 4.2: Java Binding via UniFFI** - Generate Java bindings using uniffi-bindgen-java with CompletableFuture async (INSERTED) ✓
- [x] **Phase 5: Cross-Platform Distribution** - Package publishing for PyPI, npm, and NuGet ✓
- [x] **Phase 6: Testing & Production Readiness** - Comprehensive API compatibility and integration testing ✓
- [ ] **Phase 7: Complete REST API Coverage** - Implement all missing REST endpoints (Historical, Snapshot, Technical, Corporate Actions)

## Phase Details

### Phase 1: Build Infrastructure Modernization
**Goal**: Establish unified build system with shared dependencies and automated CI/CD pipelines
**Depends on**: Nothing (first phase)
**Requirements**: BUILD-01, BUILD-02, BUILD-03
**Success Criteria** (what must be TRUE):
  1. Developer can build all language bindings (Python, Node.js, C#) from a single workspace root with one command
  2. CI pipeline automatically builds and tests all bindings on Linux, macOS, and Windows
  3. Version numbers sync automatically across core library and all language bindings
  4. Build artifacts are cached and reused across bindings, reducing total build time by 50%+
**Plans**: 3 plans

Plans:
- [x] 01-01-PLAN.md — Create Cargo workspace and migrate member crates ✓
- [x] 01-02-PLAN.md — Create Makefile and update version configs ✓
- [x] 01-03-PLAN.md — Create GitHub Actions CI/CD workflows ✓

### Phase 2: Python Binding Enhancement
**Goal**: Modernize Python binding to PyO3 0.27+ with native asyncio support and full API compatibility with fugle-marketdata-python
**Depends on**: Phase 1 (workspace for dependency management)
**Requirements**: PY-01, PY-02, PY-03, PY-04
**Success Criteria** (what must be TRUE):
  1. Python users can use async/await syntax with all REST and WebSocket operations without blocking the event loop
  2. Python users import the SDK and see full IDE autocomplete with type hints for all public APIs
  3. Python users can replace `import fugle_marketdata` with `import marketdata_py` in existing code without changing method calls or response handling
  4. WebSocket streaming delivers real-time data through Python iterator pattern without GIL-related deadlocks
**Plans**: 5 plans

Plans:
- [x] 02-01-PLAN.md — Upgrade PyO3 to 0.27 with async runtime foundation ✓
- [x] 02-02-PLAN.md — Convert REST client to native async/await API ✓
- [x] 02-03-PLAN.md — Add WebSocket async iterator and auto-reconnect ✓
- [x] 02-04-PLAN.md — Create type stubs and PEP 561 compliance ✓
- [x] 02-05-PLAN.md — Validate async functionality with integration tests ✓

### Phase 3: Node.js Binding Enhancement
**Goal**: Upgrade Node.js binding to napi-rs 3.6+ with improved TypeScript definitions and API compatibility with fugle-marketdata-node
**Depends on**: Phase 1 (workspace for dependency management)
**Requirements**: JS-01, JS-02, JS-03, JS-04
**Success Criteria** (what must be TRUE):
  1. Node.js users can use Promise-based async/await for all operations with automatic tokio-to-Promise bridging
  2. TypeScript users see accurate type definitions with no `any` types in public API surface
  3. Node.js users can replace `require('@fugle/marketdata')` with this SDK without changing method signatures or response structures
  4. WebSocket streaming emits events through EventEmitter pattern without memory leaks or event loop blocking
**Plans**: 4 plans

Plans:
- [x] 03-01-PLAN.md — Upgrade napi-rs to 3.6+ with ThreadsafeFunction refactoring ✓
- [x] 03-02-PLAN.md — Convert REST client to async/Promise-returning API ✓
- [x] 03-03-PLAN.md — Create comprehensive TypeScript type definitions ✓
- [x] 03-04-PLAN.md — Add API compatibility and integration tests ✓

### Phase 4: C# Binding Replacement
**Goal**: Replace UniFFI architecture with csbindgen for idiomatic .NET interop with Task-based async support
**Depends on**: Phase 2, Phase 3 (learn from async bridging patterns established in Python/Node.js)
**Requirements**: CS-01, CS-02, CS-03
**Success Criteria** (what must be TRUE):
  1. C# users can use async/await with Task-returning methods for all REST and WebSocket operations
  2. C# API follows PascalCase naming conventions and .NET patterns matching FubonNeo SDK structure
  3. WebSocket streaming delivers events through C# EventHandler pattern with background polling
  4. FFI boundaries handle Rust panics gracefully without corrupting .NET runtime
**Plans**: 5 plans

Plans:
- [x] 04-01-PLAN.md — Create cs/ crate with csbindgen and error handling foundation ✓
- [x] 04-02-PLAN.md — Implement REST client extern "C" exports with async callbacks ✓
- [x] 04-03-PLAN.md — Implement WebSocket client extern "C" exports with polling ✓
- [x] 04-04-PLAN.md — Create C# project with models, exceptions, and RestClient ✓
- [x] 04-05-PLAN.md — Create WebSocketClient wrapper and test suite ✓

### Phase 4.1: UniFFI Migration (INSERTED)
**Goal**: Migrate C# binding from csbindgen to UniFFI and add Go binding for unified multi-language FFI maintenance
**Depends on**: Phase 4 (existing bindings provide reference implementation)
**Requirements**: UNIFFI-01, UNIFFI-02, UNIFFI-03
**Success Criteria** (what must be TRUE):
  1. Single UDL file defines all SDK APIs (REST client, WebSocket client, models, errors)
  2. C# binding generated via uniffi-bindgen-cs maintains API compatibility with Phase 4 implementation
  3. Go binding generated via uniffi-bindgen-go provides new language support with consistent API
  4. Async operations work correctly across generated bindings (C# Task, Go goroutines)
  5. Python (PyO3) and Node.js (napi-rs) bindings remain unchanged
**Plans**: 6 plans

Plans:
- [x] 04.1-01-PLAN.md — Define typed UniFFI interface with proc-macro approach ✓
- [x] 04.1-02-PLAN.md — Implement REST client with typed async returns ✓
- [x] 04.1-03-PLAN.md — Implement WebSocket with foreign trait callbacks ✓
- [x] 04.1-04-PLAN.md — Generate C# bindings and create thin wrapper ✓
- [x] 04.1-05-PLAN.md — Generate Go bindings with channel wrapper ✓
- [x] 04.1-06-PLAN.md — Cleanup old cs/ crate and add tests/CI ✓

### Phase 4.2: Java Binding via UniFFI (INSERTED)
**Goal**: Generate Java bindings using uniffi-bindgen-java with idiomatic Java async patterns (CompletableFuture)
**Depends on**: Phase 4.1 (UniFFI infrastructure already established)
**Requirements**: JAVA-01, JAVA-02, JAVA-03
**Success Criteria** (what must be TRUE):
  1. Java binding generated via uniffi-bindgen-java from existing UniFFI proc-macro interface
  2. REST client methods return CompletableFuture<T> for async operations (not Kotlin coroutines)
  3. WebSocket streaming uses Java callback interface implementing WebSocketListener trait
  4. Java wrapper provides idiomatic API with proper exception handling (FugleException hierarchy)
  5. Tests validate structural correctness and integration with native library
**Plans**: 3 plans

Plans:
- [x] 04.2-01-PLAN.md — Install uniffi-bindgen-java and create Gradle project ✓
- [x] 04.2-02-PLAN.md — Create wrapper with builder pattern and exception hierarchy ✓
- [x] 04.2-03-PLAN.md — Add WebSocket wrapper, tests, and CI workflow ✓

### Phase 5: Cross-Platform Distribution
**Goal**: Automate package publishing with platform-specific builds for PyPI, npm, NuGet, and GitHub Packages (Java)
**Depends on**: Phase 4.1 (UniFFI bindings must be complete before distribution)
**Requirements**: PY-05, JS-05, CS-04, BUILD-02 (cross-platform builds)
**Success Criteria** (what must be TRUE):
  1. Python users can `pip install fugle-marketdata` on Linux (x86_64, aarch64), macOS (universal2), and Windows without requiring Rust toolchain
  2. Node.js users can `npm install @fugle/marketdata` and receive pre-built native addons for their platform without compilation
  3. C# users can install from NuGet and reference the package with native binaries bundled for Windows, Linux, and macOS
  4. Java users can add the Gradle dependency and get pre-built native libraries bundled in the JAR
  5. Automated release workflow publishes all packages (Python, Node.js, C#, Java) with synchronized version numbers on a single trigger
**Plans**: 6 plans

Plans:
- [x] 05-01-PLAN.md — Create Python wheel build workflow (maturin matrix) ✓
- [x] 05-02-PLAN.md — Create Node.js native addon build workflow (napi-rs matrix) ✓
- [x] 05-03-PLAN.md — Create UniFFI native library build workflow (C#/Go/Java) ✓
- [x] 05-04-PLAN.md — Create Python and Node.js publish workflows (trusted publishing) ✓
- [x] 05-05-PLAN.md — Create NuGet and Java publish workflows ✓
- [x] 05-06-PLAN.md — Create release coordinator and version sync check ✓

### Phase 6: Testing & Production Readiness
**Goal**: Validate API compatibility, integration correctness, and production reliability through comprehensive test coverage
**Depends on**: Phase 5 (distribution must work before validating end-to-end flows)
**Requirements**: TEST-01, TEST-02, TEST-03
**Success Criteria** (what must be TRUE):
  1. Each language binding passes compatibility tests verifying identical behavior to official SDK (method signatures, response types, error cases)
  2. Integration tests successfully execute real API calls for all endpoints (REST intraday/historical, WebSocket streaming) across all languages
  3. Unit tests cover all language-specific FFI boundaries including error handling, panic recovery, and memory safety
  4. Performance benchmarks demonstrate competitive speed compared to official SDKs (within 2x for Python, within 1.5x for Node.js)
**Plans**: 9 plans

Plans:
- [x] 06-01-PLAN.md — Create Python fixture-based response compatibility testing (VCR.py) ✓
- [x] 06-02-PLAN.md — Create Node.js fixture-based response compatibility testing (nock) ✓
- [x] 06-03-PLAN.md — Create Python and Node.js performance benchmarks with official SDK comparison ✓
- [x] 06-04-PLAN.md — Create CI workflows for testing and benchmark regression detection ✓
- [x] 06-05-PLAN.md — Create response compatibility tests for C#, Java, and Go bindings ✓
- [x] 06-06-PLAN.md — Create FFI boundary unit tests for all language bindings ✓
- [x] 06-07-PLAN.md — (Gap Closure) Record real VCR cassettes and JSON fixtures from official SDKs ✓
- [x] 06-08-PLAN.md — (Gap Closure) Record official SDK performance baselines ✓
- [x] 06-09-PLAN.md — (Gap Closure) Execute integration tests with real API key and document results ✓

### Phase 7: Complete REST API Coverage
**Goal**: Implement all missing REST API endpoints to achieve 100% API parity with official Fugle SDKs
**Depends on**: Phase 6 (testing infrastructure for validation)
**Requirements**: API-COMPLETE-01, API-COMPLETE-02, API-COMPLETE-03, API-COMPLETE-04
**Success Criteria** (what must be TRUE):
  1. All Stock endpoints implemented: Historical (candles, stats), Snapshot (quotes, movers, actives), Technical (SMA, RSI, KDJ, MACD, BB), Corporate Actions (capital-changes, dividends, listing-applicants), and batch Tickers
  2. All FutOpt endpoints implemented: Historical (candles, daily) and batch Tickers
  3. All new endpoints exposed through all 5 language bindings (Python, Node.js, C#, Java, Go)
  4. Response types match official SDK structures exactly (verified by compatibility tests)
  5. Integration tests pass for all new endpoints with real API calls
**Plans**: 9 plans

Plans:
- [ ] 07-01-PLAN.md — Stock Historical endpoints (candles, stats) in Rust core
- [ ] 07-02-PLAN.md — Stock Snapshot endpoints (quotes, movers, actives) in Rust core
- [ ] 07-03-PLAN.md — Stock Technical indicators (SMA, RSI, KDJ, MACD, BB) in Rust core
- [ ] 07-04-PLAN.md — Stock Corporate Actions (capital-changes, dividends, listing-applicants) in Rust core
- [ ] 07-05-PLAN.md — FutOpt Historical endpoints (candles, daily) in Rust core
- [ ] 07-06-PLAN.md — UniFFI bindings for C#, Java, Go (all new endpoints)
- [ ] 07-07-PLAN.md — Python bindings via PyO3 (all new endpoints)
- [ ] 07-08-PLAN.md — Node.js bindings via napi-rs (all new endpoints)
- [ ] 07-09-PLAN.md — Comprehensive compatibility and integration tests

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → 4 → 4.1 → 4.2 → 5 → 6 → 7

Note: Phases 2 and 3 can proceed in parallel after Phase 1 completes.
Note: Phase 4.1 and 4.2 are inserted phases to complete UniFFI bindings before distribution.

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Build Infrastructure | 3/3 | Complete ✓ | 2026-01-31 |
| 2. Python Binding | 5/5 | Complete ✓ | 2026-01-31 |
| 3. Node.js Binding | 4/4 | Complete ✓ | 2026-01-31 |
| 4. C# Binding | 5/5 | Complete ✓ | 2026-01-31 |
| 4.1 UniFFI Migration | 6/6 | Complete ✓ | 2026-01-31 |
| 4.2 Java Binding | 3/3 | Complete ✓ | 2026-01-31 |
| 5. Distribution | 6/6 | Complete ✓ | 2026-01-31 |
| 6. Testing | 9/9 | Complete ✓ | 2026-01-31 |
| 7. REST API Coverage | 0/9 | Not Started | - |
