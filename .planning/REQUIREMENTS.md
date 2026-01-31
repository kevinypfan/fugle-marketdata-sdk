# Requirements: Fugle Market Data SDK

**Defined:** 2025-01-30
**Core Value:** API-compatible drop-in replacement for official Fugle SDKs

## v1 Requirements

Requirements for initial release. Each maps to roadmap phases.

### Build Infrastructure

- [x] **BUILD-01**: Cargo workspace setup with shared dependencies across all binding crates ✓
- [x] **BUILD-02**: Cross-platform build support (Linux, macOS, Windows) ✓
- [x] **BUILD-03**: CI/CD pipeline for automated builds and tests ✓

### Python Binding

- [x] **PY-01**: Upgrade PyO3 from 0.22 to 0.27+ with maturin 1.11+ ✓
- [x] **PY-02**: Native asyncio integration for async/await support ✓
- [x] **PY-03**: API compatibility with fugle-marketdata-python (method signatures, response types) ✓
- [x] **PY-04**: Complete PEP 484 type hints for all public APIs ✓
- [ ] **PY-05**: Publish manylinux wheels to PyPI

### Node.js Binding

- [x] **JS-01**: Upgrade napi-rs from 2.16 to 3.6+ ✓
- [x] **JS-02**: Native event loop integration for async operations ✓
- [x] **JS-03**: API compatibility with fugle-marketdata-node (method signatures, response types) ✓
- [x] **JS-04**: Complete TypeScript type definitions (.d.ts) ✓
- [ ] **JS-05**: Publish native addon to npm registry

### C# Binding

- [x] **CS-01**: Replace UniFFI with csbindgen for .NET binding generation ✓
- [x] **CS-02**: Task-based async/await support for all async operations ✓
- [x] **CS-03**: API compatibility with FubonNeo patterns (naming conventions, structure) ✓
- [ ] **CS-04**: Publish to NuGet registry

### Java Binding (INSERTED)

- [x] **JAVA-01**: Generate Java bindings via uniffi-bindgen-java with CompletableFuture ✓
- [x] **JAVA-02**: Idiomatic Java wrapper with builder pattern and exception hierarchy ✓
- [x] **JAVA-03**: Tests validate structural correctness and CI workflow ✓

### Testing

- [ ] **TEST-01**: Unit tests for each language binding (Python, Node.js, C#)
- [ ] **TEST-02**: API compatibility tests verifying against official SDK behavior
- [ ] **TEST-03**: Integration tests with real Fugle API endpoints

## v2 Requirements

Deferred to future release. Tracked but not in current roadmap.

### Performance & Optimization

- **PERF-01**: Performance benchmarks comparing to official SDKs
- **PERF-02**: Zero-copy optimizations where applicable
- **PERF-03**: Memory profiling and optimization

### Extended Platform Support

- **PLAT-01**: Swift/iOS binding via UniFFI
- **PLAT-02**: Kotlin/Android binding via UniFFI
- **PLAT-03**: Go binding

### Developer Experience

- **DX-01**: Migration guides from official SDKs
- **DX-02**: Language-specific documentation sites
- **DX-03**: Example projects for each language

## Out of Scope

Explicitly excluded. Documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| Mobile SDKs (Swift/Kotlin) | Focus on server-side languages first |
| WebSocket Historical API | Only REST Historical needed per requirements |
| Breaking API changes | Must maintain compatibility with official SDKs |
| Custom API extensions | Stick to official SDK feature set |
| Synchronous-only APIs | Async-first in 2026, per research |
| Embedded data storage | Scope creep, SDK should be stateless |
| Custom config files | Code > config files per best practices |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| BUILD-01 | Phase 1 | Complete |
| BUILD-02 | Phase 1 | Complete |
| BUILD-03 | Phase 1 | Complete |
| PY-01 | Phase 2 | Complete |
| PY-02 | Phase 2 | Complete |
| PY-03 | Phase 2 | Complete |
| PY-04 | Phase 2 | Complete |
| PY-05 | Phase 5 | Pending |
| JS-01 | Phase 3 | Complete |
| JS-02 | Phase 3 | Complete |
| JS-03 | Phase 3 | Complete |
| JS-04 | Phase 3 | Complete |
| JS-05 | Phase 5 | Pending |
| CS-01 | Phase 4/4.1 | Complete |
| CS-02 | Phase 4/4.1 | Complete |
| CS-03 | Phase 4/4.1 | Complete |
| CS-04 | Phase 5 | Pending |
| JAVA-01 | Phase 4.2 | Complete |
| JAVA-02 | Phase 4.2 | Complete |
| JAVA-03 | Phase 4.2 | Complete |
| TEST-01 | Phase 6 | Pending |
| TEST-02 | Phase 6 | Pending |
| TEST-03 | Phase 6 | Pending |

**Coverage:**
- v1 requirements: 23 total (20 original + 3 Java)
- Mapped to phases: 23
- Unmapped: 0 ✓

---
*Requirements defined: 2025-01-30*
*Last updated: 2026-01-31 after Phase 4.2 completion*
