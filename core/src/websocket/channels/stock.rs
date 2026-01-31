//! Stock-specific subscription types

use crate::models::Channel;
use serde_json::{json, Value};

/// Stock-specific subscription parameters
///
/// Supports `intradayOddLot` parameter for odd lot trading.
///
/// # Example
/// ```rust
/// use marketdata_core::models::Channel;
/// use marketdata_core::websocket::channels::StockSubscription;
///
/// // Regular stock subscription
/// let sub = StockSubscription::new(Channel::Trades, "2330");
/// assert_eq!(sub.key(), "trades:2330");
///
/// // Odd lot subscription
/// let odd_lot_sub = StockSubscription::new(Channel::Trades, "2330")
///     .with_odd_lot(true);
/// assert_eq!(odd_lot_sub.key(), "trades:2330:oddlot");
/// ```
#[derive(Debug, Clone)]
pub struct StockSubscription {
    /// Channel to subscribe to
    pub channel: Channel,
    /// Stock symbol
    pub symbol: String,
    /// true: 盤中零股, false: 股票 (default)
    pub intraday_odd_lot: bool,
}

impl Default for StockSubscription {
    fn default() -> Self {
        Self {
            channel: Channel::Trades,
            symbol: String::new(),
            intraday_odd_lot: false,
        }
    }
}

impl StockSubscription {
    /// Create new stock subscription
    pub fn new(channel: Channel, symbol: impl Into<String>) -> Self {
        Self {
            channel,
            symbol: symbol.into(),
            intraday_odd_lot: false,
        }
    }

    /// Set intraday odd lot flag
    pub fn with_odd_lot(mut self, odd_lot: bool) -> Self {
        self.intraday_odd_lot = odd_lot;
        self
    }

    /// Generate subscription key for tracking
    ///
    /// Format: "channel:symbol" or "channel:symbol:oddlot"
    pub fn key(&self) -> String {
        if self.intraday_odd_lot {
            format!("{}:{}:oddlot", self.channel.as_str(), self.symbol)
        } else {
            format!("{}:{}", self.channel.as_str(), self.symbol)
        }
    }

    /// Convert to wire format for WebSocket subscribe message
    pub fn to_subscribe_data(&self) -> Value {
        let mut data = json!({
            "channel": self.channel.as_str(),
            "symbol": self.symbol,
        });

        // Only include intradayOddLot if true
        if self.intraday_odd_lot {
            data["intradayOddLot"] = json!(true);
        }

        data
    }

    /// Create subscribe request JSON
    pub fn to_subscribe_request(&self) -> Value {
        json!({
            "event": "subscribe",
            "data": self.to_subscribe_data()
        })
    }

    /// Create unsubscribe request JSON by ID
    pub fn to_unsubscribe_request(id: &str) -> Value {
        json!({
            "event": "unsubscribe",
            "data": {
                "id": id
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stock_subscription_new() {
        let sub = StockSubscription::new(Channel::Trades, "2330");
        assert_eq!(sub.channel, Channel::Trades);
        assert_eq!(sub.symbol, "2330");
        assert!(!sub.intraday_odd_lot);
    }

    #[test]
    fn test_stock_subscription_with_odd_lot() {
        let sub = StockSubscription::new(Channel::Trades, "2330").with_odd_lot(true);
        assert!(sub.intraday_odd_lot);
    }

    #[test]
    fn test_stock_subscription_key() {
        let sub = StockSubscription::new(Channel::Trades, "2330");
        assert_eq!(sub.key(), "trades:2330");
    }

    #[test]
    fn test_stock_subscription_key_odd_lot() {
        let sub = StockSubscription::new(Channel::Trades, "2330").with_odd_lot(true);
        assert_eq!(sub.key(), "trades:2330:oddlot");
    }

    #[test]
    fn test_stock_subscription_to_subscribe_data() {
        let sub = StockSubscription::new(Channel::Candles, "2454");
        let data = sub.to_subscribe_data();
        assert_eq!(data["channel"], "candles");
        assert_eq!(data["symbol"], "2454");
        assert!(data.get("intradayOddLot").is_none());
    }

    #[test]
    fn test_stock_subscription_to_subscribe_data_with_odd_lot() {
        let sub = StockSubscription::new(Channel::Trades, "2330").with_odd_lot(true);
        let data = sub.to_subscribe_data();
        assert_eq!(data["channel"], "trades");
        assert_eq!(data["symbol"], "2330");
        assert_eq!(data["intradayOddLot"], true);
    }

    #[test]
    fn test_stock_subscription_to_subscribe_request() {
        let sub = StockSubscription::new(Channel::Books, "2317");
        let req = sub.to_subscribe_request();
        assert_eq!(req["event"], "subscribe");
        assert_eq!(req["data"]["channel"], "books");
        assert_eq!(req["data"]["symbol"], "2317");
    }

    #[test]
    fn test_stock_subscription_to_unsubscribe_request() {
        let req = StockSubscription::to_unsubscribe_request("sub-123");
        assert_eq!(req["event"], "unsubscribe");
        assert_eq!(req["data"]["id"], "sub-123");
    }

    #[test]
    fn test_stock_subscription_all_channels() {
        for channel in [
            Channel::Trades,
            Channel::Candles,
            Channel::Books,
            Channel::Aggregates,
            Channel::Indices,
        ] {
            let sub = StockSubscription::new(channel, "2330");
            assert!(sub.key().starts_with(channel.as_str()));
        }
    }
}
