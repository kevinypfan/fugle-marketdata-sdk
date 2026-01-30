# Technology Stack

**Analysis Date:** 2026-01-30

## Languages

**Primary:**
- Rust 2021 - Core library (`core/Cargo.toml` edition 2021), WebSocket and REST clients
- TypeScript/JavaScript - Node.js bindings and package distribution (`js/package.json`)
- Python - Python bindings via maturin (`py/pyproject.toml` requires Python >=3.8)

**Secondary:**
- Rust procedural macros - NAPI-RS derive macros, pyo3 extensions

## Runtime

**Environment:**
- Rust toolchain (1.70+ implied by dependencies)
- Node.js >= 16 (specified in `js/package.json` engines)
- Python >= 3.8 (specified in `py/pyproject.toml`)
- Tokio async runtime (1.49) for concurrent operations

**Package Manager:**
- Cargo - Rust package management
- npm - JavaScript package management (lockfile present: `js/package-lock.json`)
- maturin - Python build backend for Rust extensions
- pip - Python package installation (evidenced in `.venv/lib`)

## Frameworks

**Core:**
- Tokio 1.49 - Async runtime with rt, rt-multi-thread, sync, time, macros features
- tokio-tungstenite 0.28 - WebSocket implementation with native-tls support
- ureq 2.10 - Synchronous HTTP client for REST API calls

**Language Bindings:**
- NAPI-RS 2.16 - Node.js FFI bindings for Rust code (`js/Cargo.toml`)
- pyo3 0.22 - Python bindings with extension-module feature (`py/Cargo.toml`)
- UniFFI 0.28 - Cross-language FFI bindings (experimental, `uniffi/Cargo.toml`)

**Testing:**
- criterion 0.5 - Performance benchmarking (`core/Cargo.toml` dev-dependencies)
- tokio-test 0.4 - Testing utilities for async Tokio code

**Build/Dev:**
- napi-rs/cli 2.18.4 - Node.js build tooling for NAPI-RS
- napi-build 2.1.3 - Build script dependencies for NAPI-RS

## Key Dependencies

**Critical:**
- tokio 1.49 - Runtime for async/await operations across all async code paths
- tokio-tungstenite 0.28 - WebSocket protocol handling for real-time streaming
- ureq 2.10 - HTTP client for REST API communication
- serde 1.0 - Serialization/deserialization for JSON payloads
- serde_json 1.0 - JSON processing (imported in core, js, py, uniffi)

**Infrastructure:**
- thiserror 2.0 - Error type derive macros for type-safe error handling
- anyhow 1.0 - Flexible error handling context
- exponential-backoff 2.0 - Reconnection retry logic with exponential backoff
- futures-util 0.3 - Future combinators for async composition
- url 2.5 - URL parsing and manipulation
- indexmap 2.2 - Ordered hash map for maintaining insertion order

## Configuration

**Environment:**
- REST API base URL: `https://api.fugle.tw/marketdata/v1.0` (from `core/src/rest/client.rs:45`)
- WebSocket URLs:
  - Stock: `wss://api.fugle.tw/marketdata/v1.0/stock/streaming`
  - FutOpt: `wss://api.fugle.tw/marketdata/v1.0/futopt/streaming`

**Authentication Methods:**
- API Key - X-API-KEY header
- Bearer Token - Authorization: Bearer header
- SDK Token - X-SDK-TOKEN header

**Build:**
- `core/Cargo.toml` - Core library configuration with features for python/js
- `js/Cargo.toml` - JavaScript bindings configuration (crate-type: cdylib)
- `py/Cargo.toml` - Python bindings configuration (crate-type: cdylib)
- `py/pyproject.toml` - Python package metadata with maturin build backend
- `js/package.json` - Node.js package metadata with NAPI configuration
- `uniffi/Cargo.toml` - Experimental cross-language bindings (workspace configuration)

## Platform Requirements

**Development:**
- Rust 1.70+ (estimated from dependencies)
- Platform targets supported: aarch64-apple-darwin, x86_64-apple-darwin, x86_64-unknown-linux-gnu, aarch64-unknown-linux-gnu, x86_64-pc-windows-msvc (`js/package.json` napi.triples)
- Native TLS support via OpenSSL or equivalent

**Production:**
- Node.js >= 16 for JavaScript consumers
- Python >= 3.8 for Python consumers
- Linux, macOS, Windows supported (per platform triples)
- Pre-built binaries available for common architectures (wheels for Python, .node files for Node.js)

---

*Stack analysis: 2026-01-30*
