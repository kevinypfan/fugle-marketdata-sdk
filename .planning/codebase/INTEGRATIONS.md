# External Integrations

**Analysis Date:** 2026-01-30

## APIs & External Services

**Fugle Market Data API:**
- Fugle marketdata service for Taiwan stock and futures/options data
  - REST Endpoint: `https://api.fugle.tw/marketdata/v1.0`
  - WebSocket Stock: `wss://api.fugle.tw/marketdata/v1.0/stock/streaming`
  - WebSocket FutOpt: `wss://api.fugle.tw/marketdata/v1.0/futopt/streaming`
  - SDK/Client: Core marketdata-core library
  - Auth: Three methods supported (API Key, Bearer Token, SDK Token)
    - `FUGLE_API_KEY` environment variable (documented in `core/README.md`)

## Data Storage

**Databases:**
- Not applicable - SDK is a client library for external API consumption
- No persistent data storage implemented

**File Storage:**
- Not applicable - No file storage operations

**Caching:**
- Not applicable - No built-in caching layer

## Authentication & Identity

**Auth Provider:**
- Custom - API key-based authentication with three methods

**Implementation:**
- REST: Header-based authentication in `core/src/rest/auth.rs`
  - `Auth::ApiKey("string")` → X-API-KEY header
  - `Auth::BearerToken("string")` → Authorization: Bearer header
  - `Auth::SdkToken("string")` → X-SDK-TOKEN header

- WebSocket: `AuthRequest` model in `core/src/models/`
  - `AuthRequest::with_api_key("key")`
  - `AuthRequest::with_token("token")`
  - `AuthRequest::with_sdk_token("sdk_token")`

- Connection flow: Client sends auth in initial WebSocket message, server validates before streaming data

## Monitoring & Observability

**Error Tracking:**
- None detected - Application-level error handling only

**Logs:**
- Not implemented - Uses Rust's error types for error propagation
- Structured error types defined in `core/src/errors.rs`

**Health Check:**
- WebSocket health check implemented in `core/src/websocket/health_check.rs`
  - Ping/pong mechanism for connection monitoring
  - Configurable via `HealthCheckConfig` in `core/src/websocket/config.rs`
  - Automatic disconnection on missed pongs

**Reconnection:**
- Automatic reconnection with exponential backoff
  - `ReconnectionConfig` in `core/src/websocket/reconnection.rs`
  - `ReconnectionManager` handles retry logic
  - Uses `exponential-backoff 2.0` crate for backoff algorithm

## CI/CD & Deployment

**Hosting:**
- Not applicable - SDK is a library package

**CI Pipeline:**
- Not detected - No `.github/workflows`, GitLab CI, or build files found at root
- Build targets: 5 platform combinations for Node.js bindings (`js/package.json`)
  - aarch64-apple-darwin
  - x86_64-apple-darwin
  - x86_64-unknown-linux-gnu
  - aarch64-unknown-linux-gnu
  - x86_64-pc-windows-msvc

**Distribution:**
- npm package: `@fubon/marketdata-js` (from `js/package.json` name)
- Python package: `marketdata-py` (from `py/pyproject.toml` name)
- Cargo crates:
  - `marketdata-core` (core library)
  - `marketdata-js` (NAPI-RS bindings)
  - `marketdata-py` (pyo3 bindings)
  - `marketdata-uniffi` (experimental cross-language bindings)

## Environment Configuration

**Required env vars:**
- `FUGLE_API_KEY` - Market data API authentication key (documented in `core/README.md`)

**Secrets location:**
- Environment variables at runtime
- No `.env` files or configuration files detected in repository
- Credentials passed directly to client constructors:
  - JavaScript: `new RestClient('api-key')` or `RestClient.with_sdk_token('token')`
  - Python: `RestClient("api-key")` or `RestClient.with_sdk_token("token")`
  - Rust: `Auth::ApiKey("key")` or `Auth::SdkToken("token")`

## Webhooks & Callbacks

**Incoming:**
- WebSocket subscriptions via `SubscribeRequest` model
  - Channels: Trades, Candles, Books, Aggregates, Indices
  - Callback pattern: `MessageReceiver` for async message handling
  - Stock subscription in `core/src/websocket/channels/stock.rs`
  - FutOpt subscription in `core/src/websocket/channels/futopt.rs`

**Outgoing:**
- None - SDK only receives data from Fugle API

## Connection Pooling

**REST:**
- ureq Agent with automatic connection pooling in `RestClient` (`core/src/rest/client.rs:37-40`)
- Connection pool reuses TCP connections across requests
- Read/write timeouts: 30 seconds each
- Not thread-safe (single-threaded per client instance)

**WebSocket:**
- Single persistent connection per `WebSocketClient` instance
- Automatic reconnection on disconnection
- Multiplexed subscriptions on single connection
- Split streams: `SplitSink` for sending, `SplitStream` for receiving

---

*Integration audit: 2026-01-30*
