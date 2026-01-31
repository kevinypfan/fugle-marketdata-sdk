//! FutOpt-specific subscription types
//!
//! This module provides subscription types for futures and options WebSocket channels.
//!
//! # Differences from Stock Subscriptions
//!
//! - Uses [`FutOptChannel`] instead of [`Channel`](crate::models::Channel)
//! - Has `after_hours` parameter instead of `intraday_odd_lot`
//! - Supports 4 channels (no Indices channel)
//!
//! # Example
//!
//! ```rust
//! use marketdata_core::models::futopt::FutOptChannel;
//! use marketdata_core::websocket::channels::FutOptSubscription;
//!
//! // Regular futures subscription
//! let sub = FutOptSubscription::new(FutOptChannel::Trades, "TXF202502");
//! assert_eq!(sub.key(), "trades:TXF202502");
//!
//! // After-hours subscription
//! let ah_sub = FutOptSubscription::new(FutOptChannel::Trades, "TXF202502")
//!     .with_after_hours(true);
//! assert_eq!(ah_sub.key(), "trades:TXF202502:afterhours");
//! ```

use crate::models::futopt::FutOptChannel;
use serde_json::{json, Value};

/// FutOpt-specific subscription parameters
///
/// Supports `afterHours` parameter for after-hours trading session (盤後).
///
/// # Fields
///
/// - `channel` - The data channel to subscribe to
/// - `symbol` - FutOpt contract symbol (e.g., "TXF202502", "TXFA4")
/// - `after_hours` - If true, subscribe to after-hours session data
///
/// # Example
///
/// ```rust
/// use marketdata_core::models::futopt::FutOptChannel;
/// use marketdata_core::websocket::channels::FutOptSubscription;
///
/// // Regular futures subscription
/// let sub = FutOptSubscription::new(FutOptChannel::Trades, "TXF202502");
/// assert_eq!(sub.key(), "trades:TXF202502");
///
/// // After-hours subscription
/// let ah_sub = FutOptSubscription::new(FutOptChannel::Trades, "TXF202502")
///     .with_after_hours(true);
/// assert_eq!(ah_sub.key(), "trades:TXF202502:afterhours");
///
/// // Generate WebSocket request JSON
/// let req = sub.to_subscribe_request();
/// assert_eq!(req["event"], "subscribe");
/// assert_eq!(req["data"]["channel"], "trades");
/// assert_eq!(req["data"]["symbol"], "TXF202502");
/// ```
#[derive(Debug, Clone)]
pub struct FutOptSubscription {
    /// Channel to subscribe to
    pub channel: FutOptChannel,
    /// FutOpt contract symbol (e.g., "TXF202502", "TXFA4")
    pub symbol: String,
    /// true: 盤後 (after-hours), false: 一般盤 (regular, default)
    pub after_hours: bool,
}

impl Default for FutOptSubscription {
    fn default() -> Self {
        Self {
            channel: FutOptChannel::Trades,
            symbol: String::new(),
            after_hours: false,
        }
    }
}

impl FutOptSubscription {
    /// Create new FutOpt subscription
    ///
    /// # Arguments
    ///
    /// * `channel` - The channel to subscribe to
    /// * `symbol` - FutOpt contract symbol
    ///
    /// # Example
    ///
    /// ```rust
    /// use marketdata_core::models::futopt::FutOptChannel;
    /// use marketdata_core::websocket::channels::FutOptSubscription;
    ///
    /// let sub = FutOptSubscription::new(FutOptChannel::Books, "TXFC4");
    /// assert_eq!(sub.channel, FutOptChannel::Books);
    /// assert_eq!(sub.symbol, "TXFC4");
    /// assert!(!sub.after_hours);
    /// ```
    pub fn new(channel: FutOptChannel, symbol: impl Into<String>) -> Self {
        Self {
            channel,
            symbol: symbol.into(),
            after_hours: false,
        }
    }

    /// Set after-hours trading session flag
    ///
    /// When true, the subscription will receive data from the after-hours
    /// trading session (盤後) instead of the regular session.
    ///
    /// # Example
    ///
    /// ```rust
    /// use marketdata_core::models::futopt::FutOptChannel;
    /// use marketdata_core::websocket::channels::FutOptSubscription;
    ///
    /// let sub = FutOptSubscription::new(FutOptChannel::Trades, "TXF202502")
    ///     .with_after_hours(true);
    /// assert!(sub.after_hours);
    /// assert_eq!(sub.key(), "trades:TXF202502:afterhours");
    /// ```
    pub fn with_after_hours(mut self, after_hours: bool) -> Self {
        self.after_hours = after_hours;
        self
    }

    /// Generate subscription key for tracking
    ///
    /// The key uniquely identifies a subscription for deduplication and management.
    ///
    /// # Format
    ///
    /// - Regular: `"channel:symbol"` (e.g., `"trades:TXF202502"`)
    /// - After-hours: `"channel:symbol:afterhours"` (e.g., `"trades:TXF202502:afterhours"`)
    ///
    /// # Example
    ///
    /// ```rust
    /// use marketdata_core::models::futopt::FutOptChannel;
    /// use marketdata_core::websocket::channels::FutOptSubscription;
    ///
    /// let sub = FutOptSubscription::new(FutOptChannel::Trades, "TXF202502");
    /// assert_eq!(sub.key(), "trades:TXF202502");
    ///
    /// let ah_sub = sub.clone().with_after_hours(true);
    /// assert_eq!(ah_sub.key(), "trades:TXF202502:afterhours");
    /// ```
    pub fn key(&self) -> String {
        if self.after_hours {
            format!("{}:{}:afterhours", self.channel.as_str(), self.symbol)
        } else {
            format!("{}:{}", self.channel.as_str(), self.symbol)
        }
    }

    /// Convert to wire format for WebSocket subscribe message
    ///
    /// Returns a JSON value containing the subscription data fields.
    /// The `afterHours` field is only included when true.
    ///
    /// # Example
    ///
    /// ```rust
    /// use marketdata_core::models::futopt::FutOptChannel;
    /// use marketdata_core::websocket::channels::FutOptSubscription;
    ///
    /// let sub = FutOptSubscription::new(FutOptChannel::Candles, "MXFB4");
    /// let data = sub.to_subscribe_data();
    /// assert_eq!(data["channel"], "candles");
    /// assert_eq!(data["symbol"], "MXFB4");
    /// assert!(data.get("afterHours").is_none()); // Not included when false
    ///
    /// let ah_sub = sub.with_after_hours(true);
    /// let ah_data = ah_sub.to_subscribe_data();
    /// assert_eq!(ah_data["afterHours"], true);
    /// ```
    pub fn to_subscribe_data(&self) -> Value {
        let mut data = json!({
            "channel": self.channel.as_str(),
            "symbol": self.symbol,
        });

        // Only include afterHours if true
        if self.after_hours {
            data["afterHours"] = json!(true);
        }

        data
    }

    /// Create subscribe request JSON
    ///
    /// Returns a complete WebSocket subscribe request message.
    ///
    /// # Example
    ///
    /// ```rust
    /// use marketdata_core::models::futopt::FutOptChannel;
    /// use marketdata_core::websocket::channels::FutOptSubscription;
    ///
    /// let sub = FutOptSubscription::new(FutOptChannel::Books, "TXO22000C4");
    /// let req = sub.to_subscribe_request();
    /// assert_eq!(req["event"], "subscribe");
    /// assert_eq!(req["data"]["channel"], "books");
    /// assert_eq!(req["data"]["symbol"], "TXO22000C4");
    /// ```
    pub fn to_subscribe_request(&self) -> Value {
        json!({
            "event": "subscribe",
            "data": self.to_subscribe_data()
        })
    }

    /// Create unsubscribe request JSON by subscription ID
    ///
    /// # Arguments
    ///
    /// * `id` - The subscription ID returned by the server on subscribe success
    ///
    /// # Example
    ///
    /// ```rust
    /// use marketdata_core::websocket::channels::FutOptSubscription;
    ///
    /// let req = FutOptSubscription::to_unsubscribe_request("futopt-sub-123");
    /// assert_eq!(req["event"], "unsubscribe");
    /// assert_eq!(req["data"]["id"], "futopt-sub-123");
    /// ```
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
    fn test_futopt_subscription_new() {
        let sub = FutOptSubscription::new(FutOptChannel::Trades, "TXF202502");
        assert_eq!(sub.channel, FutOptChannel::Trades);
        assert_eq!(sub.symbol, "TXF202502");
        assert!(!sub.after_hours);
    }

    #[test]
    fn test_futopt_subscription_default() {
        let sub = FutOptSubscription::default();
        assert_eq!(sub.channel, FutOptChannel::Trades);
        assert_eq!(sub.symbol, "");
        assert!(!sub.after_hours);
    }

    #[test]
    fn test_futopt_subscription_with_after_hours() {
        let sub = FutOptSubscription::new(FutOptChannel::Trades, "TXF202502").with_after_hours(true);
        assert!(sub.after_hours);
    }

    #[test]
    fn test_futopt_subscription_key_regular() {
        let sub = FutOptSubscription::new(FutOptChannel::Trades, "TXF202502");
        assert_eq!(sub.key(), "trades:TXF202502");
    }

    #[test]
    fn test_futopt_subscription_key_after_hours() {
        let sub =
            FutOptSubscription::new(FutOptChannel::Trades, "TXF202502").with_after_hours(true);
        assert_eq!(sub.key(), "trades:TXF202502:afterhours");
    }

    #[test]
    fn test_futopt_subscription_to_subscribe_data_regular() {
        let sub = FutOptSubscription::new(FutOptChannel::Candles, "MXFB4");
        let data = sub.to_subscribe_data();
        assert_eq!(data["channel"], "candles");
        assert_eq!(data["symbol"], "MXFB4");
        assert!(data.get("afterHours").is_none());
    }

    #[test]
    fn test_futopt_subscription_to_subscribe_data_after_hours() {
        let sub = FutOptSubscription::new(FutOptChannel::Trades, "TXF202502").with_after_hours(true);
        let data = sub.to_subscribe_data();
        assert_eq!(data["channel"], "trades");
        assert_eq!(data["symbol"], "TXF202502");
        assert_eq!(data["afterHours"], true);
    }

    #[test]
    fn test_futopt_subscription_to_subscribe_request() {
        let sub = FutOptSubscription::new(FutOptChannel::Books, "TXO22000C4");
        let req = sub.to_subscribe_request();
        assert_eq!(req["event"], "subscribe");
        assert_eq!(req["data"]["channel"], "books");
        assert_eq!(req["data"]["symbol"], "TXO22000C4");
    }

    #[test]
    fn test_futopt_subscription_to_unsubscribe_request() {
        let req = FutOptSubscription::to_unsubscribe_request("futopt-sub-123");
        assert_eq!(req["event"], "unsubscribe");
        assert_eq!(req["data"]["id"], "futopt-sub-123");
    }

    #[test]
    fn test_futopt_subscription_all_channels_key() {
        for channel in [
            FutOptChannel::Trades,
            FutOptChannel::Books,
            FutOptChannel::Candles,
            FutOptChannel::Aggregates,
        ] {
            let sub = FutOptSubscription::new(channel, "TXF202502");
            assert!(sub.key().starts_with(channel.as_str()));
            assert!(sub.key().contains("TXF202502"));
        }
    }

    #[test]
    fn test_futopt_subscription_clone() {
        let sub = FutOptSubscription::new(FutOptChannel::Aggregates, "MXFC4");
        let cloned = sub.clone();
        assert_eq!(sub.channel, cloned.channel);
        assert_eq!(sub.symbol, cloned.symbol);
        assert_eq!(sub.after_hours, cloned.after_hours);
    }

    #[test]
    fn test_futopt_subscription_string_conversion() {
        // Test that Into<String> works with different types
        let sub1 = FutOptSubscription::new(FutOptChannel::Trades, "TXF202502");
        let sub2 = FutOptSubscription::new(FutOptChannel::Trades, String::from("TXF202502"));
        assert_eq!(sub1.symbol, sub2.symbol);
    }
}
