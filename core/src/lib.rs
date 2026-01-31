//! marketdata-core: Core library for Fubon market data streaming
//!
//! This crate provides:
//! - Error types for market data operations
//! - Data models matching official Fugle marketdata SDK structures
//! - Async runtime for REST and WebSocket clients
//! - Stock and FutOpt (futures/options) market data support
//! - FFI bindings for Python and Node.js (Phase 6)
//!
//! # Stock Subscription Example
//!
//! ```rust
//! use marketdata_core::{Quote, Trade, Channel, SubscribeRequest};
//!
//! // Create a WebSocket subscription
//! let sub = SubscribeRequest::trades("2330");
//! ```
//!
//! # FutOpt Subscription Example
//!
//! ```rust
//! use marketdata_core::{FutOptChannel, FutOptSubscription};
//!
//! // Create a futures subscription
//! let sub = FutOptSubscription::new(FutOptChannel::Trades, "TXF202502");
//!
//! // After-hours trading
//! let ah_sub = FutOptSubscription::new(FutOptChannel::Books, "MXFC4")
//!     .with_after_hours(true);
//! ```

pub mod errors;
pub mod models;
pub mod rest;
pub mod runtime;
pub mod websocket;

// Re-export error types
pub use errors::MarketDataError;

// Re-export REST client types
pub use rest::{Auth, RestClient};

// Re-export WebSocket types
pub use websocket::{
    ConnectionConfig, ConnectionEvent, ConnectionState, HealthCheckConfig, MessageReceiver,
    ReconnectionConfig, WebSocketClient,
};

// Re-export model types for convenience
pub use models::{
    // Common types
    PriceLevel, ResponseMeta, TotalStats, TradeInfo, TradingHalt,
    // REST response types
    HistoricalCandle, HistoricalCandlesResponse, IntradayCandle, IntradayCandlesResponse,
    Quote, Ticker, Trade, TradesResponse, VolumeAtPrice, VolumesResponse,
    // WebSocket types
    AuthRequest, Channel, SubscribeRequest, UnsubscribeRequest, WebSocketMessage, WebSocketRequest,
};

// Re-export streaming message types (Phase 4)
pub use models::streaming::{
    AggregatesData, BooksData, CandleData, CandleHistoryItem, CandlesSnapshot, DataPayload,
    ErrorData, IndicesData, SnapshotPayload, StreamMessage, StreamTrade, SubscribedData,
    TradesData,
};

// Re-export channel subscription and parsing utilities (Phase 4)
pub use websocket::channels::{parse_channel_data, parse_stream_message, ChannelData};
pub use websocket::StockSubscription;

// Re-export FutOpt types (Phase 5)
pub use models::futopt::{
    ContractType, FutOptChannel, FutOptLastTrade, FutOptPriceLevel, FutOptQuote, FutOptSession,
    FutOptTicker, FutOptTotalStats, FutOptType, OptionRight, Product, ProductsResponse,
};
pub use websocket::channels::FutOptSubscription;

// Re-export runtime
pub use runtime::AsyncRuntime;

// Future modules (to be added in later phases):
// pub mod rest;
// pub mod ws;
