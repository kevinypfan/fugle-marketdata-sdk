# Codebase Structure

**Analysis Date:** 2026-01-30

## Directory Layout

```
fugle-marketdata-sdk/
├── core/                          # Rust core library (shared by all bindings)
│   ├── src/
│   │   ├── lib.rs                 # Core module entry point and re-exports
│   │   ├── errors.rs              # MarketDataError enum and conversions
│   │   ├── runtime.rs             # FFI-safe async runtime wrapper
│   │   ├── models/                # Data structures for API responses
│   │   │   ├── mod.rs             # Model module entry
│   │   │   ├── common.rs          # Shared types (PriceLevel, TotalStats, TradeInfo)
│   │   │   ├── quote.rs           # Quote response model
│   │   │   ├── ticker.rs          # Ticker response model
│   │   │   ├── trade.rs           # Trade/TradesResponse models
│   │   │   ├── candle.rs          # Intraday/Historical candle models
│   │   │   ├── volume.rs          # Volume at price models
│   │   │   ├── subscription.rs    # WebSocket request/response types
│   │   │   ├── streaming.rs       # Real-time streaming message types
│   │   │   └── futopt/            # Futures/options-specific models
│   │   ├── rest/                  # REST API client
│   │   │   ├── mod.rs             # REST module entry and re-exports
│   │   │   ├── auth.rs            # Auth enum (SdkToken, ApiKey, BearerToken)
│   │   │   ├── client.rs          # RestClient, StockClient, IntradayClient
│   │   │   ├── error.rs           # ureq error conversions
│   │   │   ├── stock/             # Stock-specific endpoints
│   │   │   │   ├── mod.rs
│   │   │   │   └── intraday/      # Quote, ticker, trades, candles, volumes
│   │   │   └── futopt/            # Futures/options endpoints
│   │   │       ├── mod.rs
│   │   │       └── intraday/      # FutOpt quote, trades, candles, products
│   │   └── websocket/             # WebSocket streaming client
│   │       ├── mod.rs             # WebSocket module entry
│   │       ├── config.rs          # ConnectionConfig
│   │       ├── connection.rs      # WebSocketClient and state machine
│   │       ├── subscription.rs    # SubscriptionManager
│   │       ├── message.rs         # MessageReceiver (mpsc wrapper)
│   │       ├── reconnection.rs    # ReconnectionManager with exponential backoff
│   │       ├── health_check.rs    # HealthCheck and ping/pong logic
│   │       └── channels/          # Channel-specific parsing
│   │           ├── mod.rs
│   │           ├── parser.rs      # parse_stream_message, parse_channel_data
│   │           ├── stock.rs       # StockSubscription builder
│   │           └── futopt.rs      # FutOptSubscription builder
│   ├── Cargo.toml                 # Rust dependencies (tokio, ureq, serde, etc.)
│   ├── Cargo.lock
│   ├── tests/                     # Integration tests
│   ├── benches/                   # Performance benchmarks
│   ├── examples/                  # Usage examples
│   └── README.md
├── js/                            # Node.js NAPI-RS bindings
│   ├── src/
│   │   ├── lib.rs                 # NAPI entry point and module registration
│   │   ├── client.rs              # RestClient NAPI bindings
│   │   ├── websocket.rs           # WebSocketClient NAPI bindings
│   │   └── errors.rs              # MarketDataError NAPI mapping
│   ├── index.js                   # JavaScript entry point (loads .node binary)
│   ├── index.d.ts                 # TypeScript type definitions
│   ├── package.json               # NPM package metadata
│   ├── Cargo.toml                 # Build config for NAPI-RS
│   ├── build.rs                   # Custom build script
│   ├── test_rest.js               # REST client integration test
│   ├── test_websocket.js          # WebSocket client integration test
│   └── README.md
├── py/                            # Python PyO3 bindings
│   ├── src/
│   │   ├── lib.rs                 # PyO3 module entry (marketdata_py)
│   │   ├── client.rs              # RestClient Python class bindings
│   │   ├── websocket.rs           # WebSocketClient Python class bindings
│   │   ├── callback.rs            # Python callback handler for WebSocket
│   │   ├── iterator.rs            # MessageIterator for message streaming
│   │   ├── types.rs               # Python type conversions
│   │   └── errors.rs              # MarketDataError Python exception mapping
│   ├── pyproject.toml             # Python project config (maturin build)
│   ├── Cargo.toml                 # Rust build config for PyO3
│   ├── Cargo.lock
│   ├── test_rest.py               # REST client integration test
│   ├── test_websocket.py          # WebSocket client integration test
│   ├── test_ws_callback.py        # WebSocket callback mode test
│   ├── test_ws_stream.py          # WebSocket iterator mode test
│   ├── test_2330.py               # Specific symbol test
│   └── README.md
├── uniffi/                        # EXPERIMENTAL: UniFFI bindings (C#, Go, C++, etc.)
│   ├── src/
│   │   ├── lib.rs                 # UniFFI module entry and error mapping
│   │   ├── client.rs              # RestClient UniFFI bindings (JSON returns)
│   │   ├── futopt.rs              # FutOpt-specific bindings
│   │   └── marketdata.udl         # UniFFI Interface Definition Language
│   ├── Cargo.toml
│   ├── build.rs                   # UniFFI code generation
│   └── README.md
├── .planning/
│   └── codebase/                  # GSD codebase documentation
│       ├── ARCHITECTURE.md        # (this file) System design and patterns
│       └── STRUCTURE.md           # (this file) File organization guide
└── .git/                          # Git repository
```

## Directory Purposes

**core/src/:**
- Purpose: Unified Rust implementation of Fugle marketdata SDK
- Contains: Error handling, data models, REST client, WebSocket client, async runtime
- Key files: `lib.rs` (main entry), `errors.rs`, `models/mod.rs`, `rest/mod.rs`, `websocket/mod.rs`

**core/src/models/:**
- Purpose: Serialize/deserialize API responses and streaming messages
- Contains: REST response types (Quote, Ticker, Trade, Candle), WebSocket types, FutOpt models
- Key files: `mod.rs` (module entry), `streaming.rs` (real-time message types)

**core/src/rest/:**
- Purpose: Synchronous HTTP client for REST API endpoints
- Contains: RestClient, Auth, connection pooling via ureq, error conversion
- Pattern: Organized as `stock/intraday/` and `futopt/intraday/` endpoint modules
- Key files: `client.rs` (main entry), `auth.rs` (credential types)

**core/src/websocket/:**
- Purpose: Asynchronous WebSocket client for real-time streaming
- Contains: Connection lifecycle, subscription management, reconnection logic, health checking
- Pattern: State machine (Disconnected → Connecting → Authenticating → Connected)
- Key files: `connection.rs` (main client), `channels/parser.rs` (message parsing)

**js/src/:**
- Purpose: NAPI-RS Rust bindings for Node.js
- Contains: JS class wrappers for RestClient, WebSocketClient
- Exports to: `index.js` (JavaScript entry), `index.d.ts` (TypeScript types)

**py/src/:**
- Purpose: PyO3 Rust bindings for Python
- Contains: Python class bindings (RestClient, WebSocketClient, StockClient, FutOptClient)
- Module: `marketdata_py` (Python package name)
- Key files: `lib.rs` (module entry), `client.rs`, `websocket.rs`

**uniffi/src/:**
- Purpose: EXPERIMENTAL UniFFI bindings for C#, Go, C++, Kotlin, Swift
- Contains: FFI definitions in UDL (Interface Definition Language)
- Design: All values returned as JSON strings for cross-language compatibility

## Key File Locations

**Entry Points:**

Core:
- `core/src/lib.rs` - Rust library entry, public API re-exports

JavaScript:
- `js/index.js` - Node.js entry point, loads compiled `.node` binary
- `js/index.d.ts` - TypeScript type definitions

Python:
- `py/src/lib.rs` - PyO3 module registration (`marketdata_py`)
- Compiled as: `marketdata_py.so` / `.pyd` file

**Configuration:**

- `core/Cargo.toml` - Core dependencies (tokio, ureq, serde, tungstenite, etc.)
- `js/package.json` - NPM package metadata, NAPI-RS build config
- `py/pyproject.toml` - Python project metadata, maturin build backend
- `uniffi/src/marketdata.udl` - UniFFI interface definitions

**Core Logic:**

- `core/src/errors.rs` - Error type and conversions
- `core/src/rest/client.rs` - RestClient with connection pooling
- `core/src/websocket/connection.rs` - WebSocketClient state machine
- `core/src/models/streaming.rs` - Real-time message types
- `core/src/websocket/channels/parser.rs` - Message parsing logic

**Testing:**

- `js/test_rest.js` - Node.js REST client tests
- `js/test_websocket.js` - Node.js WebSocket client tests
- `py/test_rest.py` - Python REST client tests
- `py/test_websocket.py` - Python WebSocket client tests
- `py/test_ws_callback.py` - Python callback-based streaming
- `py/test_ws_stream.py` - Python iterator-based streaming

## Naming Conventions

**Files:**

- Rust modules: `snake_case.rs` (e.g., `connection.rs`, `health_check.rs`)
- Module entries: `mod.rs` for directory-level exports
- Test files: `test_*.rs` inside src/ or standalone in tests/ dir
- Python test scripts: `test_*.py` prefixed
- JavaScript test scripts: `test_*.js` prefixed

**Directories:**

- Feature/domain: `snake_case` (e.g., `websocket/`, `futopt/`)
- API categories: `intraday/` for intraday endpoints, `historical/` for historical
- Language bindings: `js/`, `py/`, `uniffi/` (lowercase language abbreviations)

**Rust Identifiers:**

- Modules: `snake_case` (e.g., `websocket`, `connection_config`)
- Types/Structs: `PascalCase` (e.g., `RestClient`, `ConnectionConfig`, `WebSocketMessage`)
- Functions: `snake_case` (e.g., `parse_stream_message`, `parse_channel_data`)
- Constants: `UPPER_SNAKE_CASE`
- Traits: `PascalCase` (e.g., `From<T>`)

## Where to Add New Code

**New REST Endpoint:**

1. Add endpoint handler in `core/src/rest/stock/intraday/` or `core/src/rest/futopt/intraday/`
2. Add response model in `core/src/models/` (e.g., `core/src/models/new_model.rs`)
3. Export from `core/src/models/mod.rs` and `core/src/lib.rs`
4. Tests: Create `core/tests/rest_endpoint_test.rs`

**New WebSocket Channel:**

1. Add channel variant to `Channel` enum in `core/src/models/subscription.rs`
2. Add parsing logic in `core/src/websocket/channels/parser.rs`
3. Add channel-specific data type in `core/src/models/streaming.rs`
4. Update `parse_channel_data()` function with new variant
5. Tests: Create `core/tests/ws_channel_test.rs`

**New Error Type:**

1. Add variant to `MarketDataError` enum in `core/src/errors.rs`
2. Implement error conversion if from external crate
3. Update FFI mappings: `js/src/errors.rs`, `py/src/errors.rs`, `uniffi/src/lib.rs`
4. Tests: Add test case in error handling tests

**JavaScript Usage Example:**

- Add to `js/test_*.js` files following existing patterns
- Ensure TypeScript types in `js/index.d.ts` are updated

**Python Usage Example:**

- Add to `py/test_*.py` files following existing patterns
- Use pytest conventions for test functions

**Utilities:**

- Shared helpers: `core/src/` (Rust), no separate utilities directory
- No separate utils module - keep logic in responsible modules

## Special Directories

**core/target/:**
- Purpose: Rust build artifacts (compiled binaries, intermediate files)
- Generated: Yes (by cargo build)
- Committed: No (.gitignore excludes)

**js/node_modules/:**
- Purpose: NPM dependencies for NAPI-RS CLI tooling
- Generated: Yes (by npm install)
- Committed: No (.gitignore excludes)

**py/.venv/:**
- Purpose: Python virtual environment for development
- Generated: Yes (by python -m venv)
- Committed: No (.gitignore excludes)

**.planning/codebase/:**
- Purpose: GSD codebase documentation (ARCHITECTURE.md, STRUCTURE.md, etc.)
- Generated: No (manually created by GSD mapper)
- Committed: Yes (part of project docs)

## Shared State and Dependencies

**Authentication Flow:**
- `RestClient` created with `Auth` enum (defined in `core/src/rest/auth.rs`)
- Auth injected into all HTTP requests
- `WebSocketClient` created with `AuthRequest` (defined in `core/src/models/subscription.rs`)

**Error Flow:**
- All modules return `Result<T, MarketDataError>`
- Core `MarketDataError` defined in `core/src/errors.rs`
- Language bindings convert to idiomatic exception types (Python: Exception, JS: Error)

**Async Runtime:**
- Tokio runtime managed by `AsyncRuntime` in `core/src/runtime.rs`
- Used by WebSocket client for connection and health checks
- FFI safety ensured via `ffi_catch_ptr!` and `ffi_catch_void!` macros

---

*Structure analysis: 2026-01-30*
