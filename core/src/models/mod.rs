//! Data models for Fugle Market Data API responses
//!
//! These models match the official Fugle marketdata SDK response structures.
//!
//! # REST API Models
//! - [`Quote`] - Real-time stock quote (intraday/quote/{symbol})
//! - [`Ticker`] - Stock ticker info (intraday/ticker/{symbol})
//! - [`Trade`], [`TradesResponse`] - Trade data (intraday/trades/{symbol})
//! - [`IntradayCandle`], [`IntradayCandlesResponse`] - Intraday candles
//! - [`HistoricalCandle`], [`HistoricalCandlesResponse`] - Historical candles
//! - [`VolumeAtPrice`], [`VolumesResponse`] - Volume profile
//!
//! # WebSocket Models
//! - [`Channel`] - WebSocket channel types
//! - [`SubscribeRequest`], [`UnsubscribeRequest`] - Subscription management
//! - [`WebSocketMessage`], [`WebSocketRequest`] - Message wrappers
//! - [`AuthRequest`] - Authentication

mod candle;
mod common;
pub mod futopt;
mod quote;
pub mod streaming;
mod subscription;
mod ticker;
mod trade;
mod volume;

// Common types
pub use common::{PriceLevel, ResponseMeta, TotalStats, TradeInfo, TradingHalt};

// REST response types
pub use candle::{
    HistoricalCandle, HistoricalCandlesResponse, IntradayCandle, IntradayCandlesResponse,
};
pub use quote::Quote;
pub use ticker::Ticker;
pub use trade::{Trade, TradesResponse};
pub use volume::{VolumeAtPrice, VolumesResponse};

// WebSocket types
pub use subscription::{
    AuthRequest, Channel, SubscribeRequest, UnsubscribeRequest, WebSocketMessage, WebSocketRequest,
};
