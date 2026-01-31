//! UniFFI model types with automatic conversion from core types
//!
//! These types use `#[derive(uniffi::Record)]` to generate FFI-compatible
//! structs that map to UDL dictionary definitions.

use marketdata_core::models as core;

// ============================================================================
// Common Types
// ============================================================================

/// Bid/Ask price level for order book
#[derive(Debug, Clone, uniffi::Record)]
pub struct PriceLevel {
    pub price: f64,
    pub size: i64,
}

impl From<core::PriceLevel> for PriceLevel {
    fn from(p: core::PriceLevel) -> Self {
        Self {
            price: p.price,
            size: p.size,
        }
    }
}

/// Total trading statistics
#[derive(Debug, Clone, uniffi::Record)]
pub struct TotalStats {
    pub trade_value: f64,
    pub trade_volume: i64,
    pub trade_volume_at_bid: Option<i64>,
    pub trade_volume_at_ask: Option<i64>,
    pub transaction: Option<i64>,
    pub time: Option<i64>,
}

impl From<core::TotalStats> for TotalStats {
    fn from(t: core::TotalStats) -> Self {
        Self {
            trade_value: t.trade_value,
            trade_volume: t.trade_volume,
            trade_volume_at_bid: t.trade_volume_at_bid,
            trade_volume_at_ask: t.trade_volume_at_ask,
            transaction: t.transaction,
            time: t.time,
        }
    }
}

/// Trade execution info
#[derive(Debug, Clone, uniffi::Record)]
pub struct TradeInfo {
    pub bid: Option<f64>,
    pub ask: Option<f64>,
    pub price: f64,
    pub size: i64,
    pub time: i64,
}

impl From<core::TradeInfo> for TradeInfo {
    fn from(t: core::TradeInfo) -> Self {
        Self {
            bid: t.bid,
            ask: t.ask,
            price: t.price,
            size: t.size,
            time: t.time,
        }
    }
}

/// Trading halt status
#[derive(Debug, Clone, uniffi::Record)]
pub struct TradingHalt {
    pub is_halted: bool,
    pub time: Option<i64>,
}

impl From<core::TradingHalt> for TradingHalt {
    fn from(t: core::TradingHalt) -> Self {
        Self {
            is_halted: t.is_halted,
            time: t.time,
        }
    }
}

// ============================================================================
// Stock Models
// ============================================================================

/// Real-time stock quote
#[derive(Debug, Clone, uniffi::Record)]
pub struct Quote {
    // Response metadata
    pub date: String,
    pub data_type: Option<String>,
    pub exchange: Option<String>,
    pub market: Option<String>,
    pub symbol: String,
    pub name: Option<String>,
    // OHLC prices with timestamps
    pub open_price: Option<f64>,
    pub open_time: Option<i64>,
    pub high_price: Option<f64>,
    pub high_time: Option<i64>,
    pub low_price: Option<f64>,
    pub low_time: Option<i64>,
    pub close_price: Option<f64>,
    pub close_time: Option<i64>,
    // Current trading info
    pub last_price: Option<f64>,
    pub last_size: Option<i64>,
    pub avg_price: Option<f64>,
    pub change: Option<f64>,
    pub change_percent: Option<f64>,
    pub amplitude: Option<f64>,
    // Order book
    pub bids: Vec<PriceLevel>,
    pub asks: Vec<PriceLevel>,
    // Aggregated stats
    pub total: Option<TotalStats>,
    pub last_trade: Option<TradeInfo>,
    pub last_trial: Option<TradeInfo>,
    pub trading_halt: Option<TradingHalt>,
    // Limit price flags
    pub is_limit_down_price: bool,
    pub is_limit_up_price: bool,
    pub is_limit_down_bid: bool,
    pub is_limit_up_bid: bool,
    pub is_limit_down_ask: bool,
    pub is_limit_up_ask: bool,
    pub is_limit_down_halt: bool,
    pub is_limit_up_halt: bool,
    // Trading session flags
    pub is_trial: bool,
    pub is_delayed_open: bool,
    pub is_delayed_close: bool,
    pub is_continuous: bool,
    pub is_open: bool,
    pub is_close: bool,
    pub last_updated: Option<i64>,
}

impl From<core::Quote> for Quote {
    fn from(q: core::Quote) -> Self {
        Self {
            date: q.date,
            data_type: q.data_type,
            exchange: q.exchange,
            market: q.market,
            symbol: q.symbol,
            name: q.name,
            open_price: q.open_price,
            open_time: q.open_time,
            high_price: q.high_price,
            high_time: q.high_time,
            low_price: q.low_price,
            low_time: q.low_time,
            close_price: q.close_price,
            close_time: q.close_time,
            last_price: q.last_price,
            last_size: q.last_size,
            avg_price: q.avg_price,
            change: q.change,
            change_percent: q.change_percent,
            amplitude: q.amplitude,
            bids: q.bids.into_iter().map(Into::into).collect(),
            asks: q.asks.into_iter().map(Into::into).collect(),
            total: q.total.map(Into::into),
            last_trade: q.last_trade.map(Into::into),
            last_trial: q.last_trial.map(Into::into),
            trading_halt: q.trading_halt.map(Into::into),
            is_limit_down_price: q.is_limit_down_price,
            is_limit_up_price: q.is_limit_up_price,
            is_limit_down_bid: q.is_limit_down_bid,
            is_limit_up_bid: q.is_limit_up_bid,
            is_limit_down_ask: q.is_limit_down_ask,
            is_limit_up_ask: q.is_limit_up_ask,
            is_limit_down_halt: q.is_limit_down_halt,
            is_limit_up_halt: q.is_limit_up_halt,
            is_trial: q.is_trial,
            is_delayed_open: q.is_delayed_open,
            is_delayed_close: q.is_delayed_close,
            is_continuous: q.is_continuous,
            is_open: q.is_open,
            is_close: q.is_close,
            last_updated: q.last_updated,
        }
    }
}

/// Stock ticker info
#[derive(Debug, Clone, uniffi::Record)]
pub struct Ticker {
    // Response metadata
    pub date: String,
    pub data_type: Option<String>,
    pub exchange: Option<String>,
    pub market: Option<String>,
    pub symbol: String,
    // Stock info
    pub name: Option<String>,
    pub name_en: Option<String>,
    pub industry: Option<String>,
    pub security_type: Option<String>,
    // Price limits
    pub reference_price: Option<f64>,
    pub limit_up_price: Option<f64>,
    pub limit_down_price: Option<f64>,
    pub previous_close: Option<f64>,
    // Trading rules
    pub can_day_trade: bool,
    pub can_buy_day_trade: bool,
    pub can_below_flat_margin_short_sell: bool,
    pub can_below_flat_sbl_short_sell: bool,
    // Attention flags
    pub is_attention: bool,
    pub is_disposition: bool,
    pub is_unusually_recommended: bool,
    pub is_specific_abnormally: bool,
    pub is_newly_compiled: bool,
    // Trading parameters
    pub matching_interval: Option<i32>,
    pub security_status: Option<String>,
    pub board_lot: Option<i32>,
    pub trading_currency: Option<String>,
    // Warrant/ETN specific
    pub exercise_price: Option<f64>,
    pub exercised_volume: Option<i64>,
    pub cancelled_volume: Option<i64>,
    pub remaining_volume: Option<i64>,
    pub exercise_ratio: Option<f64>,
    pub cap_price: Option<f64>,
    pub floor_price: Option<f64>,
    pub maturity_date: Option<String>,
    // Session times
    pub open_time: Option<String>,
    pub close_time: Option<String>,
}

impl From<core::Ticker> for Ticker {
    fn from(t: core::Ticker) -> Self {
        Self {
            date: t.date,
            data_type: t.data_type,
            exchange: t.exchange,
            market: t.market,
            symbol: t.symbol,
            name: t.name,
            name_en: t.name_en,
            industry: t.industry,
            security_type: t.security_type,
            reference_price: t.reference_price,
            limit_up_price: t.limit_up_price,
            limit_down_price: t.limit_down_price,
            previous_close: t.previous_close,
            can_day_trade: t.can_day_trade,
            can_buy_day_trade: t.can_buy_day_trade,
            can_below_flat_margin_short_sell: t.can_below_flat_margin_short_sell,
            can_below_flat_sbl_short_sell: t.can_below_flat_sbl_short_sell,
            is_attention: t.is_attention,
            is_disposition: t.is_disposition,
            is_unusually_recommended: t.is_unusually_recommended,
            is_specific_abnormally: t.is_specific_abnormally,
            is_newly_compiled: t.is_newly_compiled,
            matching_interval: t.matching_interval,
            security_status: t.security_status,
            board_lot: t.board_lot,
            trading_currency: t.trading_currency,
            exercise_price: t.exercise_price,
            exercised_volume: t.exercised_volume,
            cancelled_volume: t.cancelled_volume,
            remaining_volume: t.remaining_volume,
            exercise_ratio: t.exercise_ratio,
            cap_price: t.cap_price,
            floor_price: t.floor_price,
            maturity_date: t.maturity_date,
            open_time: t.open_time,
            close_time: t.close_time,
        }
    }
}

/// Single trade execution
#[derive(Debug, Clone, uniffi::Record)]
pub struct Trade {
    pub bid: Option<f64>,
    pub ask: Option<f64>,
    pub price: f64,
    pub size: i64,
    pub time: i64,
}

impl From<core::Trade> for Trade {
    fn from(t: core::Trade) -> Self {
        Self {
            bid: t.bid,
            ask: t.ask,
            price: t.price,
            size: t.size,
            time: t.time,
        }
    }
}

/// Trades response
#[derive(Debug, Clone, uniffi::Record)]
pub struct TradesResponse {
    pub date: String,
    pub data_type: Option<String>,
    pub exchange: Option<String>,
    pub market: Option<String>,
    pub symbol: String,
    pub data: Vec<Trade>,
}

impl From<core::TradesResponse> for TradesResponse {
    fn from(t: core::TradesResponse) -> Self {
        Self {
            date: t.date,
            data_type: t.data_type,
            exchange: t.exchange,
            market: t.market,
            symbol: t.symbol,
            data: t.data.into_iter().map(Into::into).collect(),
        }
    }
}

/// Single intraday candle
#[derive(Debug, Clone, uniffi::Record)]
pub struct IntradayCandle {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: i64,
    pub average: Option<f64>,
    pub time: i64,
}

impl From<core::IntradayCandle> for IntradayCandle {
    fn from(c: core::IntradayCandle) -> Self {
        Self {
            open: c.open,
            high: c.high,
            low: c.low,
            close: c.close,
            volume: c.volume,
            average: c.average,
            time: c.time,
        }
    }
}

/// Intraday candles response
#[derive(Debug, Clone, uniffi::Record)]
pub struct IntradayCandlesResponse {
    pub date: String,
    pub data_type: Option<String>,
    pub exchange: Option<String>,
    pub market: Option<String>,
    pub symbol: String,
    pub timeframe: Option<String>,
    pub data: Vec<IntradayCandle>,
}

impl From<core::IntradayCandlesResponse> for IntradayCandlesResponse {
    fn from(c: core::IntradayCandlesResponse) -> Self {
        Self {
            date: c.date,
            data_type: c.data_type,
            exchange: c.exchange,
            market: c.market,
            symbol: c.symbol,
            timeframe: c.timeframe,
            data: c.data.into_iter().map(Into::into).collect(),
        }
    }
}

/// Single historical candle
#[derive(Debug, Clone, uniffi::Record)]
pub struct HistoricalCandle {
    pub date: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: i64,
    pub turnover: Option<f64>,
    pub change: Option<f64>,
}

impl From<core::HistoricalCandle> for HistoricalCandle {
    fn from(c: core::HistoricalCandle) -> Self {
        Self {
            date: c.date,
            open: c.open,
            high: c.high,
            low: c.low,
            close: c.close,
            volume: c.volume,
            turnover: c.turnover,
            change: c.change,
        }
    }
}

/// Historical candles response
#[derive(Debug, Clone, uniffi::Record)]
pub struct HistoricalCandlesResponse {
    pub symbol: String,
    pub data_type: Option<String>,
    pub exchange: Option<String>,
    pub market: Option<String>,
    pub timeframe: Option<String>,
    pub adjusted: Option<bool>,
    pub data: Vec<HistoricalCandle>,
}

impl From<core::HistoricalCandlesResponse> for HistoricalCandlesResponse {
    fn from(c: core::HistoricalCandlesResponse) -> Self {
        Self {
            symbol: c.symbol,
            data_type: c.data_type,
            exchange: c.exchange,
            market: c.market,
            timeframe: c.timeframe,
            adjusted: c.adjusted,
            data: c.data.into_iter().map(Into::into).collect(),
        }
    }
}

/// Volume at a specific price level
#[derive(Debug, Clone, uniffi::Record)]
pub struct VolumeAtPrice {
    pub price: f64,
    pub volume: i64,
    pub volume_at_bid: Option<i64>,
    pub volume_at_ask: Option<i64>,
}

impl From<core::VolumeAtPrice> for VolumeAtPrice {
    fn from(v: core::VolumeAtPrice) -> Self {
        Self {
            price: v.price,
            volume: v.volume,
            volume_at_bid: v.volume_at_bid,
            volume_at_ask: v.volume_at_ask,
        }
    }
}

/// Volumes response
#[derive(Debug, Clone, uniffi::Record)]
pub struct VolumesResponse {
    pub date: String,
    pub data_type: Option<String>,
    pub exchange: Option<String>,
    pub market: Option<String>,
    pub symbol: String,
    pub data: Vec<VolumeAtPrice>,
}

impl From<core::VolumesResponse> for VolumesResponse {
    fn from(v: core::VolumesResponse) -> Self {
        Self {
            date: v.date,
            data_type: v.data_type,
            exchange: v.exchange,
            market: v.market,
            symbol: v.symbol,
            data: v.data.into_iter().map(Into::into).collect(),
        }
    }
}

// ============================================================================
// FutOpt Models
// ============================================================================

/// FutOpt price level
#[derive(Debug, Clone, uniffi::Record)]
pub struct FutOptPriceLevel {
    pub price: f64,
    pub size: i64,
}

impl From<core::futopt::FutOptPriceLevel> for FutOptPriceLevel {
    fn from(p: core::futopt::FutOptPriceLevel) -> Self {
        Self {
            price: p.price,
            size: p.size,
        }
    }
}

/// FutOpt total stats
#[derive(Debug, Clone, uniffi::Record)]
pub struct FutOptTotalStats {
    pub trade_volume: i64,
    pub total_bid_match: Option<i64>,
    pub total_ask_match: Option<i64>,
}

impl From<core::futopt::FutOptTotalStats> for FutOptTotalStats {
    fn from(t: core::futopt::FutOptTotalStats) -> Self {
        Self {
            trade_volume: t.trade_volume,
            total_bid_match: t.total_bid_match,
            total_ask_match: t.total_ask_match,
        }
    }
}

/// FutOpt last trade info
#[derive(Debug, Clone, uniffi::Record)]
pub struct FutOptLastTrade {
    pub price: f64,
    pub size: i64,
    pub time: i64,
}

impl From<core::futopt::FutOptLastTrade> for FutOptLastTrade {
    fn from(t: core::futopt::FutOptLastTrade) -> Self {
        Self {
            price: t.price,
            size: t.size,
            time: t.time,
        }
    }
}

/// FutOpt quote
#[derive(Debug, Clone, uniffi::Record)]
pub struct FutOptQuote {
    // Response metadata
    pub date: String,
    pub contract_type: Option<String>,
    pub exchange: Option<String>,
    pub symbol: String,
    pub name: Option<String>,
    // Reference prices
    pub previous_close: Option<f64>,
    // OHLC prices with timestamps
    pub open_price: Option<f64>,
    pub open_time: Option<i64>,
    pub high_price: Option<f64>,
    pub high_time: Option<i64>,
    pub low_price: Option<f64>,
    pub low_time: Option<i64>,
    pub close_price: Option<f64>,
    pub close_time: Option<i64>,
    // Current trading info
    pub last_price: Option<f64>,
    pub last_size: Option<i64>,
    pub avg_price: Option<f64>,
    pub change: Option<f64>,
    pub change_percent: Option<f64>,
    pub amplitude: Option<f64>,
    // Order book
    pub bids: Vec<FutOptPriceLevel>,
    pub asks: Vec<FutOptPriceLevel>,
    // Aggregated stats
    pub total: Option<FutOptTotalStats>,
    pub last_trade: Option<FutOptLastTrade>,
    pub last_updated: Option<i64>,
}

impl From<core::futopt::FutOptQuote> for FutOptQuote {
    fn from(q: core::futopt::FutOptQuote) -> Self {
        Self {
            date: q.date,
            contract_type: q.contract_type,
            exchange: q.exchange,
            symbol: q.symbol,
            name: q.name,
            previous_close: q.previous_close,
            open_price: q.open_price,
            open_time: q.open_time,
            high_price: q.high_price,
            high_time: q.high_time,
            low_price: q.low_price,
            low_time: q.low_time,
            close_price: q.close_price,
            close_time: q.close_time,
            last_price: q.last_price,
            last_size: q.last_size,
            avg_price: q.avg_price,
            change: q.change,
            change_percent: q.change_percent,
            amplitude: q.amplitude,
            bids: q.bids.into_iter().map(Into::into).collect(),
            asks: q.asks.into_iter().map(Into::into).collect(),
            total: q.total.map(Into::into),
            last_trade: q.last_trade.map(Into::into),
            last_updated: q.last_updated,
        }
    }
}

/// FutOpt ticker
#[derive(Debug, Clone, uniffi::Record)]
pub struct FutOptTicker {
    // Response metadata
    pub date: String,
    pub contract_type: Option<String>,
    pub exchange: Option<String>,
    pub symbol: String,
    pub name: Option<String>,
    // Reference price
    pub reference_price: Option<f64>,
    // Contract dates
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub settlement_date: Option<String>,
    // Additional fields
    pub contract_sub_type: Option<String>,
    pub is_dynamic_banding: bool,
    pub flow_group: Option<i32>,
}

impl From<core::futopt::FutOptTicker> for FutOptTicker {
    fn from(t: core::futopt::FutOptTicker) -> Self {
        Self {
            date: t.date,
            contract_type: t.contract_type,
            exchange: t.exchange,
            symbol: t.symbol,
            name: t.name,
            reference_price: t.reference_price,
            start_date: t.start_date,
            end_date: t.end_date,
            settlement_date: t.settlement_date,
            contract_sub_type: t.contract_sub_type,
            is_dynamic_banding: t.is_dynamic_banding,
            flow_group: t.flow_group,
        }
    }
}

/// FutOpt product
#[derive(Debug, Clone, uniffi::Record)]
pub struct Product {
    pub product_type: Option<String>,
    pub exchange: Option<String>,
    pub symbol: String,
    pub name: Option<String>,
    pub underlying_symbol: Option<String>,
    pub contract_type: Option<String>,
    pub contract_size: Option<i64>,
    pub underlying_type: Option<String>,
    pub status_code: Option<String>,
    pub trading_currency: Option<String>,
    pub quote_acceptable: bool,
    pub can_block_trade: bool,
    pub start_date: Option<String>,
    pub expiry_type: Option<String>,
    pub market_close_group: Option<i32>,
    pub end_session: Option<i32>,
}

impl From<core::futopt::Product> for Product {
    fn from(p: core::futopt::Product) -> Self {
        Self {
            product_type: p.product_type,
            exchange: p.exchange,
            symbol: p.symbol,
            name: p.name,
            underlying_symbol: p.underlying_symbol,
            contract_type: p.contract_type,
            contract_size: p.contract_size,
            underlying_type: p.underlying_type,
            status_code: p.status_code,
            trading_currency: p.trading_currency,
            quote_acceptable: p.quote_acceptable,
            can_block_trade: p.can_block_trade,
            start_date: p.start_date,
            expiry_type: p.expiry_type,
            market_close_group: p.market_close_group,
            end_session: p.end_session,
        }
    }
}

/// FutOpt products response
#[derive(Debug, Clone, uniffi::Record)]
pub struct ProductsResponse {
    pub date: Option<String>,
    pub product_type: Option<String>,
    pub session: Option<String>,
    pub contract_type: Option<String>,
    pub status: Option<String>,
    pub data: Vec<Product>,
}

impl From<core::futopt::ProductsResponse> for ProductsResponse {
    fn from(p: core::futopt::ProductsResponse) -> Self {
        Self {
            date: p.date,
            product_type: p.product_type,
            session: p.session,
            contract_type: p.contract_type,
            status: p.status,
            data: p.data.into_iter().map(Into::into).collect(),
        }
    }
}

// ============================================================================
// WebSocket Message Model
// ============================================================================

/// Streaming message (simplified for FFI)
#[derive(Debug, Clone, uniffi::Record)]
pub struct StreamMessage {
    pub event: String,
    pub channel: Option<String>,
    pub symbol: Option<String>,
    pub id: Option<String>,
    pub data_json: Option<String>,
    pub error_code: Option<i32>,
    pub error_message: Option<String>,
}

impl From<core::WebSocketMessage> for StreamMessage {
    fn from(msg: core::WebSocketMessage) -> Self {
        // Extract error info from data if event is "error"
        let (error_code, error_message) = if msg.event == "error" {
            let code = msg
                .data
                .as_ref()
                .and_then(|d| d.get("code"))
                .and_then(|v| v.as_i64())
                .map(|c| c as i32);
            let message = msg
                .data
                .as_ref()
                .and_then(|d| d.get("message"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            (code, message)
        } else {
            (None, None)
        };

        Self {
            event: msg.event,
            channel: msg.channel,
            symbol: msg.symbol,
            id: msg.id,
            data_json: msg.data.map(|d| d.to_string()),
            error_code,
            error_message,
        }
    }
}
