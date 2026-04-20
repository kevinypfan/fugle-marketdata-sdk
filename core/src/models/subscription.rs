//! WebSocket subscription types - matches Fugle WebSocket API

use serde::{Deserialize, Serialize};

/// WebSocket channel types for stock market data
///
/// These match the official Fugle WebSocket API channels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Channel {
    /// Real-time trades
    Trades,
    /// Candlestick data
    Candles,
    /// Order book (bids/asks)
    Books,
    /// Aggregate data (quote-like)
    Aggregates,
    /// Index data
    Indices,
}

impl Channel {
    /// Get channel name as string
    pub fn as_str(&self) -> &'static str {
        match self {
            Channel::Trades => "trades",
            Channel::Candles => "candles",
            Channel::Books => "books",
            Channel::Aggregates => "aggregates",
            Channel::Indices => "indices",
        }
    }
}

/// Subscription request for WebSocket
///
/// Modifier flags (`after_hours`, `intraday_odd_lot`) are preserved across
/// reconnection so a 盤後 or 盤中零股 subscription comes back as the same
/// session — previous design stored only `{channel, symbol}` which silently
/// downgraded on resubscribe.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct SubscribeRequest {
    /// Channel to subscribe to
    pub channel: String,

    /// Stock symbol (optional for some channels)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,

    /// FutOpt after-hours session flag. Sent as `afterHours: true` on wire
    /// when set; absent otherwise so stock path serializes unchanged.
    #[serde(rename = "afterHours", skip_serializing_if = "Option::is_none")]
    pub after_hours: Option<bool>,

    /// Stock intraday odd-lot flag. Sent as `intradayOddLot: true` on wire
    /// when set; absent otherwise.
    #[serde(rename = "intradayOddLot", skip_serializing_if = "Option::is_none")]
    pub intraday_odd_lot: Option<bool>,
}

impl SubscribeRequest {
    /// Create a new subscription request
    pub fn new(channel: Channel, symbol: impl Into<String>) -> Self {
        Self {
            channel: channel.as_str().to_string(),
            symbol: Some(symbol.into()),
            ..Default::default()
        }
    }

    /// Create a trades channel subscription
    pub fn trades(symbol: impl Into<String>) -> Self {
        Self::new(Channel::Trades, symbol)
    }

    /// Create a candles channel subscription
    pub fn candles(symbol: impl Into<String>) -> Self {
        Self::new(Channel::Candles, symbol)
    }

    /// Create a books channel subscription
    pub fn books(symbol: impl Into<String>) -> Self {
        Self::new(Channel::Books, symbol)
    }

    /// Create an aggregates channel subscription
    pub fn aggregates(symbol: impl Into<String>) -> Self {
        Self::new(Channel::Aggregates, symbol)
    }

    /// Generate subscription key for tracking.
    ///
    /// Includes modifier suffix so 盤後/零股 subscriptions occupy distinct
    /// slots from their regular-session counterparts — the key is the
    /// identity used by `SubscriptionManager` for reconnect, replacement,
    /// and unsubscribe lookup.
    pub fn key(&self) -> String {
        let base = match &self.symbol {
            Some(symbol) => format!("{}:{}", self.channel, symbol),
            None => self.channel.clone(),
        };
        if self.after_hours == Some(true) {
            format!("{base}:afterhours")
        } else if self.intraday_odd_lot == Some(true) {
            format!("{base}:oddlot")
        } else {
            base
        }
    }
}

/// Unsubscribe request for WebSocket
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UnsubscribeRequest {
    /// Subscription ID to unsubscribe
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Multiple subscription IDs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ids: Option<Vec<String>>,
}

impl UnsubscribeRequest {
    /// Unsubscribe by single ID
    pub fn by_id(id: impl Into<String>) -> Self {
        Self {
            id: Some(id.into()),
            ids: None,
        }
    }

    /// Unsubscribe by multiple IDs
    pub fn by_ids(ids: Vec<String>) -> Self {
        Self {
            id: None,
            ids: Some(ids),
        }
    }
}

/// WebSocket message wrapper (incoming messages)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketMessage {
    /// Event type (e.g., "data", "subscribed", "error", "authenticated", "pong")
    pub event: String,

    /// Message data (varies by event type)
    #[serde(default)]
    pub data: Option<serde_json::Value>,

    /// Channel (for data events)
    #[serde(default)]
    pub channel: Option<String>,

    /// Symbol (for data events)
    #[serde(default)]
    pub symbol: Option<String>,

    /// Subscription ID (for subscribed events)
    #[serde(default)]
    pub id: Option<String>,
}

impl WebSocketMessage {
    /// Check if this is an authentication success message
    pub fn is_authenticated(&self) -> bool {
        self.event == "authenticated"
    }

    /// Check if this is an error message
    pub fn is_error(&self) -> bool {
        self.event == "error"
    }

    /// Check if this is a data message
    pub fn is_data(&self) -> bool {
        self.event == "data"
    }

    /// Check if this is a pong message. With the activity-timer health check
    /// the SDK never sends internal pings, so any pong arriving on this
    /// connection is a response to a user-initiated `ping(state)` and is
    /// forwarded to user message callbacks unchanged.
    pub fn is_pong(&self) -> bool {
        self.event == "pong"
    }

    /// Check if this is a server-initiated heartbeat (`{"event":"heartbeat"}`).
    /// Heartbeats arrive every ~30 seconds and carry a microsecond timestamp
    /// in `data.time`. They are forwarded to user message callbacks so callers
    /// can use them for latency measurement or clock alignment.
    pub fn is_heartbeat(&self) -> bool {
        self.event == "heartbeat"
    }

    /// Check if this is a subscribed confirmation
    pub fn is_subscribed(&self) -> bool {
        self.event == "subscribed"
    }

    /// Get error message if this is an error
    pub fn error_message(&self) -> Option<String> {
        if !self.is_error() {
            return None;
        }
        self.data
            .as_ref()
            .and_then(|d| d.get("message"))
            .and_then(|m| m.as_str())
            .map(|s| s.to_string())
    }

}

/// WebSocket authentication request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthRequest {
    /// API key (if using API key auth)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub apikey: Option<String>,

    /// Bearer token (if using token auth)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,

    /// SDK token (if using SDK token auth)
    #[serde(rename = "sdkToken", skip_serializing_if = "Option::is_none")]
    pub sdk_token: Option<String>,
}

impl AuthRequest {
    /// Create API key auth request
    pub fn with_api_key(api_key: impl Into<String>) -> Self {
        Self {
            apikey: Some(api_key.into()),
            token: None,
            sdk_token: None,
        }
    }

    /// Create bearer token auth request
    pub fn with_token(token: impl Into<String>) -> Self {
        Self {
            apikey: None,
            token: Some(token.into()),
            sdk_token: None,
        }
    }

    /// Create SDK token auth request
    pub fn with_sdk_token(sdk_token: impl Into<String>) -> Self {
        Self {
            apikey: None,
            token: None,
            sdk_token: Some(sdk_token.into()),
        }
    }
}

/// WebSocket outgoing message (for sending to server)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketRequest {
    /// Event type
    pub event: String,

    /// Event data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl WebSocketRequest {
    /// Create auth request
    pub fn auth(auth: AuthRequest) -> Self {
        Self {
            event: "auth".to_string(),
            data: Some(serde_json::to_value(auth).unwrap()),
        }
    }

    /// Create subscribe request
    pub fn subscribe(sub: SubscribeRequest) -> Self {
        Self {
            event: "subscribe".to_string(),
            data: Some(serde_json::to_value(sub).unwrap()),
        }
    }

    /// Create unsubscribe request
    pub fn unsubscribe(unsub: UnsubscribeRequest) -> Self {
        Self {
            event: "unsubscribe".to_string(),
            data: Some(serde_json::to_value(unsub).unwrap()),
        }
    }

    /// Create ping request
    pub fn ping(state: Option<String>) -> Self {
        Self {
            event: "ping".to_string(),
            data: state.map(|s| serde_json::json!({"state": s})),
        }
    }

    /// Create subscriptions list request
    pub fn subscriptions() -> Self {
        Self {
            event: "subscriptions".to_string(),
            data: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_serialization() {
        let channel = Channel::Trades;
        let json = serde_json::to_string(&channel).unwrap();
        assert_eq!(json, "\"trades\"");
    }

    #[test]
    fn test_channel_deserialization() {
        let channel: Channel = serde_json::from_str("\"candles\"").unwrap();
        assert_eq!(channel, Channel::Candles);
    }

    #[test]
    fn test_subscribe_request() {
        let req = SubscribeRequest::trades("2330");
        assert_eq!(req.channel, "trades");
        assert_eq!(req.symbol.as_deref(), Some("2330"));
        assert_eq!(req.key(), "trades:2330");
    }

    #[test]
    fn test_subscribe_request_serialization() {
        let req = SubscribeRequest::trades("2330");
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"channel\":\"trades\""));
        assert!(json.contains("\"symbol\":\"2330\""));
        // Modifier flags absent when None — stock regular-session path
        // wire payload must stay byte-identical to pre-fix behavior.
        assert!(!json.contains("afterHours"));
        assert!(!json.contains("intradayOddLot"));
    }

    #[test]
    fn test_subscribe_request_after_hours_key_and_wire() {
        let req = SubscribeRequest {
            channel: "trades".to_string(),
            symbol: Some("TXF1!".to_string()),
            after_hours: Some(true),
            ..Default::default()
        };
        // Key preserves afterhours suffix → reconnect replays the correct session
        assert_eq!(req.key(), "trades:TXF1!:afterhours");
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"afterHours\":true"));
    }

    #[test]
    fn test_subscribe_request_oddlot_key_and_wire() {
        let req = SubscribeRequest {
            channel: "trades".to_string(),
            symbol: Some("2330".to_string()),
            intraday_odd_lot: Some(true),
            ..Default::default()
        };
        assert_eq!(req.key(), "trades:2330:oddlot");
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"intradayOddLot\":true"));
    }

    #[test]
    fn test_subscribe_request_deserialize_without_modifiers() {
        // Legacy payloads without the new fields must still deserialize.
        let json = r#"{"channel":"trades","symbol":"2330"}"#;
        let req: SubscribeRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.after_hours, None);
        assert_eq!(req.intraday_odd_lot, None);
        assert_eq!(req.key(), "trades:2330");
    }

    #[test]
    fn test_unsubscribe_request() {
        let req = UnsubscribeRequest::by_id("sub-123");
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"id\":\"sub-123\""));
    }

    #[test]
    fn test_websocket_message_deserialization() {
        let json = r#"{
            "event": "data",
            "channel": "trades",
            "symbol": "2330",
            "data": {"price": 583.0, "size": 1000}
        }"#;
        let msg: WebSocketMessage = serde_json::from_str(json).unwrap();
        assert!(msg.is_data());
        assert_eq!(msg.channel.as_deref(), Some("trades"));
        assert_eq!(msg.symbol.as_deref(), Some("2330"));
    }

    #[test]
    fn test_websocket_error_message() {
        let json = r#"{
            "event": "error",
            "data": {"message": "Unauthorized"}
        }"#;
        let msg: WebSocketMessage = serde_json::from_str(json).unwrap();
        assert!(msg.is_error());
        assert_eq!(msg.error_message(), Some("Unauthorized".to_string()));
    }

    #[test]
    fn test_websocket_authenticated() {
        let json = r#"{"event": "authenticated"}"#;
        let msg: WebSocketMessage = serde_json::from_str(json).unwrap();
        assert!(msg.is_authenticated());
    }

    #[test]
    fn test_auth_request_api_key() {
        let req = AuthRequest::with_api_key("my-api-key");
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"apikey\":\"my-api-key\""));
        assert!(!json.contains("token"));
        assert!(!json.contains("sdkToken"));
    }

    #[test]
    fn test_auth_request_sdk_token() {
        let req = AuthRequest::with_sdk_token("my-sdk-token");
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"sdkToken\":\"my-sdk-token\""));
    }

    #[test]
    fn test_websocket_request_auth() {
        let req = WebSocketRequest::auth(AuthRequest::with_api_key("test"));
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"event\":\"auth\""));
        assert!(json.contains("\"apikey\":\"test\""));
    }

    #[test]
    fn test_websocket_request_subscribe() {
        let req = WebSocketRequest::subscribe(SubscribeRequest::trades("2330"));
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"event\":\"subscribe\""));
        assert!(json.contains("\"channel\":\"trades\""));
    }

    #[test]
    fn test_websocket_request_ping() {
        let req = WebSocketRequest::ping(Some("test-state".to_string()));
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"event\":\"ping\""));
        assert!(json.contains("\"state\":\"test-state\""));
    }
}
