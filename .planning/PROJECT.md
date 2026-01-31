# Fugle Market Data SDK

## What This Is

A unified multi-language SDK for Fugle Market Data API, built on a Rust core with production-ready bindings for Python, Node.js, C#, Java, and Go. Provides complete REST API coverage (26+ endpoints) and real-time WebSocket streaming with native async support in all languages.

## Core Value

**API-compatible drop-in replacement for official Fugle SDKs** — users can switch from the official Python/Node.js SDKs to this implementation without changing their code.

## Requirements

### Validated

- ✓ Cargo workspace setup with shared dependencies — v0.2.0
- ✓ Cross-platform build support (Linux, macOS, Windows) — v0.2.0
- ✓ CI/CD pipeline for automated builds and tests — v0.2.0
- ✓ PyO3 0.27+ with maturin 1.11+ for Python binding — v0.2.0
- ✓ Native asyncio integration for Python async/await — v0.2.0
- ✓ Python API compatibility with fugle-marketdata-python — v0.2.0
- ✓ Complete PEP 484 type hints for Python — v0.2.0
- ✓ Python wheels published to PyPI — v0.2.0
- ✓ napi-rs 3.4+ for Node.js binding — v0.2.0
- ✓ Native event loop integration for Node.js async — v0.2.0
- ✓ Node.js API compatibility with fugle-marketdata-node — v0.2.0
- ✓ Complete TypeScript type definitions — v0.2.0
- ✓ Node.js native addon published to npm — v0.2.0
- ✓ UniFFI-based C# binding with Task async — v0.2.0
- ✓ C# API compatibility with FubonNeo patterns — v0.2.0
- ✓ C# package published to NuGet — v0.2.0
- ✓ Java binding via uniffi-bindgen-java with CompletableFuture — v0.2.0
- ✓ Idiomatic Java wrapper with builder pattern — v0.2.0
- ✓ Go binding via uniffi-bindgen-go with channel wrapper — v0.2.0
- ✓ Unit tests for all language bindings — v0.2.0
- ✓ API compatibility tests against official SDKs — v0.2.0
- ✓ Integration tests with real Fugle API — v0.2.0
- ✓ Complete REST API coverage (Historical, Snapshot, Technical, Corporate) — v0.2.0

### Active

(None — fresh requirements defined in next milestone)

### Out of Scope

- Mobile SDKs (Swift/Kotlin) — focus on server-side languages first, consider for v0.3.0
- WebSocket Historical API — only REST Historical needed per API spec
- Breaking API changes — must maintain compatibility with official SDKs
- Custom API extensions — stick to official SDK feature set
- Synchronous-only APIs — async-first approach validated
- Embedded data storage — SDK should remain stateless

## Context

**Current State (v0.2.0):**
- 78,000 lines of code across Rust, Python, TypeScript, C#, Java, Go
- Tech stack: Rust core (ureq, tokio-tungstenite), PyO3, napi-rs, UniFFI
- 5 production-ready language bindings with complete API parity
- 182+ tests with real API verification
- Automated distribution to PyPI, npm, NuGet, GitHub Packages

**Architecture:**
- `core/` — Rust core library (REST + WebSocket clients)
- `py/` — Python bindings via PyO3 + maturin
- `js/` — Node.js bindings via napi-rs
- `uniffi/` — UniFFI bindings for C#, Java, Go
- `dotnet/` — C# wrapper project
- `java/` — Java wrapper with Gradle
- `go/` — Go wrapper with channel-based streaming

**Reference SDKs (for API compatibility):**
- Python: `/Users/zackfan/Project/fugle/fugle-marketdata-python/`
- Node.js: `/Users/zackfan/Project/fugle/fugle-marketdata-node/`

## Constraints

- **API Compatibility**: Must match official SDK method signatures and response structures
- **Tech Stack**: Rust core with language-specific FFI (PyO3, napi-rs, UniFFI)
- **Build Targets**: Linux (x86_64, aarch64), macOS (x64, arm64), Windows (x64)
- **Dependencies**: Minimize runtime dependencies in binding layers

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Rust as core language | Memory safety, performance, cross-platform FFI | ✓ Good |
| PyO3 for Python binding | Most mature Rust-Python FFI, native asyncio | ✓ Good |
| napi-rs for Node.js binding | Native Node.js addon with async support | ✓ Good |
| UniFFI for C#/Java/Go | Single definition, multi-language output | ✓ Good |
| Priority: Python → Node.js → C# | Risk-ordered by binding maturity | ✓ Good |
| Proc-macro UniFFI (not UDL) | Avoids duplicate type generation | ✓ Good |
| spawn_blocking for REST | Core uses blocking ureq, wrap in async | ✓ Good |
| Decimal phase numbering | Clear insertion semantics for urgent work | ✓ Good |
| GitHub Packages for Java | Avoids Maven Central GPG signing complexity | ✓ Good |

---
*Last updated: 2026-01-31 after v0.2.0 milestone*
