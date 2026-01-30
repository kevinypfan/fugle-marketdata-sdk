# Roadmap: Fugle Market Data SDK

## Overview

Transform the fugle-marketdata-sdk from an extracted prototype into a production-ready, multi-language SDK with API-compatible bindings for Python, Node.js, and C#. The journey begins by modernizing build infrastructure with a Cargo workspace, then enhances Python and Node.js bindings in parallel (the lowest-risk paths), replaces the C# binding architecture with the appropriate tooling, ensures cross-platform distribution, and validates production readiness through comprehensive testing.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [ ] **Phase 1: Build Infrastructure Modernization** - Unified workspace and CI/CD foundation
- [ ] **Phase 2: Python Binding Enhancement** - Upgrade to PyO3 0.27+ with full async support
- [ ] **Phase 3: Node.js Binding Enhancement** - Upgrade to napi-rs 3.6+ with TypeScript improvements
- [ ] **Phase 4: C# Binding Replacement** - Migrate from UniFFI to csbindgen with Task async
- [ ] **Phase 5: Cross-Platform Distribution** - Package publishing for PyPI, npm, and NuGet
- [ ] **Phase 6: Testing & Production Readiness** - Comprehensive API compatibility and integration testing

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
- [ ] 01-01-PLAN.md — Create Cargo workspace and migrate member crates
- [ ] 01-02-PLAN.md — Create Makefile and update version configs
- [ ] 01-03-PLAN.md — Create GitHub Actions CI/CD workflows

### Phase 2: Python Binding Enhancement
**Goal**: Modernize Python binding to PyO3 0.27+ with native asyncio support and full API compatibility with fugle-marketdata-python
**Depends on**: Phase 1 (workspace for dependency management)
**Requirements**: PY-01, PY-02, PY-03, PY-04
**Success Criteria** (what must be TRUE):
  1. Python users can use async/await syntax with all REST and WebSocket operations without blocking the event loop
  2. Python users import the SDK and see full IDE autocomplete with type hints for all public APIs
  3. Python users can replace `import fugle_marketdata` with `import marketdata_py` in existing code without changing method calls or response handling
  4. WebSocket streaming delivers real-time data through Python iterator pattern without GIL-related deadlocks
**Plans**: TBD

Plans:
- [ ] 02-01: TBD during planning

### Phase 3: Node.js Binding Enhancement
**Goal**: Upgrade Node.js binding to napi-rs 3.6+ with improved TypeScript definitions and API compatibility with fugle-marketdata-node
**Depends on**: Phase 1 (workspace for dependency management)
**Requirements**: JS-01, JS-02, JS-03, JS-04
**Success Criteria** (what must be TRUE):
  1. Node.js users can use Promise-based async/await for all operations with automatic tokio-to-Promise bridging
  2. TypeScript users see accurate type definitions with no `any` types in public API surface
  3. Node.js users can replace `require('@fugle/marketdata')` with this SDK without changing method signatures or response structures
  4. WebSocket streaming emits events through EventEmitter pattern without memory leaks or event loop blocking
**Plans**: TBD

Plans:
- [ ] 03-01: TBD during planning

### Phase 4: C# Binding Replacement
**Goal**: Replace UniFFI architecture with csbindgen for idiomatic .NET interop with Task-based async support
**Depends on**: Phase 2, Phase 3 (learn from async bridging patterns established in Python/Node.js)
**Requirements**: CS-01, CS-02, CS-03
**Success Criteria** (what must be TRUE):
  1. C# users can use async/await with Task-returning methods for all REST and WebSocket operations
  2. C# API follows PascalCase naming conventions and .NET patterns matching FubonNeo SDK structure
  3. WebSocket streaming delivers events through C# EventHandler pattern with background polling
  4. FFI boundaries handle Rust panics gracefully without corrupting .NET runtime
**Plans**: TBD

Plans:
- [ ] 04-01: TBD during planning

### Phase 5: Cross-Platform Distribution
**Goal**: Automate package publishing with platform-specific builds for PyPI, npm, and NuGet registries
**Depends on**: Phase 2, Phase 3, Phase 4 (all bindings must be complete before distribution)
**Requirements**: PY-05, JS-05, CS-04, BUILD-02 (cross-platform builds)
**Success Criteria** (what must be TRUE):
  1. Python users can `pip install fugle-marketdata` on Linux (x86_64, aarch64), macOS (universal2), and Windows without requiring Rust toolchain
  2. Node.js users can `npm install @fugle/marketdata` and receive pre-built native addons for their platform without compilation
  3. C# users can install from NuGet and reference the package with native binaries bundled for Windows, Linux, and macOS
  4. Automated release workflow publishes all three packages with synchronized version numbers on a single trigger
**Plans**: TBD

Plans:
- [ ] 05-01: TBD during planning

### Phase 6: Testing & Production Readiness
**Goal**: Validate API compatibility, integration correctness, and production reliability through comprehensive test coverage
**Depends on**: Phase 5 (distribution must work before validating end-to-end flows)
**Requirements**: TEST-01, TEST-02, TEST-03
**Success Criteria** (what must be TRUE):
  1. Each language binding passes compatibility tests verifying identical behavior to official SDK (method signatures, response types, error cases)
  2. Integration tests successfully execute real API calls for all endpoints (REST intraday/historical, WebSocket streaming) across all languages
  3. Unit tests cover all language-specific FFI boundaries including error handling, panic recovery, and memory safety
  4. Performance benchmarks demonstrate competitive speed compared to official SDKs (within 2x for Python, within 1.5x for Node.js)
**Plans**: TBD

Plans:
- [ ] 06-01: TBD during planning

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → 4 → 5 → 6

Note: Phases 2 and 3 can proceed in parallel after Phase 1 completes.

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Build Infrastructure | 0/3 | Ready to execute | - |
| 2. Python Binding | 0/TBD | Not started | - |
| 3. Node.js Binding | 0/TBD | Not started | - |
| 4. C# Binding | 0/TBD | Not started | - |
| 5. Distribution | 0/TBD | Not started | - |
| 6. Testing | 0/TBD | Not started | - |
