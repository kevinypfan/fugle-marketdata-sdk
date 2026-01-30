# Architecture

**Analysis Date:** 2026-01-30

## Pattern Overview

**Overall:** Multi-language FFI bridge architecture with unified Rust core

**Key Characteristics:**
- Single Rust core library (`marketdata-core`) providing all market data logic
- Language-specific bindings (NAPI-RS for Node.js, PyO3 for Python, UniFFI for others)
- Separated concerns: REST clients, WebSocket streaming, data models, error handling
- Authentication abstraction supporting multiple credential types
- Dual transport: synchronous REST (via ureq) and asynchronous WebSocket (via Tokio)

## Layers

**Core Logic Layer (Rust):**
- Purpose: Market data acquisition, parsing, subscription management
- Location: `core/src/`
- Contains: Error types, data models, REST client, WebSocket client, runtime abstractions
- Depends on: tokio (async runtime), ureq (HTTP), tokio-tungstenite (WebSocket), serde (serialization)
- Used by: JS bindings, Python bindings, UniFFI bindings

**REST Client Layer:**
- Purpose: Synchronous HTTP requests to Fugle API endpoints
- Location: `core/src/rest/`
- Contains: `RestClient` (main entry point), auth types, stock/futopt endpoint handlers
- Pattern: Agent-based connection pooling via ureq with base URL configuration
- Endpoints organized as: `stock/intraday/` and `futopt/intraday/`

**WebSocket Streaming Layer:**
- Purpose: Real-time asynchronous message streaming with reconnection/health checking
- Location: `core/src/websocket/`
- Contains: Connection lifecycle, message parsing, subscription management, reconnection logic
- Components:
  - `connection.rs`: State machine (Disconnected → Connecting → Authenticating → Connected)
  - `subscription.rs`: Subscription request/response handling
  - `channels/`: Channel-specific parsing (stock vs. futopt)
  - `reconnection.rs`: Exponential backoff retry logic
  - `health_check.rs`: Pong monitoring for stale connections

**Data Models Layer:**
- Purpose: Serialize/deserialize API responses and streaming messages
- Location: `core/src/models/`
- Contains: REST response types (Quote, Ticker, Trade, Candle, Volume), WebSocket types, FutOpt-specific models
- Streaming models: `StreamMessage`, `ChannelData`, `TradesData`, `BooksData`, etc.

**Language Bindings Layer:**
- Purpose: Expose Rust types to host languages with idiomatic APIs
- Locations:
  - Node.js: `js/src/` (NAPI-RS) → `index.js` entry, `index.d.ts` types
  - Python: `py/src/` (PyO3) → `marketdata_py` module
  - Other: `uniffi/src/` (UniFFI) → JSON string returns, UDL schema

**FFI Runtime Layer:**
- Purpose: Safe async task spawning across FFI boundaries without panic leakage
- Location: `core/src/runtime.rs`
- Pattern: Wraps Tokio runtime with panic catch macros (`ffi_catch_ptr!`, `ffi_catch_void!`)

## Data Flow

**REST API Flow:**

1. Application creates `RestClient` with `Auth::SdkToken` / `Auth::ApiKey` / `Auth::BearerToken`
2. Calls chain: `client.stock().intraday().quote("2330")`
3. `StockClient` → `IntradayClient` → executes HTTP GET via `ureq::Agent`
4. Server responds with JSON (e.g., quote response with price, bid/ask, volume)
5. `serde_json` deserializes into `Quote` struct
6. Result returned to caller (language-specific wrapper in JS/Python)

**WebSocket Streaming Flow:**

1. Application creates `WebSocketClient` with `ConnectionConfig`
2. Calls `client.connect()` → initiates async connection to `wss://stream.fugle.tw/...`
3. State transitions: `Disconnected` → `Connecting` → `Authenticating` → `Connected`
4. Application subscribes: `client.subscribe(SubscribeRequest::trades("2330"))`
5. Sends JSON subscribe request over WebSocket
6. Server sends `subscribed` event, then streams trade messages
7. Each message parsed by channel-specific parser (`parse_channel_data`)
8. Messages delivered via `MessageReceiver` (mpsc channel for FFI safety)
9. Health check periodically sends ping, expects pong within timeout
10. If pong missed: triggers reconnection with exponential backoff

**State Management:**

- WebSocket connection state: `Arc<RwLock<ConnectionState>>` for thread-safe reads
- Subscriptions: `Arc<SubscriptionManager>` tracks active subscriptions
- Event notifications: mpsc channel (`Arc<Mutex<mpsc::Receiver>>`) for event listeners
- Message queue: mpsc channel for received messages (thread-safe for FFI)

## Key Abstractions

**Auth (Authentication):**
- Purpose: Flexible credential handling for API authentication
- Types: `SdkToken`, `ApiKey`, `BearerToken`
- Usage: Passed to `RestClient::new()`, injected into HTTP headers
- Files: `core/src/rest/auth.rs`

**Channel (Subscription Type):**
- Purpose: Type-safe subscription channel specification
- Stock channels: Trades, Books, Candles, Tickers (with odd-lot variant)
- FutOpt channels: Trades, Books, Indexes, Products (with after-hours variant)
- Files: `core/src/models/subscription.rs`, `core/src/websocket/channels/`

**ChannelData (Parsed Message):**
- Purpose: Enum representing different parsed streaming message types
- Variants: `TradesData`, `BooksData`, `CandlesData`, `SnapshotPayload`
- Returns: Strongly-typed data after parsing (no untyped JSON)
- Files: `core/src/websocket/channels/parser.rs`

**WebSocketMessage (Top-level Streaming Event):**
- Purpose: Wrapper for all WebSocket events from server
- Types: Snapshot (initial subscription data), Data (real-time updates), Error (subscription errors), Connected, Subscribed
- Files: `core/src/models/streaming.rs`

**MarketDataError (Error Type):**
- Purpose: Unified error handling across REST, WebSocket, and FFI
- Variants: InvalidSymbol, DeserializationError, RuntimeError, ConnectionError, AuthError, ApiError, TimeoutError, WebSocketError, ClientClosed
- Pattern: Implements `From<TungsteniteError>`, `From<serde_json::Error>`, `From<anyhow::Error>`
- Files: `core/src/errors.rs`

## Entry Points

**Rust Library (Direct):**
- Location: `core/src/lib.rs`
- Creates: `RestClient` with `Auth`, `WebSocketClient` with `ConnectionConfig`
- Exports: All public types via re-exports (models, websocket, rest modules)

**Node.js Binding:**
- Location: `js/src/lib.rs` (NAPI-RS entry point)
- Classes exposed: `RestClient`, `WebSocketClient` plus sub-clients (`StockClient`, `FutOptClient`)
- Usage: `require('@fubon/marketdata-js')` or `new RestClient('api-key')`
- Files: `js/index.js` (wrapper), `js/index.d.ts` (TypeScript definitions)

**Python Binding:**
- Location: `py/src/lib.rs` (PyO3 entry point)
- Module: `marketdata_py` (registered in `#[pymodule]`)
- Classes: `RestClient`, `WebSocketClient`, `StockClient`, `FutOptClient`, etc.
- Usage: `from marketdata_py import RestClient; client = RestClient('api-key')`
- Error type: `MarketDataError` (Python exception subclass)

**UniFFI Binding (Alpha):**
- Location: `uniffi/src/lib.rs`
- Schema: `uniffi/src/marketdata.udl` (Interface Definition Language)
- Returns: All values as JSON strings (cross-language compatibility)
- Targets: C#, Go, C++, Kotlin, Swift (via UniFFI code generation)

## Error Handling

**Strategy:** Error classification with numeric codes + typed enum variants

**Patterns:**

1. **Creation-time errors:** Invalid symbol format or deserialization failure → `MarketDataError::InvalidSymbol`, `MarketDataError::DeserializationError`
2. **Connection errors:** TCP/WebSocket failures → `ConnectionError` (retryable)
3. **Authentication errors:** 401/403 HTTP or TLS cert → `AuthError` (fatal)
4. **API errors:** HTTP 4xx/5xx responses → `ApiError` with status code
5. **Timeout errors:** Operations exceeding duration → `TimeoutError`
6. **FFI panic handling:** Panics at FFI boundaries caught and converted to null pointers or error returns

**Error code ranges:**
- 1000-1999: Client errors (bad input, deserialization)
- 2000-2999: Server/API errors (auth, connection, HTTP)
- 3000-3999: Network errors (timeout, WebSocket)
- 9000-9999: Internal errors (unexpected failures)

## Cross-Cutting Concerns

**Logging:**
- Approach: eprintln! for FFI panic boundaries, standard Rust logging for core
- Example: `eprintln!("PANIC: Caught panic at FFI boundary")` in `ffi_catch_ptr!` macro

**Validation:**
- Symbol validation: `InvalidSymbol` error for malformed symbols
- Channel validation: Enum types prevent invalid channel subscriptions
- Product type validation: FutOpt requests validated against FUTURE/OPTION types

**Authentication:**
- Three credential types in `Auth` enum: `SdkToken`, `ApiKey`, `BearerToken`
- Injected via HTTP headers (X-API-Key, Authorization)
- Configuration at client creation time, immutable afterward

**Async/Sync Bridging:**
- REST: Synchronous ureq HTTP (blocking but simple for sync languages)
- WebSocket: Async Tokio spawned in background thread
- FFI: `AsyncRuntime` wraps Tokio, `ffi_catch_*` macros prevent panic leakage
- Message passing: mpsc channels for thread-safe notification across FFI

---

*Architecture analysis: 2026-01-30*
