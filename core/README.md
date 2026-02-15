# marketdata-core

Rust library for Fugle market data streaming. Provides REST API and WebSocket clients for Taiwan stock and futures/options market data.

## Features

- **REST Client**: Synchronous HTTP client for market data queries
  - Stock intraday data (quote, ticker, candles, trades, volumes)
  - FutOpt (futures/options) intraday data
- **WebSocket Client**: Async real-time streaming
  - Stock channels: trades, candles, books, aggregates, indices
  - FutOpt channels: trades, candles, books, aggregates
  - Automatic reconnection with exponential backoff
  - Health check monitoring
- **Authentication**: API key, bearer token, or SDK token
- **FFI-ready**: Error codes and types designed for Python/JavaScript bindings

## Quick Start

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
marketdata-core = { path = "../marketdata-core" }
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

### REST API

```rust
use marketdata_core::{RestClient, Auth};

fn main() -> Result<(), marketdata_core::MarketDataError> {
    // Create client with API key authentication
    let client = RestClient::new(Auth::ApiKey(
        std::env::var("FUGLE_API_KEY").expect("FUGLE_API_KEY not set")
    ));

    // Get stock quote
    let quote = client.stock().intraday().quote().symbol("2330").send()?;
    println!("TSMC Quote:");
    println!("  Price: {:?}", quote.close_price);
    println!("  Change: {:?}", quote.change);
    println!("  Volume: {:?}", quote.total.trade_volume);

    // Get stock ticker info
    let ticker = client.stock().intraday().ticker().symbol("2330").send()?;
    println!("\nTicker: {} - {}", ticker.symbol, ticker.name);

    // Get intraday candles (5-minute)
    let candles = client.stock().intraday().candles()
        .symbol("2330")
        .timeframe("5")
        .send()?;
    println!("\nCandles: {} entries", candles.data.len());

    // Get FutOpt quote
    let futopt_quote = client.futopt().intraday().quote()
        .symbol("TXF202502")
        .send()?;
    println!("\nFutures Quote: {:?}", futopt_quote.close_price);

    Ok(())
}
```

### WebSocket Streaming

```rust
use marketdata_core::{
    AuthRequest, Channel, WebSocketClient,
    websocket::{ConnectionConfig, ConnectionEvent},
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), marketdata_core::MarketDataError> {
    // Create WebSocket client
    let config = ConnectionConfig::fugle_stock(
        AuthRequest::with_api_key(
            std::env::var("FUGLE_API_KEY").expect("FUGLE_API_KEY not set")
        )
    );
    let client = WebSocketClient::new(config);

    // Connect and authenticate
    client.connect().await?;
    println!("Connected to WebSocket");

    // Subscribe to channels
    use marketdata_core::websocket::StockSubscription;
    client.subscribe_channel(StockSubscription::new(Channel::Trades, "2330")).await?;
    client.subscribe_channel(StockSubscription::new(Channel::Books, "2330")).await?;
    println!("Subscribed to 2330 trades and books");

    // Get message receiver
    let messages = client.messages();

    // Process messages in a separate task
    let msg_handle = tokio::spawn(async move {
        for _ in 0..10 {
            match messages.recv_timeout(std::time::Duration::from_secs(5)) {
                Ok(msg) => {
                    if msg.is_data() {
                        println!("Data: {:?} - {:?}", msg.channel, msg.symbol);
                    }
                }
                Err(_) => break,
            }
        }
    });

    // Wait for messages
    msg_handle.await.ok();

    // Graceful disconnect
    client.disconnect().await?;
    println!("Disconnected");

    Ok(())
}
```

## Authentication

Three authentication methods are supported:

```rust
use marketdata_core::Auth;

// 1. API Key (most common)
let auth = Auth::ApiKey("your-api-key".to_string());

// 2. Bearer Token
let auth = Auth::BearerToken("your-bearer-token".to_string());

// 3. SDK Token
let auth = Auth::SdkToken("your-sdk-token".to_string());
```

For WebSocket:

```rust
use marketdata_core::AuthRequest;

let auth = AuthRequest::with_api_key("your-api-key");
let auth = AuthRequest::with_token("your-bearer-token");
let auth = AuthRequest::with_sdk_token("your-sdk-token");
```

## Configuration

### ReconnectionConfig

Control WebSocket automatic reconnection behavior with exponential backoff:

```rust
use marketdata_core::websocket::ReconnectionConfig;

let reconnect = ReconnectionConfig::new(
    10,      // max_attempts (min: 1)
    2000,    // initial_delay_ms (min: 100ms)
    120000   // max_delay_ms
)?;
```

**Parameters:**
- `max_attempts` (usize): Maximum reconnection attempts (default: 5, range: 1+)
- `initial_delay` (u64): Initial delay for exponential backoff in milliseconds (default: 1000ms, min: 100ms)
- `max_delay` (u64): Maximum delay cap in milliseconds (default: 60000ms)

**Validation:**
- `max_attempts` must be >= 1
- `initial_delay` must be >= 100ms (prevents connection storms)
- `max_delay` must be >= `initial_delay` (logical constraint)

### HealthCheckConfig

Control WebSocket health check (ping-pong) behavior to detect stale connections:

```rust
use marketdata_core::websocket::HealthCheckConfig;

let health = HealthCheckConfig::new(
    true,    // enabled (default: false)
    15000,   // interval_ms (min: 5000ms)
    3        // max_missed_pongs (min: 1)
)?;
```

**Parameters:**
- `enabled` (bool): Whether health check is enabled (default: false, aligned with official SDKs)
- `interval` (u64): Ping interval in milliseconds (default: 30000ms, min: 5000ms)
- `max_missed_pongs` (usize): Maximum missed pongs before considering connection stale (default: 2, min: 1)

**Validation:**
- `interval` must be >= 5000ms (prevents excessive overhead)
- `max_missed_pongs` must be >= 1

### Config Constants

All configuration constants are exported from `lib.rs` for use in binding layers:

```rust
// Reconnection defaults
pub const DEFAULT_MAX_RECONNECT_ATTEMPTS: usize = 5;
pub const DEFAULT_INITIAL_RECONNECT_DELAY_MS: u64 = 1000;
pub const DEFAULT_MAX_RECONNECT_DELAY_MS: u64 = 60000;
pub const MIN_INITIAL_DELAY_MS: u64 = 100;

// Health check defaults
pub const DEFAULT_HEALTH_CHECK_ENABLED: bool = false;
pub const DEFAULT_HEALTH_CHECK_INTERVAL_MS: u64 = 30000;
pub const DEFAULT_MAX_MISSED_PONGS: usize = 2;
pub const MIN_HEALTH_CHECK_INTERVAL_MS: u64 = 5000;
```

## Error Handling

All operations return `Result<T, MarketDataError>`:

```rust
use marketdata_core::MarketDataError;

match client.stock().intraday().quote().symbol("2330").send() {
    Ok(quote) => println!("Price: {:?}", quote.close_price),
    Err(MarketDataError::AuthError { msg }) => {
        eprintln!("Authentication failed: {}", msg);
    }
    Err(MarketDataError::ApiError { status, message }) => {
        eprintln!("API error {}: {}", status, message);
    }
    Err(MarketDataError::TimeoutError { operation }) => {
        eprintln!("Timeout during: {}", operation);
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

Error codes for FFI consumers:

| Code | Error Type |
|------|------------|
| 1001 | InvalidSymbol |
| 1002 | DeserializationError |
| 1003 | RuntimeError |
| 1004 | ConfigError |
| 2001 | ConnectionError |
| 2002 | AuthError |
| 2003 | ApiError |
| 2010 | ClientClosed |
| 3001 | TimeoutError |
| 3002 | WebSocketError |
| 9999 | Other |

## API Reference

See the [API documentation](https://docs.rs/marketdata-core) for complete details.

### REST Endpoints

**Stock Intraday:**
- `client.stock().intraday().quote()` - Real-time quote
- `client.stock().intraday().ticker()` - Symbol information
- `client.stock().intraday().candles()` - OHLCV candles
- `client.stock().intraday().trades()` - Trade history
- `client.stock().intraday().volumes()` - Volume by price

**FutOpt Intraday:**
- `client.futopt().intraday().quote()` - Real-time quote
- `client.futopt().intraday().ticker()` - Contract information
- `client.futopt().intraday().tickers()` - Multiple contracts
- `client.futopt().intraday().candles()` - OHLCV candles
- `client.futopt().intraday().trades()` - Trade history
- `client.futopt().intraday().volumes()` - Volume by price
- `client.futopt().intraday().products()` - Product listing

### WebSocket Channels

| Channel | Description |
|---------|-------------|
| Trades | Real-time trade executions |
| Candles | Real-time candlestick updates |
| Books | Order book (5 levels bid/ask) |
| Aggregates | Aggregated market data |
| Indices | Index values (stock only) |

## Examples

See the [examples/](examples/) directory:

- `rest_basic.rs` - REST API usage
- `websocket_basic.rs` - WebSocket streaming

Run examples:

```bash
export FUGLE_API_KEY="your-api-key"
cargo run --example rest_basic
cargo run --example websocket_basic
```

## License

MIT
