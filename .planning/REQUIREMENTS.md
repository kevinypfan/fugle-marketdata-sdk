# Requirements: Fugle Market Data SDK

**Defined:** 2025-01-30
**Core Value:** API-compatible drop-in replacement for official Fugle SDKs

## v1 Requirements

Requirements for initial release. Each maps to roadmap phases.

### Build Infrastructure

- [ ] **BUILD-01**: Cargo workspace setup with shared dependencies across all binding crates
- [ ] **BUILD-02**: Cross-platform build support (Linux, macOS, Windows)
- [ ] **BUILD-03**: CI/CD pipeline for automated builds and tests

### Python Binding

- [ ] **PY-01**: Upgrade PyO3 from 0.22 to 0.27+ with maturin 1.11+
- [ ] **PY-02**: Native asyncio integration for async/await support
- [ ] **PY-03**: API compatibility with fugle-marketdata-python (method signatures, response types)
- [ ] **PY-04**: Complete PEP 484 type hints for all public APIs
- [ ] **PY-05**: Publish manylinux wheels to PyPI

### Node.js Binding

- [ ] **JS-01**: Upgrade napi-rs from 2.16 to 3.6+
- [ ] **JS-02**: Native event loop integration for async operations
- [ ] **JS-03**: API compatibility with fugle-marketdata-node (method signatures, response types)
- [ ] **JS-04**: Complete TypeScript type definitions (.d.ts)
- [ ] **JS-05**: Publish native addon to npm registry

### C# Binding

- [ ] **CS-01**: Replace UniFFI with csbindgen for .NET binding generation
- [ ] **CS-02**: Task-based async/await support for all async operations
- [ ] **CS-03**: API compatibility with FubonNeo patterns (naming conventions, structure)
- [ ] **CS-04**: Publish to NuGet registry

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
| BUILD-01 | TBD | Pending |
| BUILD-02 | TBD | Pending |
| BUILD-03 | TBD | Pending |
| PY-01 | TBD | Pending |
| PY-02 | TBD | Pending |
| PY-03 | TBD | Pending |
| PY-04 | TBD | Pending |
| PY-05 | TBD | Pending |
| JS-01 | TBD | Pending |
| JS-02 | TBD | Pending |
| JS-03 | TBD | Pending |
| JS-04 | TBD | Pending |
| JS-05 | TBD | Pending |
| CS-01 | TBD | Pending |
| CS-02 | TBD | Pending |
| CS-03 | TBD | Pending |
| CS-04 | TBD | Pending |
| TEST-01 | TBD | Pending |
| TEST-02 | TBD | Pending |
| TEST-03 | TBD | Pending |

**Coverage:**
- v1 requirements: 20 total
- Mapped to phases: 0
- Unmapped: 20 ⚠️

---
*Requirements defined: 2025-01-30*
*Last updated: 2025-01-30 after initial definition*
