# Project Research Summary

**Project:** Fugle MarketData SDK (Rust Core with Multi-Language Bindings)
**Domain:** Multi-language SDK for financial market data APIs
**Researched:** 2026-01-30
**Confidence:** HIGH (Python/Node.js), MEDIUM-HIGH (C#)

## Executive Summary

This project builds multi-language SDK bindings (Python, Node.js, C#) from a shared Rust core for accessing Fugle's financial market data APIs. The architecture follows a proven "shared computation, language-specific interface" pattern, where 80-90% of business logic resides in Rust with idiomatic FFI layers for each target language. Research indicates this is a mature, production-ready approach in 2026.

**Recommended stack:** PyO3/maturin for Python (upgrade from 0.22 to 0.27), napi-rs v3 for Node.js (upgrade from 2.16), and **csbindgen** for C# (replacing current UniFFI which targets mobile, not .NET). This combination provides optimal ergonomics and performance while maintaining language idiomaticity. The Python binding is production-ready, Node.js needs upgrade for better TypeScript support, and C# requires architectural rework.

**Key risks:** (1) Async runtime deadlocks between Tokio and language event loops (Python GIL, Node.js single-thread), (2) API compatibility breaks with existing official SDKs, and (3) cross-platform binary distribution failures (manylinux, macOS universal2, Windows MSVC). All are mitigable with established patterns from PyO3/napi-rs ecosystems.

## Key Findings

### Recommended Stack

The ecosystem for Rust-to-language bindings has matured significantly by 2026. PyO3 (v0.27) and maturin (v1.11) are industry standards for Python, with excellent async/await integration and zero-configuration wheel building. napi-rs v3.6 provides first-class Node.js support with automatic Promise bridging and superior TypeScript definitions compared to v2. For C#, research reveals UniFFI is optimized for mobile platforms (Kotlin/Swift), not .NET; csbindgen (v1.9) is purpose-built for Rust-to-C# interop.

**Core technologies:**
- **PyO3 0.27 + maturin 1.11**: Python bindings — industry standard, abi3-py38 for multi-version wheels, mature async runtime bridging
- **napi-rs 3.6 + @napi-rs/cli 2.18**: Node.js bindings — best TypeScript support, automatic tokio-to-Promise conversion, production-proven (SWC, Prisma)
- **csbindgen 1.9**: C# bindings — idiomatic .NET P/Invoke generation, simpler than UniFFI for desktop/.NET scenarios
- **cross 0.2.5 + just 1.40**: Build orchestration — Docker-based cross-compilation, simple task runner for multi-language builds
- **Tokio runtime**: Shared async execution across all bindings, proven FFI boundary patterns

**Version upgrades required:**
- Python: 0.22 → 0.27 (low risk, incremental changes)
- Node.js: 2.16 → 3.6 (medium risk, ThreadsafeFunction API changed but improved ergonomics)
- C#: UniFFI → csbindgen (high effort, requires API redesign from UDL to C FFI)

### Expected Features

Multi-language SDKs in 2026 require language-idiomatic design over one-size-fits-all approaches. Type safety, async patterns, and error handling must feel native to each language while maintaining functional parity.

**Must have (table stakes):**
- Language-idiomatic API design (snake_case Python, camelCase Node.js, PascalCase C#)
- Comprehensive error handling with rich context (HTTP status, URLs, retry-ability classification)
- Native async/await support (asyncio, Promise, Task) — blocking I/O unacceptable
- Type safety with IDE completion (type hints, TypeScript .d.ts, strong C# typing)
- Dual client pattern (REST + WebSocket with shared auth)
- Authentication flexibility (API key, bearer token, SDK token with mutual exclusion)
- Connection management (pooling, retries with exponential backoff, configurable timeouts)
- Semantic versioning with deprecation warnings before breaking changes

**Should have (competitive):**
- Zero-copy performance via Rust core (10-100x faster than pure Python/Node.js)
- Smart retry with circuit breaker pattern (prevent cascading failures during outages)
- Type-safe request builders (compile-time validation, fluent API)
- Comprehensive logging and distributed tracing (OpenTelemetry integration)
- Rate limit handling (automatic 429 detection, Retry-After parsing)
- Migration utilities from official SDKs (compatibility layer, deprecation warnings)

**Defer (v2+):**
- Streaming data aggregation (rolling windows, VWAP calculation from ticks)
- Offline mode with request recording (development without live API)
- Built-in backtesting framework (separate library responsibility)
- Embedded data storage (users have their own storage solutions)

### Architecture Approach

The architecture follows a three-layer separation: (1) Core library in pure Rust with business logic and tokio async operations, (2) FFI bridge layer handling type conversion and async runtime bridging, (3) Language layer providing idiomatic APIs and documentation. This ensures core logic remains testable and maintainable while FFI boundaries manage impedance mismatch.

**Major components:**
1. **marketdata-core** — Pure Rust library with REST/WebSocket clients, authentication, error handling. No FFI concerns, 100% safe Rust.
2. **FFI Bridge Layer (py/, js/, cs/)** — Type conversion (Rust ↔ Python/JS/C#), async runtime bridging (tokio ↔ asyncio/Promise/Task), error mapping, lifetime management with Arc/Mutex.
3. **Language Layer** — Idiomatic wrappers, property decorators, EventEmitter patterns, comprehensive documentation with language-specific examples.

**Critical architectural decisions:**
- **Async bridging:** Use pyo3-async-runtimes for Python (GIL management), napi-rs automatic async fn → Promise conversion, C# Task.Run() wrapper over sync FFI
- **WebSocket callbacks:** Python iterator pattern, Node.js EventEmitter with ThreadsafeFunction, C# event handlers with background polling
- **FFI boundary:** JSON serialization for complex types (performance vs flexibility trade-off), opaque handles (Arc) for stateful objects, Result → exception mapping

**Build organization:** Migrate to Cargo workspace with shared dependencies and single Cargo.lock. Currently each binding builds independently causing version drift and no artifact reuse.

### Critical Pitfalls

Based on multi-language SDK research, these pitfalls cause production incidents:

1. **Panic unwinding across FFI boundaries** — Rust panics escape causing process crashes or corrupted Python/Node.js runtime. Prevention: Use std::panic::catch_unwind at all FFI entry points; PyO3/napi-rs handle this automatically but verify custom FFI code. Critical for Phase 1/2/3 during FFI boundary audits.

2. **Async runtime deadlocks with GIL and event loops** — Python GIL acquisition deadlocks with Tokio, Node.js event loop blocks on sync Rust operations. Prevention: Never call block_on() from FFI; use pyo3-async-runtimes for Python, napi-rs async fn for Node.js, release GIL before blocking operations. Critical for Phase 1 (Python WebSocket) and Phase 2 (Node.js).

3. **Breaking API compatibility subtly** — Type mismatches (JavaScript number loses i64 precision), error behavior changes, async/sync signature differences. Prevention: Generate compatibility test suite from official SDK tests, use property-based testing for edge cases, document all deviations. Critical for all phases — compare against official SDK test suites.

4. **Cross-platform binary distribution failures** — manylinux compliance issues, macOS universal2 builds, Windows MSVC dependencies. Prevention: Use manylinux2014 containers for Python, cross-compilation for Node.js targets, test on minimal Docker images. Critical for Phase 4 (distribution setup).

5. **Memory safety violations at FFI boundaries** — Use-after-free, dangling pointers, circular references between Rust and language GC. Prevention: Use Arc for shared ownership, Mutex for mutable state, document ownership transfer, avoid raw pointers. Critical for Phase 2 (Node.js Buffer/TypedArray) and Phase 3 (C# interop).

## Implications for Roadmap

Based on research findings, suggested phase structure prioritizes language-specific migrations with cross-cutting concerns:

### Phase 1: Python Binding Modernization
**Rationale:** Python binding is most mature (already on PyO3), upgrade path is lowest risk, and Python has largest user base in financial data domain. Establishes patterns for other bindings.

**Delivers:**
- PyO3 0.22 → 0.27 upgrade
- Full async/await support with pyo3-async-runtimes
- WebSocket iterator pattern for streaming data
- Type hints (.pyi stubs) for all public APIs
- Compatibility testing against fugle-marketdata-python

**Addresses:**
- Table stakes: Language-idiomatic async/await, type safety
- Architecture: Async runtime bridging patterns for Python GIL

**Avoids:**
- Pitfall #2: GIL deadlocks (use pyo3-asyncio, never block_on)
- Pitfall #3: API compatibility (test against official SDK)

**Research flags:** LOW — PyO3 patterns well-documented, upgrade incremental

### Phase 2: Node.js Binding Enhancement
**Rationale:** Node.js requires napi-rs v2 → v3 upgrade for better TypeScript support. Can proceed in parallel with Python but benefits from async bridging patterns established in Phase 1.

**Delivers:**
- napi-rs 2.16 → 3.6 upgrade (breaking changes in ThreadsafeFunction API)
- Improved TypeScript definitions (not `any` types)
- EventEmitter pattern for WebSocket events
- Memory leak testing with proper cleanup
- Compatibility testing against fugle-marketdata-node

**Addresses:**
- Table stakes: Type safety, native async patterns
- Differentiators: Zero-copy performance via Rust core

**Avoids:**
- Pitfall #2: Event loop blocking (always use async fn)
- Pitfall #3: Type precision loss (use string for i64 > 2^53)
- Pitfall #5: Memory leaks with Buffer/TypedArray

**Research flags:** MEDIUM — napi-rs v3 ThreadsafeFunction changes need testing, memory management patterns need validation

### Phase 3: C# Binding Replacement
**Rationale:** Current UniFFI approach inappropriate for .NET (targets mobile). Requires architectural rework to csbindgen. Sequential after Python/Node.js to leverage established patterns.

**Delivers:**
- Replace UniFFI with csbindgen 1.9
- Refactor API from UDL to extern "C" FFI
- C# async wrapper (Task.Run pattern over sync FFI)
- Event pattern for WebSocket (EventHandler + background polling)
- NuGet packaging workflow

**Addresses:**
- Stack: Replace wrong tool (UniFFI) with .NET-appropriate tool (csbindgen)
- Architecture: C# Task bridging over tokio futures

**Avoids:**
- Pitfall #1: Panic handling at custom FFI boundaries (csbindgen is lower-level than PyO3/napi-rs)
- Pitfall #3: API compatibility with FubonNeo patterns

**Research flags:** MEDIUM-HIGH — C# ecosystem less mature, csbindgen patterns need validation, async bridging requires design doc

### Phase 4: Workspace Migration & Build System
**Rationale:** Cross-cutting concern that unblocks parallel Phase 1/2 work. Establishes shared dependencies and build optimization before distribution phase.

**Delivers:**
- Cargo workspace with shared dependencies
- Unified version management
- justfile for task orchestration (build-all, test-all, release-all)
- CI matrix builds for all platforms (GitHub Actions)

**Addresses:**
- Architecture: Build organization, version synchronization
- Pitfall #9: Version skew between core and bindings

**Avoids:**
- Pitfall #10: Build time optimization neglected (shared target/, sccache)

**Research flags:** LOW — Cargo workspaces well-documented, standard practice

### Phase 5: Cross-Platform Distribution
**Rationale:** Depends on Phases 1-3 completing binding implementations. Critical for production deployment.

**Delivers:**
- manylinux2014 wheel builds for Python (x86_64, aarch64)
- Cross-compilation for Node.js (macOS universal2, Linux, Windows)
- NuGet packaging for C#
- Automated release workflow (maturin publish, npm publish, nuget push)
- Platform testing matrix (CentOS 7, Alpine, macOS M1, Windows 11)

**Addresses:**
- Pitfall #4: Cross-platform distribution failures (manylinux compliance, macOS code signing, Windows DLL deps)

**Avoids:**
- "Works on my machine" deployments
- PyPI rejection due to platform tags

**Research flags:** MEDIUM — Platform-specific issues require testing on real hardware (GitHub Actions provides matrix)

### Phase 6: Testing & Documentation
**Rationale:** Cross-cutting concern that should start in Phase 1 but intensifies after core implementations complete. Ensures production readiness.

**Delivers:**
- Compatibility test suite generated from official SDKs
- Property-based testing for edge cases (Hypothesis, fast-check)
- Integration tests against real API (staging environment)
- Language-specific documentation (pydoc, JSDoc, XML docs)
- Migration guides from official SDKs
- Performance benchmarks (Rust vs pure Python/Node.js)

**Addresses:**
- Pitfall #7: Test coverage gaps (network errors, rate limiting, concurrent access)
- Pitfall #8: Documentation mismatches between languages

**Avoids:**
- Production incidents from untested edge cases
- User confusion from copy-pasted docs

**Research flags:** LOW — Testing strategies well-established, need execution discipline

### Phase Ordering Rationale

- **Sequential core:** Phase 4 (workspace) must complete before Phases 1-3 for dependency management
- **Parallel bindings:** Phases 1 (Python) and 2 (Node.js) can proceed concurrently after Phase 4
- **Sequential C#:** Phase 3 depends on Phases 1/2 establishing async bridging patterns (higher risk, benefits from precedent)
- **Distribution last:** Phase 5 requires Phases 1-3 complete (no point distributing incomplete bindings)
- **Testing throughout:** Phase 6 starts in Phase 1 but intensifies after implementations stabilize

**Dependency graph:**
```
Phase 4 (Workspace) → Phase 1 (Python) ──┐
                   → Phase 2 (Node.js) ──┤→ Phase 5 (Distribution) → Phase 6 (Testing)
                   → Phase 3 (C#) ────────┘
```

**Critical path:** Phase 4 (1 week) → Phase 3 (3 weeks, longest) → Phase 5 (1 week) = ~5 weeks minimum
**Parallelization:** Phases 1 and 2 (2 weeks each) can overlap, saving 2 weeks if resources available

### Research Flags

**Phases likely needing deeper research during planning:**
- **Phase 3 (C# binding):** csbindgen patterns sparse, async bridging design needed, NuGet packaging workflow unclear
- **Phase 5 (Distribution):** Platform-specific quirks (macOS code signing, Alpine musl builds) need hands-on testing

**Phases with standard patterns (skip research-phase):**
- **Phase 1 (Python):** PyO3 patterns mature, upgrade path incremental, official docs comprehensive
- **Phase 2 (Node.js):** napi-rs v3 documented, ThreadsafeFunction changes known, migration path clear
- **Phase 4 (Workspace):** Cargo workspaces standard practice, just vs cargo-make decision straightforward

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | PyO3/napi-rs versions verified via official docs (Jan 2026), production-proven |
| Features | HIGH | Based on official SDK analysis + Azure/AWS SDK patterns, direct code inspection |
| Architecture | HIGH | PyO3/napi-rs async patterns documented, real-world examples (Temporal, Spikard) |
| Pitfalls | HIGH | PyO3/napi-rs GitHub issues + RFCs, verified with 2025-2026 sources |
| C# Stack | MEDIUM-HIGH | csbindgen verified via NuGet (v1.9.7), less ecosystem maturity than PyO3/napi-rs |
| C# Patterns | MEDIUM | Fewer examples, async bridging requires validation, UniFFI-to-csbindgen migration uncharted |

**Overall confidence:** HIGH for Python/Node.js paths (95%+ certainty on approach), MEDIUM-HIGH for C# path (80% certainty, needs validation)

### Gaps to Address

Research identified areas requiring resolution during planning/execution:

- **C# async bridging design:** Multiple strategies possible (block_on, spawn+poll, Task.Run wrapper). Recommend prototyping all three in Phase 3 to measure performance/ergonomics trade-offs. Task.Run likely winner but needs validation.

- **Node.js ThreadsafeFunction migration:** napi-rs v2 → v3 API changed. Need careful audit of callback handling in WebSocket client. BREAKING CHANGE but improved safety. Allocate time for refactoring.

- **Cross-platform testing hardware:** GitHub Actions provides x86_64 matrix but ARM64 (Apple Silicon, AWS Graviton) requires self-hosted runners or paid CI. Budget consideration for Phase 5.

- **Performance benchmarking methodology:** "10-100x faster" claim from research needs validation with real workload. Design benchmark suite in Phase 1 comparing Rust-backed SDK vs pure Python for message throughput, parsing speed, memory usage.

- **manylinux compliance for Alpine:** Research shows musllinux support (PEP 656) but less mature than glibc manylinux. If Alpine support required, add musllinux build target in Phase 5 (medium effort).

- **C# NuGet packaging specifics:** Research didn't cover .csproj setup for packaging native library + C# wrapper. Will need .NET-specific research during Phase 3 or consult NuGet packaging docs.

## Sources

### Primary (HIGH confidence)
- [PyO3 Changelog v0.27.2](https://pyo3.rs/main/changelog.html) — Latest stable release features
- [Maturin User Guide](https://www.maturin.rs/) — Python wheel building
- [napi-rs Documentation](https://napi.rs/) — Node.js bindings framework
- [napi-rs v3 Announcement](https://napi.rs/blog/announce-v3) — Breaking changes and improvements
- [csbindgen GitHub](https://github.com/Cysharp/csbindgen) — C# FFI code generation
- [Building Great SDKs - Pragmatic Engineer](https://newsletter.pragmaticengineer.com/p/building-great-sdks) — SDK design principles
- [Azure SDK Design Guidelines](https://azure.github.io/azure-sdk/python_design.html) — Language-idiomatic patterns
- [AWS SDK Best Practices](https://docs.aws.amazon.com/sdk-for-java/latest/developer-guide/best-practices.html) — Connection management, retry logic
- [PyO3 GIL Deadlock Discussion](https://github.com/PyO3/pyo3/discussions/3045) — Async runtime conflicts
- [Rust FFI Unwind RFC](https://rust-lang.github.io/rfcs/2797-project-ffi-unwind.html) — Panic handling across FFI

### Secondary (MEDIUM confidence)
- [csbindgen NuGet v1.9.7](https://www.nuget.org/packages/csbindgen) — Version verification
- [Temporal Rust Core SDK](https://www.infoq.com/news/2025/11/temporal-rust-polygot-sdk/) — Multi-language architecture example
- [Stainless SDK Error Handling Patterns](https://www.stainless.com/sdk-api-best-practices/standard-error-handling-patterns-in-sdks-across-languages) — Cross-language error conventions
- [manylinux PEP 513](https://peps.python.org/pep-0513/) — Python binary distribution standards
- [Effective Rust: Control FFI Boundaries](https://www.effective-rust.com/ffi.html) — Performance optimization

### Tertiary (LOW confidence)
- [Modern API Design Best Practices 2026](https://www.xano.com/blog/modern-api-design-best-practices/) — General trends, not Rust-specific
- [Code Documentation Best Practices 2026](https://www.qodo.ai/blog/code-documentation-best-practices-2026/) — Documentation patterns
- Property-based testing strategies — inferred from Hypothesis/fast-check docs, not SDK-specific

---
*Research completed: 2026-01-30*
*Ready for roadmap: yes*
