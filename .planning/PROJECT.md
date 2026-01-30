# Fugle Market Data SDK

## What This Is

A unified multi-language SDK for Fugle Market Data API, built on a Rust core with bindings for Python, Node.js, and C#. This project extracts and generalizes the SDK from the fubon-sdk-core project, replacing language-specific implementations with a single Rust codebase that provides consistent behavior and API compatibility across all target languages.

## Core Value

**API-compatible drop-in replacement for official Fugle SDKs** — users can switch from the official Python/Node.js SDKs to this implementation without changing their code.

## Requirements

### Validated

- ✓ Rust core REST client implementation — existing
- ✓ Rust core WebSocket client implementation — existing
- ✓ Python binding via PyO3/maturin — existing, tested
- ✓ Node.js binding via napi-rs — existing, untested

### Active

- [ ] Python API compatibility with fugle-marketdata-python
- [ ] Python test coverage for all endpoints
- [ ] Node.js API compatibility with fugle-marketdata-node
- [ ] Node.js test coverage for all endpoints
- [ ] C# binding via UniFFI
- [ ] C# API compatibility with FubonNeo patterns
- [ ] C# test coverage for all endpoints
- [ ] PyPI package publishing setup
- [ ] npm package publishing setup
- [ ] NuGet package publishing setup

### Out of Scope

- Mobile SDKs (Swift/Kotlin) — focus on server-side languages first
- WebSocket Historical API — only REST Historical needed
- Breaking API changes — must maintain compatibility with official SDKs
- Custom API extensions — stick to official SDK feature set

## Context

**Origin:** This project was extracted from `/Users/zackfan/Project/fubon/sdk-core` to create a standalone, multi-language SDK package.

**Reference SDKs (for API compatibility validation):**
- Python: `/Users/zackfan/Project/fugle/fugle-marketdata-python/`
- Node.js: `/Users/zackfan/Project/fugle/fugle-marketdata-node/`
- C#: `/Users/zackfan/Project/fubon/sdk-core/fubon-cs/FubonNeo`

**Current Architecture:**
- `core/` — Rust core library (REST + WebSocket clients)
- `py/` — Python bindings via PyO3 + maturin
- `js/` — Node.js bindings via napi-rs
- `uniffi/` — UniFFI definitions (for C# and future mobile support)

**API Coverage:**
- REST Intraday: quotes, trades, volumes, etc.
- REST Historical: candles, historical quotes
- WebSocket Intraday: real-time market data streaming

## Constraints

- **API Compatibility**: Must match official SDK method signatures and response structures exactly
- **Tech Stack**: Rust core with language-specific FFI bindings (PyO3, napi-rs, UniFFI)
- **Build Targets**: Must support common platforms (Linux, macOS, Windows) for each language
- **Dependencies**: Minimize runtime dependencies in binding layers

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Rust as core language | Memory safety, performance, cross-platform FFI support | — Pending |
| PyO3 for Python binding | Most mature Rust-Python FFI solution | — Pending |
| napi-rs for Node.js binding | Native Node.js addon with good async support | — Pending |
| UniFFI for C# binding | Multi-language support from single definition | — Pending |
| Priority: Python → Node.js → C# | Python already working, Node.js implemented, C# needs new work | — Pending |

---
*Last updated: 2025-01-30 after initialization*
