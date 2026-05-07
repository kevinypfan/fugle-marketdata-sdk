# fugle-marketdata

Rust SDK for [Fugle](https://developer.fugle.tw) market data. Provides REST API and WebSocket clients for Taiwan stock and futures/options market data.

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

## Installation

```bash
cargo add fugle-marketdata
cargo add tokio --features rt-multi-thread,macros
```

## Quick Start

### REST API

```rust,no_run
use fugle_marketdata::{RestClient, Auth};

# fn main() -> Result<(), fugle_marketdata::MarketDataError> {
let client = RestClient::new(Auth::ApiKey(
    std::env::var("FUGLE_API_KEY").expect("FUGLE_API_KEY not set")
));

// Stock quote
let quote = client.stock().intraday().quote().symbol("2330").send()?;
println!("TSMC close price: {:?}", quote.close_price);

// Intraday candles (5-minute)
let candles = client.stock().intraday().candles()
    .symbol("2330")
    .timeframe("5")
    .send()?;
println!("Candles: {} entries", candles.data.len());

// FutOpt quote
let futopt_quote = client.futopt().intraday().quote()
    .symbol("TXFC4")
    .send()?;
println!("Futures close price: {:?}", futopt_quote.close_price);
# Ok(())
# }
```

### WebSocket Streaming

```rust,no_run
use fugle_marketdata::{
    AuthRequest, Channel, WebSocketClient,
    websocket::{ConnectionConfig, StockSubscription},
};
use std::time::Duration;

# async fn run() -> Result<(), fugle_marketdata::MarketDataError> {
let config = ConnectionConfig::fugle_stock(
    AuthRequest::with_api_key(
        std::env::var("FUGLE_API_KEY").expect("FUGLE_API_KEY not set")
    )
);
let client = WebSocketClient::new(config);

client.connect().await?;
client.subscribe_channel(StockSubscription::new(Channel::Trades, "2330")).await?;
client.subscribe_channel(StockSubscription::new(Channel::Books, "2330")).await?;

let messages = client.messages();
for _ in 0..10 {
    // receive_timeout returns Result<Option<msg>, _>:
    //   Ok(Some(msg)) — message received
    //   Ok(None)      — timeout elapsed, no message
    //   Err(_)        — channel closed
    if let Ok(Some(msg)) = messages.receive_timeout(Duration::from_secs(5)) {
        if msg.is_data() {
            println!("Data: {:?} - {:?}", msg.channel, msg.symbol);
        }
    }
}

client.disconnect().await?;
# Ok(())
# }
```

## Authentication

```rust,no_run
use fugle_marketdata::{Auth, AuthRequest};

// REST authentication
let _ = Auth::ApiKey("your-api-key".to_string());
let _ = Auth::BearerToken("your-bearer-token".to_string());
let _ = Auth::SdkToken("your-sdk-token".to_string());

// WebSocket authentication
let _ = AuthRequest::with_api_key("your-api-key");
let _ = AuthRequest::with_token("your-bearer-token");
let _ = AuthRequest::with_sdk_token("your-sdk-token");
```

## Configuration

### Reconnection

```rust,no_run
use fugle_marketdata::websocket::ReconnectionConfig;
use std::time::Duration;

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let reconnect = ReconnectionConfig::new(
    10,                              // max_attempts
    Duration::from_millis(2_000),    // initial_delay (min 100ms)
    Duration::from_millis(120_000),  // max_delay
)?;
# drop(reconnect);
# Ok(())
# }
```

### Health Check

The SDK uses passive activity detection at the WebSocket read site — no
background task, no protocol-level pings. The dispatch loop wraps each
`ws_read.next()` in `tokio::time::timeout(heartbeat_timeout, ...)` and emits
`ConnectionEvent::HeartbeatTimeout` when the timer fires, which then triggers
the auto-reconnect path.

```rust,no_run
use fugle_marketdata::websocket::HealthCheckConfig;
use std::time::Duration;

# fn main() -> Result<(), Box<dyn std::error::Error>> {
// Default: enabled=true, heartbeat_timeout=35s (server's 30s + 5s buffer)
let health = HealthCheckConfig::default();

// Custom timeout (the floor is 5s, but values below the live server's 30s
// heartbeat period will cause repeated false disconnects)
let health = HealthCheckConfig::with_timeout(Duration::from_secs(45))?;

// Opt out (discouraged — stalled connections won't surface until the OS
// times out the underlying TCP, typically hours later)
let health = HealthCheckConfig::disabled();
# drop(health);
# Ok(())
# }
```

## Error Handling

All operations return `Result<T, MarketDataError>`:

```rust,no_run
use fugle_marketdata::{MarketDataError, RestClient, Auth};

# fn main() {
# let client = RestClient::new(Auth::ApiKey("key".into()));
match client.stock().intraday().quote().symbol("2330").send() {
    Ok(quote) => println!("Price: {:?}", quote.close_price),
    Err(MarketDataError::AuthError { msg }) => eprintln!("Auth failed: {}", msg),
    Err(MarketDataError::ApiError { status, message }) => eprintln!("API {}: {}", status, message),
    Err(MarketDataError::TimeoutError { operation }) => eprintln!("Timeout: {}", operation),
    Err(e) => eprintln!("Error: {}", e),
}
# }
```

Error codes (for FFI consumers):

| Code | Variant |
|------|---------|
| 1001 | `InvalidSymbol` |
| 1002 | `DeserializationError` |
| 1003 | `RuntimeError` |
| 1004 | `ConfigError` |
| 2001 | `ConnectionError` |
| 2002 | `AuthError` |
| 2003 | `ApiError` |
| 2010 | `ClientClosed` |
| 3001 | `TimeoutError` |
| 3002 | `WebSocketError` |
| 9999 | `Other` |

## API Reference

See the [API documentation on docs.rs](https://docs.rs/fugle-marketdata) for the full type catalogue.

### REST Endpoints

**Stock intraday** — `client.stock().intraday()`:
- `.quote()` — real-time quote
- `.ticker()` — symbol information
- `.candles()` — OHLCV candles
- `.trades()` — trade history
- `.volumes()` — volume by price

**FutOpt intraday** — `client.futopt().intraday()`:
- `.quote()`, `.ticker()`, `.tickers()`, `.candles()`, `.trades()`, `.volumes()`, `.products()`

### WebSocket Channels

| Channel | Description |
|---------|-------------|
| `Trades` | Real-time trade executions |
| `Candles` | Real-time candlestick updates |
| `Books` | Order book (5 levels bid/ask) |
| `Aggregates` | Aggregated market data |
| `Indices` | Index values (stock only) |

## Architecture

`fugle-marketdata` is a thin facade over the [`fugle-marketdata-core`](https://docs.rs/fugle-marketdata-core) kernel crate. The kernel is the same library used by the Python, Node.js, and other language bindings of the official Fugle SDK. End users should depend on `fugle-marketdata`; advanced users who need direct kernel access can depend on `fugle-marketdata-core`.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
