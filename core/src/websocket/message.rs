//! FFI-safe message receiver for WebSocket messages
//!
//! Provides blocking and timeout-based message reception suitable for FFI bindings.
//! Uses std::sync::mpsc (not tokio channels) for compatibility with non-async FFI consumers.

use crate::models::WebSocketMessage;
use crate::websocket::{ConnectionEvent, HealthCheck, SubscriptionManager};
use crate::MarketDataError;
use futures_util::stream::SplitStream;
use futures_util::StreamExt;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tokio_tungstenite::tungstenite::Message;

/// Type alias for WebSocket read half
type WsStream = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

/// FFI-safe message receiver with blocking API
///
/// Thread-safe: Uses Mutex internally to allow sharing across threads.
/// This enables background threads to receive messages while the main
/// thread handles other operations.
///
/// From RESEARCH.md: "FFI 友善的阻塞式 channel"
pub struct MessageReceiver {
    rx: Mutex<mpsc::Receiver<WebSocketMessage>>,
}

impl MessageReceiver {
    /// Create a new message receiver
    pub fn new(rx: mpsc::Receiver<WebSocketMessage>) -> Self {
        Self { rx: Mutex::new(rx) }
    }

    /// Receive a message (blocking)
    ///
    /// Blocks until a message is received or channel is closed.
    ///
    /// # Errors
    ///
    /// Returns `ConnectionError` if channel is closed
    pub fn receive(&self) -> Result<WebSocketMessage, MarketDataError> {
        let rx = self.rx.lock().map_err(|_| MarketDataError::ConnectionError {
            msg: "Message receiver lock poisoned".to_string(),
        })?;
        rx.recv().map_err(|_| MarketDataError::ConnectionError {
            msg: "Message channel closed".to_string(),
        })
    }

    /// Receive a message with timeout
    ///
    /// Returns:
    /// - `Ok(Some(msg))` if message received within timeout
    /// - `Ok(None)` if timeout elapsed with no message
    /// - `Err` if channel closed
    pub fn receive_timeout(
        &self,
        timeout: Duration,
    ) -> Result<Option<WebSocketMessage>, MarketDataError> {
        let rx = self.rx.lock().map_err(|_| MarketDataError::ConnectionError {
            msg: "Message receiver lock poisoned".to_string(),
        })?;
        match rx.recv_timeout(timeout) {
            Ok(msg) => Ok(Some(msg)),
            Err(mpsc::RecvTimeoutError::Timeout) => Ok(None),
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                Err(MarketDataError::ConnectionError {
                    msg: "Message channel closed".to_string(),
                })
            }
        }
    }

    /// Try to receive a message without blocking
    ///
    /// Returns:
    /// - `Some(msg)` if message available
    /// - `None` if no message available or channel closed
    pub fn try_receive(&self) -> Option<WebSocketMessage> {
        self.rx.lock().ok()?.try_recv().ok()
    }
}

/// Dispatch incoming WebSocket messages to appropriate channels
///
/// This task runs in the background after connect() succeeds.
/// It will terminate when:
/// 1. WebSocket connection closes (returns close code)
/// 2. Server sends Close frame (returns close code from frame)
/// 3. WebSocket error occurs (returns None)
/// 4. Message channel closes (returns None)
/// 5. Task is aborted by disconnect() (task cancelled at .await point)
///
/// The function is cancellation-safe: aborting at any `.await` point
/// will not leave resources in an inconsistent state.
///
/// # Arguments
///
/// * `ws_read` - The read half of the WebSocket stream
/// * `message_tx` - Channel to send parsed messages to consumers
/// * `event_tx` - Channel to send connection events
/// * `health_check` - Health check manager for pong tracking
///
/// # Returns
///
/// Close code from the WebSocket close frame, or None if connection
/// was dropped without a proper close or due to an error.
pub(crate) async fn dispatch_messages(
    mut ws_read: WsStream,
    message_tx: mpsc::Sender<WebSocketMessage>,
    event_tx: mpsc::Sender<ConnectionEvent>,
    health_check: Arc<HealthCheck>,
    subscriptions: Arc<SubscriptionManager>,
) -> Option<u16> {
    while let Some(msg_result) = ws_read.next().await {
        match msg_result {
            Ok(Message::Text(text)) => {
                match serde_json::from_str::<WebSocketMessage>(&text) {
                    Ok(ws_msg) => {
                        health_check.touch();
                        // Mutex is only taken when event == "subscribed" (cheap
                        // string compare for every other message).
                        handle_subscribed_event(&subscriptions, &ws_msg);
                        if message_tx.send(ws_msg).is_err() {
                            return None;
                        }
                    }
                    Err(e) => {
                        let _ = event_tx.send(ConnectionEvent::Error {
                            message: format!("Failed to deserialize message: {}", e),
                            code: 2003,
                        });
                    }
                }
            }
            Ok(Message::Binary(data)) => {
                match serde_json::from_slice::<WebSocketMessage>(&data) {
                    Ok(ws_msg) => {
                        health_check.touch();
                        handle_subscribed_event(&subscriptions, &ws_msg);
                        if message_tx.send(ws_msg).is_err() {
                            return None;
                        }
                    }
                    Err(e) => {
                        let _ = event_tx.send(ConnectionEvent::Error {
                            message: format!("Failed to deserialize binary message: {}", e),
                            code: 2003,
                        });
                    }
                }
            }
            Ok(Message::Pong(_)) => {
                // RFC 6455 control-frame pong: count as activity. Fugle
                // sends pong via JSON message; this branch is defensive.
                health_check.touch();
            }
            Ok(Message::Close(close_frame)) => {
                // Server initiated close - RFC 6455 compliant handling
                let code = close_frame.as_ref().map(|cf| cf.code.into());
                let reason = close_frame
                    .as_ref()
                    .map(|cf| cf.reason.to_string())
                    .unwrap_or_else(|| "Server initiated close".to_string());

                // Send disconnected event with close details
                let _ = event_tx.send(ConnectionEvent::Disconnected {
                    code,
                    reason,
                });

                return code;
            }
            Ok(Message::Ping(_)) => {
                // Server sent ping, tokio-tungstenite auto-responds with pong
                // No action needed
            }
            Err(e) => {
                // WebSocket error - connection likely broken
                let _ = event_tx.send(ConnectionEvent::Error {
                    message: format!("WebSocket error: {}", e),
                    code: 2001,
                });
                return None;
            }
            Ok(Message::Frame(_)) => {
                // Raw frames shouldn't appear in normal usage
            }
        }
    }

    // Stream ended without close frame (connection dropped)
    let _ = event_tx.send(ConnectionEvent::Disconnected {
        code: None,
        reason: "Connection closed".to_string(),
    });
    None
}

/// Build the subscription key used by `SubscriptionManager`, mirroring the
/// suffix rules in `SubscribeRequest::key()`. The suffix only appears when
/// the respective flag is explicitly true — matching server ack shapes that
/// may omit the field for regular sessions.
fn build_sub_key(channel: &str, symbol: &str, after_hours: bool, odd_lot: bool) -> String {
    let base = format!("{}:{}", channel, symbol);
    if after_hours {
        format!("{base}:afterhours")
    } else if odd_lot {
        format!("{base}:oddlot")
    } else {
        base
    }
}

/// If `msg` is a `subscribed` ack, record the server-assigned id in the
/// subscription manager. Supports two wire shapes observed in the Fugle
/// protocol:
///   - single: top-level `{event, id, channel, symbol, afterHours?, intradayOddLot?}`
///   - batched: `{event, data: [{id, channel, symbol, afterHours?, intradayOddLot?}, ...]}`
/// Any shape we can't parse is silently ignored — the unsub fallback path
/// (sending the local key as id) keeps the wire format valid even without
/// a recorded server id.
pub(crate) fn handle_subscribed_event(
    subscriptions: &SubscriptionManager,
    msg: &WebSocketMessage,
) {
    if msg.event != "subscribed" {
        return;
    }

    // Batched shape: data is an array of subscription entries.
    if let Some(arr) = msg.data.as_ref().and_then(|d| d.as_array()) {
        for entry in arr {
            let Some(id) = entry.get("id").and_then(|v| v.as_str()) else {
                continue;
            };
            let Some(channel) = entry.get("channel").and_then(|v| v.as_str()) else {
                continue;
            };
            let Some(symbol) = entry.get("symbol").and_then(|v| v.as_str()) else {
                continue;
            };
            let after_hours = entry
                .get("afterHours")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let odd_lot = entry
                .get("intradayOddLot")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            subscriptions.record_server_id(
                build_sub_key(channel, symbol, after_hours, odd_lot),
                id.to_string(),
            );
        }
        return;
    }

    // Single shape: pull fields from data object when present, falling back
    // to the WebSocketMessage top-level fields the model already exposes.
    let data_obj = msg.data.as_ref().and_then(|d| d.as_object());
    let id = data_obj
        .and_then(|d| d.get("id"))
        .and_then(|v| v.as_str())
        .map(String::from)
        .or_else(|| msg.id.clone());
    let channel = data_obj
        .and_then(|d| d.get("channel"))
        .and_then(|v| v.as_str())
        .map(String::from)
        .or_else(|| msg.channel.clone());
    let symbol = data_obj
        .and_then(|d| d.get("symbol"))
        .and_then(|v| v.as_str())
        .map(String::from)
        .or_else(|| msg.symbol.clone());
    let after_hours = data_obj
        .and_then(|d| d.get("afterHours"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let odd_lot = data_obj
        .and_then(|d| d.get("intradayOddLot"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if let (Some(id), Some(channel), Some(symbol)) = (id, channel, symbol) {
        subscriptions.record_server_id(
            build_sub_key(&channel, &symbol, after_hours, odd_lot),
            id,
        );
    }
}

/// Internal ping sender
///
/// Sends WebSocket ping frames when signaled by health check
#[allow(dead_code)] // Will be used when ping support is fully implemented
pub(crate) async fn send_pings(
    mut ws_sink: futures_util::stream::SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    ping_rx: mpsc::Receiver<()>,
) {
    use futures_util::SinkExt;
    while ping_rx.recv().is_ok() {
        if ws_sink.send(Message::Ping(vec![].into())).await.is_err() {
            // Failed to send ping, connection likely closed
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_receive_blocking() {
        let (tx, rx) = mpsc::channel();
        let receiver = MessageReceiver::new(rx);

        // Spawn thread to send message
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(10));
            let msg = WebSocketMessage {
                event: "data".to_string(),
                data: None,
                channel: Some("trades".to_string()),
                symbol: Some("2330".to_string()),
                id: None,
            };
            tx.send(msg).unwrap();
        });

        // Should block and receive
        let result = receiver.receive();
        assert!(result.is_ok());
        let msg = result.unwrap();
        assert_eq!(msg.event, "data");
        assert_eq!(msg.channel, Some("trades".to_string()));
    }

    #[test]
    fn test_receive_timeout_returns_none() {
        let (_tx, rx) = mpsc::channel();
        let receiver = MessageReceiver::new(rx);

        // No message sent, should timeout
        let result = receiver.receive_timeout(Duration::from_millis(50));
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_receive_timeout_returns_message() {
        let (tx, rx) = mpsc::channel();
        let receiver = MessageReceiver::new(rx);

        // Send message immediately
        let msg = WebSocketMessage {
            event: "data".to_string(),
            data: None,
            channel: Some("trades".to_string()),
            symbol: Some("2330".to_string()),
            id: None,
        };
        tx.send(msg).unwrap();

        // Should receive before timeout
        let result = receiver.receive_timeout(Duration::from_secs(1));
        assert!(result.is_ok());
        let received = result.unwrap();
        assert!(received.is_some());
        assert_eq!(received.unwrap().event, "data");
    }

    #[test]
    fn test_try_receive_non_blocking() {
        let (tx, rx) = mpsc::channel();
        let receiver = MessageReceiver::new(rx);

        // No message, should return None immediately
        assert!(receiver.try_receive().is_none());

        // Send message
        let msg = WebSocketMessage {
            event: "data".to_string(),
            data: None,
            channel: None,
            symbol: None,
            id: None,
        };
        tx.send(msg).unwrap();

        // Should receive immediately
        let received = receiver.try_receive();
        assert!(received.is_some());
        assert_eq!(received.unwrap().event, "data");
    }

    #[test]
    fn test_channel_closed_returns_error() {
        let (tx, rx) = mpsc::channel();
        let receiver = MessageReceiver::new(rx);

        // Close channel by dropping sender
        drop(tx);

        // Should return error
        let result = receiver.receive();
        assert!(result.is_err());
        match result {
            Err(MarketDataError::ConnectionError { msg }) => {
                assert!(msg.contains("closed"));
            }
            _ => panic!("Expected ConnectionError"),
        }
    }

    #[test]
    fn test_channel_closed_timeout_returns_error() {
        let (tx, rx) = mpsc::channel();
        let receiver = MessageReceiver::new(rx);

        // Close channel
        drop(tx);

        // Should return error, not timeout
        let result = receiver.receive_timeout(Duration::from_secs(1));
        assert!(result.is_err());
    }

    #[test]
    fn test_try_receive_after_close() {
        let (tx, rx) = mpsc::channel();
        let receiver = MessageReceiver::new(rx);

        // Send message then close
        let msg = WebSocketMessage {
            event: "data".to_string(),
            data: None,
            channel: None,
            symbol: None,
            id: None,
        };
        tx.send(msg).unwrap();
        drop(tx);

        // Should still receive buffered message
        let received = receiver.try_receive();
        assert!(received.is_some());

        // Next try should return None (channel closed, no more messages)
        let received2 = receiver.try_receive();
        assert!(received2.is_none());
    }

    fn parse_msg(json: &str) -> WebSocketMessage {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn test_handle_subscribed_ignores_non_subscribed() {
        let manager = SubscriptionManager::new();
        let msg = parse_msg(
            r#"{"event":"data","id":"sub-1","channel":"trades","symbol":"2330"}"#,
        );
        handle_subscribed_event(&manager, &msg);
        assert!(manager.take_server_id("trades:2330").is_none());
    }

    #[test]
    fn test_handle_subscribed_single_top_level() {
        let manager = SubscriptionManager::new();
        let msg = parse_msg(
            r#"{"event":"subscribed","id":"sub-abc","channel":"trades","symbol":"2330"}"#,
        );
        handle_subscribed_event(&manager, &msg);
        assert_eq!(
            manager.take_server_id("trades:2330"),
            Some("sub-abc".to_string())
        );
    }

    #[test]
    fn test_handle_subscribed_single_with_after_hours() {
        let manager = SubscriptionManager::new();
        let msg = parse_msg(
            r#"{"event":"subscribed","data":{"id":"sub-ah","channel":"books","symbol":"TXFE6","afterHours":true}}"#,
        );
        handle_subscribed_event(&manager, &msg);
        assert_eq!(
            manager.take_server_id("books:TXFE6:afterhours"),
            Some("sub-ah".to_string())
        );
        // Without the suffix it's a different key — mustn't collide.
        assert!(manager.take_server_id("books:TXFE6").is_none());
    }

    #[test]
    fn test_handle_subscribed_single_with_odd_lot() {
        let manager = SubscriptionManager::new();
        let msg = parse_msg(
            r#"{"event":"subscribed","data":{"id":"sub-odd","channel":"trades","symbol":"2330","intradayOddLot":true}}"#,
        );
        handle_subscribed_event(&manager, &msg);
        assert_eq!(
            manager.take_server_id("trades:2330:oddlot"),
            Some("sub-odd".to_string())
        );
    }

    #[test]
    fn test_handle_subscribed_batched_array() {
        let manager = SubscriptionManager::new();
        let msg = parse_msg(
            r#"{"event":"subscribed","data":[
                {"id":"sub-1","channel":"trades","symbol":"2330"},
                {"id":"sub-2","channel":"books","symbol":"TXFE6","afterHours":true},
                {"id":"sub-3","channel":"trades","symbol":"2317","intradayOddLot":true}
            ]}"#,
        );
        handle_subscribed_event(&manager, &msg);
        assert_eq!(manager.take_server_id("trades:2330"), Some("sub-1".into()));
        assert_eq!(
            manager.take_server_id("books:TXFE6:afterhours"),
            Some("sub-2".into())
        );
        assert_eq!(
            manager.take_server_id("trades:2317:oddlot"),
            Some("sub-3".into())
        );
    }

    #[test]
    fn test_handle_subscribed_missing_fields_no_op() {
        let manager = SubscriptionManager::new();
        // No id, no channel — nothing to record.
        let msg = parse_msg(r#"{"event":"subscribed","symbol":"2330"}"#);
        handle_subscribed_event(&manager, &msg);
        assert!(manager.take_server_id("trades:2330").is_none());
    }
}
