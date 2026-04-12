//! WebSocket client with foreign trait callbacks for UniFFI bindings
//!
//! This module provides a WebSocket client that delivers typed StreamMessage events
//! to foreign language consumers (C#, Go) via the WebSocketListener trait.
//!
//! # Architecture
//!
//! ```text
//! Foreign Code (C#/Go)                 Rust (UniFFI)
//! ┌─────────────────────┐              ┌─────────────────────┐
//! │ class MyListener    │              │ WebSocketClient     │
//! │   implements        │──callback────│   spawns message    │
//! │   IWebSocketListener│              │   forwarding task   │
//! │                     │              │                     │
//! │ OnMessage(msg) ◄────│──────────────│ run_message_loop()  │
//! │ OnConnected()  ◄────│              │                     │
//! │ OnDisconnected()◄───│              │ CoreWebSocketClient │
//! │ OnError(err)   ◄────│              │   .messages()       │
//! └─────────────────────┘              └─────────────────────┘
//! ```
//!
//! # Thread Safety
//!
//! The `WebSocketListener` trait requires `Send + Sync` for thread-safe
//! callback invocation. Foreign implementations must be thread-safe.

use crate::errors::MarketDataError;
use crate::models::StreamMessage;
use marketdata_core::websocket::{
    ConnectionConfig, MessageReceiver, WebSocketClient as CoreWebSocketClient,
};
use marketdata_core::AuthRequest;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Callback interface for WebSocket events
///
/// Foreign code (C#, Go) implements this trait to receive WebSocket events.
/// The implementation must be thread-safe (Send + Sync) as callbacks may be
/// invoked from background tokio tasks.
///
/// # Example (C#)
///
/// ```csharp
/// class MyListener : IWebSocketListener {
///     public void OnConnected() {
///         Console.WriteLine("Connected!");
///     }
///     public void OnDisconnected() {
///         Console.WriteLine("Disconnected");
///     }
///     public void OnMessage(StreamMessage message) {
///         Console.WriteLine($"Got {message.Event} for {message.Symbol}");
///     }
///     public void OnError(string errorMessage) {
///         Console.WriteLine($"Error: {errorMessage}");
///     }
/// }
/// ```
#[uniffi::export(with_foreign)]
pub trait WebSocketListener: Send + Sync {
    /// Called when WebSocket connection is established
    fn on_connected(&self);

    /// Called when WebSocket connection is closed
    fn on_disconnected(&self);

    /// Called when a message is received
    fn on_message(&self, message: StreamMessage);

    /// Called when an error occurs
    fn on_error(&self, error_message: String);

    /// Called when a reconnection attempt starts
    fn on_reconnecting(&self, attempt: u32);

    /// Called when all reconnection attempts are exhausted
    fn on_reconnect_failed(&self, attempts: u32);
}

/// Reconnection configuration record for FFI
///
/// All fields are optional — zero/false values mean "use default".
#[derive(Debug, Clone, uniffi::Record)]
pub struct ReconnectConfigRecord {
    /// Maximum reconnection attempts (default: 5, min: 1)
    pub max_attempts: u32,
    /// Initial reconnection delay in milliseconds (default: 1000, min: 100)
    pub initial_delay_ms: u64,
    /// Maximum reconnection delay in milliseconds (default: 60000)
    pub max_delay_ms: u64,
}

impl ReconnectConfigRecord {
    fn to_core(&self) -> marketdata_core::ReconnectionConfig {
        let mut cfg = marketdata_core::ReconnectionConfig::default();
        if self.max_attempts > 0 {
            cfg.max_attempts = self.max_attempts;
        }
        if self.initial_delay_ms > 0 {
            cfg.initial_delay = std::time::Duration::from_millis(self.initial_delay_ms);
        }
        if self.max_delay_ms > 0 {
            cfg.max_delay = std::time::Duration::from_millis(self.max_delay_ms);
        }
        cfg
    }
}

/// Health check configuration record for FFI
///
/// All fields are optional — zero/false values mean "use default".
#[derive(Debug, Clone, uniffi::Record)]
pub struct HealthCheckConfigRecord {
    /// Whether health check is enabled (default: false)
    pub enabled: bool,
    /// Interval between ping messages in milliseconds (default: 30000, min: 5000)
    pub interval_ms: u64,
    /// Maximum missed pongs before disconnect (default: 2, min: 1)
    pub max_missed_pongs: u64,
}

impl HealthCheckConfigRecord {
    fn to_core(&self) -> marketdata_core::HealthCheckConfig {
        let mut cfg = marketdata_core::HealthCheckConfig::default();
        cfg.enabled = self.enabled;
        if self.interval_ms > 0 {
            cfg.interval = std::time::Duration::from_millis(self.interval_ms);
        }
        if self.max_missed_pongs > 0 {
            cfg.max_missed_pongs = self.max_missed_pongs;
        }
        cfg
    }
}

/// Endpoint type for WebSocket connection
#[derive(Debug, Clone, Copy, uniffi::Enum)]
pub enum WebSocketEndpoint {
    /// Stock market data endpoint
    Stock,
    /// Futures and options market data endpoint
    FutOpt,
}

/// WebSocket client for real-time market data streaming
///
/// Wraps the core WebSocketClient and forwards messages to the provided
/// WebSocketListener implementation via a background task.
#[derive(uniffi::Object)]
pub struct WebSocketClient {
    inner: Arc<Mutex<Option<CoreWebSocketClient>>>,
    listener: Arc<dyn WebSocketListener>,
    api_key: String,
    endpoint: WebSocketEndpoint,
    connected: Arc<AtomicBool>,
    shutdown: Arc<AtomicBool>,
    reconnect_config: Option<marketdata_core::ReconnectionConfig>,
    health_check_config: Option<marketdata_core::HealthCheckConfig>,
}

impl WebSocketClient {
    /// Create a new WebSocket client (internal constructor)
    fn new_internal(
        api_key: String,
        listener: Arc<dyn WebSocketListener>,
        endpoint: WebSocketEndpoint,
        reconnect_config: Option<marketdata_core::ReconnectionConfig>,
        health_check_config: Option<marketdata_core::HealthCheckConfig>,
    ) -> Arc<Self> {
        Arc::new(Self {
            inner: Arc::new(Mutex::new(None)),
            listener,
            api_key,
            endpoint,
            connected: Arc::new(AtomicBool::new(false)),
            shutdown: Arc::new(AtomicBool::new(false)),
            reconnect_config,
            health_check_config,
        })
    }
}

#[uniffi::export]
impl WebSocketClient {
    /// Create a new WebSocket client for stock market data
    ///
    /// # Arguments
    /// * `api_key` - Fugle API key for authentication
    /// * `listener` - Callback interface for receiving WebSocket events
    #[uniffi::constructor]
    pub fn new(api_key: String, listener: Arc<dyn WebSocketListener>) -> Arc<Self> {
        Self::new_internal(api_key, listener, WebSocketEndpoint::Stock, None, None)
    }

    /// Create a new WebSocket client for a specific endpoint
    ///
    /// # Arguments
    /// * `api_key` - Fugle API key for authentication
    /// * `listener` - Callback interface for receiving WebSocket events
    /// * `endpoint` - The market data endpoint (Stock or FutOpt)
    #[uniffi::constructor]
    pub fn new_with_endpoint(
        api_key: String,
        listener: Arc<dyn WebSocketListener>,
        endpoint: WebSocketEndpoint,
    ) -> Arc<Self> {
        Self::new_internal(api_key, listener, endpoint, None, None)
    }

    /// Create a new WebSocket client with full configuration
    ///
    /// # Arguments
    /// * `api_key` - Fugle API key for authentication
    /// * `listener` - Callback interface for receiving WebSocket events
    /// * `endpoint` - The market data endpoint (Stock or FutOpt)
    /// * `reconnect_config` - Optional reconnection configuration
    /// * `health_check_config` - Optional health check configuration
    #[uniffi::constructor]
    pub fn new_with_config(
        api_key: String,
        listener: Arc<dyn WebSocketListener>,
        endpoint: WebSocketEndpoint,
        reconnect_config: Option<ReconnectConfigRecord>,
        health_check_config: Option<HealthCheckConfigRecord>,
    ) -> Arc<Self> {
        Self::new_internal(
            api_key,
            listener,
            endpoint,
            reconnect_config.map(|c| c.to_core()),
            health_check_config.map(|c| c.to_core()),
        )
    }

    /// Check if the client is currently connected
    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::SeqCst)
    }

    /// Check if the client has been shut down
    pub fn is_closed(&self) -> bool {
        self.shutdown.load(Ordering::SeqCst)
    }
}

#[uniffi::export(async_runtime = "tokio")]
impl WebSocketClient {
    /// Connect to the WebSocket server
    ///
    /// Establishes connection, authenticates, and starts a background task
    /// to forward messages to the listener.
    ///
    /// # Errors
    ///
    /// Returns error if connection or authentication fails.
    pub async fn connect(&self) -> Result<(), MarketDataError> {
        // Create auth request
        let auth = AuthRequest::with_api_key(&self.api_key);

        // Create connection config based on endpoint
        let config = match self.endpoint {
            WebSocketEndpoint::Stock => ConnectionConfig::fugle_stock(auth),
            WebSocketEndpoint::FutOpt => ConnectionConfig::fugle_futopt(auth),
        };

        // Create core WebSocket client with optional reconnection/health-check config
        let core_ws = if let (Some(rc), Some(hc)) = (&self.reconnect_config, &self.health_check_config) {
            CoreWebSocketClient::with_full_config(config, rc.clone(), hc.clone())
        } else if let Some(rc) = &self.reconnect_config {
            CoreWebSocketClient::with_full_config(
                config,
                rc.clone(),
                marketdata_core::HealthCheckConfig::default(),
            )
        } else if let Some(hc) = &self.health_check_config {
            CoreWebSocketClient::with_full_config(
                config,
                marketdata_core::ReconnectionConfig::default(),
                hc.clone(),
            )
        } else {
            CoreWebSocketClient::new(config)
        };

        // Connect to server
        core_ws.connect().await?;

        // CRITICAL: Obtain message receiver from core WebSocket API
        // The core WebSocket client exposes messages via client.messages() method
        // which returns Arc<MessageReceiver>
        let receiver: Arc<MessageReceiver> = core_ws.messages();

        // Capture state events receiver BEFORE storing core_ws (move would lose access)
        let events = Arc::clone(core_ws.state_events());

        // Store client in inner
        {
            let mut guard = self.inner.lock().await;
            *guard = Some(core_ws);
        }

        // Update connected state
        self.connected.store(true, Ordering::SeqCst);

        // Reset shutdown flag for this connection
        self.shutdown.store(false, Ordering::SeqCst);

        // Notify listener
        self.listener.on_connected();

        // Spawn task to forward messages to listener
        let listener = Arc::clone(&self.listener);
        let shutdown = Arc::clone(&self.shutdown);
        let connected = Arc::clone(&self.connected);
        tokio::spawn(async move {
            run_message_loop(receiver, listener, shutdown, connected).await;
        });

        // Spawn thread to monitor state events and forward to listener
        let event_listener = Arc::clone(&self.listener);
        let event_connected = Arc::clone(&self.connected);
        let event_shutdown = Arc::clone(&self.shutdown);
        std::thread::Builder::new()
            .name("ws_event_monitor".to_string())
            .spawn(move || {
                loop {
                    if event_shutdown.load(Ordering::SeqCst) {
                        break;
                    }
                    let event = {
                        let rx = events.blocking_lock();
                        rx.recv()
                    };
                    match event {
                        Ok(event) => {
                            use marketdata_core::websocket::ConnectionEvent;
                            match event {
                                ConnectionEvent::Reconnecting { attempt } => {
                                    event_listener.on_reconnecting(attempt);
                                }
                                ConnectionEvent::ReconnectFailed { attempts } => {
                                    event_listener.on_reconnect_failed(attempts);
                                    event_connected.store(false, Ordering::SeqCst);
                                }
                                ConnectionEvent::Error { message, .. } => {
                                    event_listener.on_error(message);
                                }
                                ConnectionEvent::Disconnected { .. } => {
                                    event_listener.on_disconnected();
                                    event_connected.store(false, Ordering::SeqCst);
                                }
                                ConnectionEvent::Authenticated => {
                                    event_connected.store(true, Ordering::SeqCst);
                                }
                                // Map Unauthenticated to on_error so existing UniFFI
                                // listeners (Java/Go/C#) get notified of credential
                                // rejection without needing a new trait method.
                                // To expose a dedicated callback, add `on_unauthenticated`
                                // to the WebSocketListener trait and re-run uniffi-bindgen.
                                ConnectionEvent::Unauthenticated { message } => {
                                    event_listener.on_error(format!(
                                        "Unauthenticated: {}",
                                        message
                                    ));
                                }
                                _ => {}
                            }
                        }
                        Err(_) => break, // Channel closed
                    }
                }
            })
            .ok();

        Ok(())
    }

    /// Subscribe to a channel for a symbol
    ///
    /// # Arguments
    /// * `channel` - Channel name (e.g., "trades", "candles", "books")
    /// * `symbol` - Symbol to subscribe (e.g., "2330")
    ///
    /// # Errors
    ///
    /// Returns error if not connected or subscription fails.
    pub async fn subscribe(&self, channel: String, symbol: String) -> Result<(), MarketDataError> {
        let guard = self.inner.lock().await;
        if let Some(ref ws) = *guard {
            use marketdata_core::models::{Channel, SubscribeRequest};

            // Parse channel string to Channel enum
            let channel_enum = match channel.as_str() {
                "trades" => Channel::Trades,
                "candles" => Channel::Candles,
                "books" => Channel::Books,
                "aggregates" => Channel::Aggregates,
                "indices" => Channel::Indices,
                _ => {
                    return Err(MarketDataError::ConfigError {
                        msg: format!("Unknown channel: {}", channel),
                    })
                }
            };

            let req = SubscribeRequest::new(channel_enum, &symbol);
            ws.subscribe(req).await?;
            Ok(())
        } else {
            Err(MarketDataError::WebSocketError {
                msg: "Not connected".to_string(),
            })
        }
    }

    /// Unsubscribe from a channel for a symbol
    ///
    /// # Arguments
    /// * `channel` - Channel name
    /// * `symbol` - Symbol to unsubscribe
    ///
    /// # Errors
    ///
    /// Returns error if not connected.
    pub async fn unsubscribe(&self, channel: String, symbol: String) -> Result<(), MarketDataError> {
        let guard = self.inner.lock().await;
        if let Some(ref ws) = *guard {
            // Build key for unsubscribe
            let key = format!("{}:{}", channel, symbol);
            ws.unsubscribe(&key).await?;
            Ok(())
        } else {
            Err(MarketDataError::WebSocketError {
                msg: "Not connected".to_string(),
            })
        }
    }

    /// Send a ping message to the server
    ///
    /// # Arguments
    /// * `state` - Optional state string echoed back in the pong response
    pub async fn ping(&self, state: Option<String>) -> Result<(), MarketDataError> {
        let guard = self.inner.lock().await;
        if let Some(ref ws) = *guard {
            let request = marketdata_core::WebSocketRequest::ping(state);
            ws.send(request).await?;
            Ok(())
        } else {
            Err(MarketDataError::WebSocketError {
                msg: "Not connected".to_string(),
            })
        }
    }

    /// Query the server for current subscriptions
    pub async fn query_subscriptions(&self) -> Result<(), MarketDataError> {
        let guard = self.inner.lock().await;
        if let Some(ref ws) = *guard {
            let request = marketdata_core::WebSocketRequest::subscriptions();
            ws.send(request).await?;
            Ok(())
        } else {
            Err(MarketDataError::WebSocketError {
                msg: "Not connected".to_string(),
            })
        }
    }

    /// Disconnect from the WebSocket server
    ///
    /// Gracefully closes the connection and stops the message forwarding task.
    pub async fn disconnect(&self) {
        // Signal shutdown to message loop
        self.shutdown.store(true, Ordering::SeqCst);

        // Take and disconnect the client
        let mut guard = self.inner.lock().await;
        if let Some(ws) = guard.take() {
            let _ = ws.disconnect().await;
        }

        // Update connected state
        self.connected.store(false, Ordering::SeqCst);

        // Notify listener
        self.listener.on_disconnected();
    }
}

/// Background task that forwards messages from core MessageReceiver to listener
///
/// The core WebSocket exposes `pub fn messages(&self) -> Arc<MessageReceiver>`
/// (see core/src/websocket/connection.rs:279). MessageReceiver has blocking
/// `receive()` method, so we use spawn_blocking for async context.
async fn run_message_loop(
    receiver: Arc<MessageReceiver>,
    listener: Arc<dyn WebSocketListener>,
    shutdown: Arc<AtomicBool>,
    connected: Arc<AtomicBool>,
) {
    use std::time::Duration;

    loop {
        // Check for shutdown signal
        if shutdown.load(Ordering::SeqCst) {
            break;
        }

        // Use spawn_blocking since MessageReceiver::receive_timeout() is blocking
        let receiver_clone = Arc::clone(&receiver);
        let result = tokio::task::spawn_blocking(move || {
            // Use timeout to allow periodic shutdown checks
            receiver_clone.receive_timeout(Duration::from_millis(100))
        })
        .await;

        match result {
            Ok(Ok(Some(ws_msg))) => {
                // Convert core WebSocketMessage to UniFFI StreamMessage
                let stream_msg = StreamMessage::from(ws_msg);
                listener.on_message(stream_msg);
            }
            Ok(Ok(None)) => {
                // Timeout, continue loop (allows shutdown check)
                continue;
            }
            Ok(Err(e)) => {
                // Channel closed or error
                listener.on_error(e.to_string());
                break;
            }
            Err(e) => {
                // Task join error
                listener.on_error(format!("Task join error: {}", e));
                break;
            }
        }
    }

    // Update connected state
    connected.store(false, Ordering::SeqCst);

    // Only call on_disconnected if not already called by disconnect()
    if !shutdown.load(Ordering::SeqCst) {
        listener.on_disconnected();
    }
}

/// Create a new WebSocket client for stock market data
///
/// # Arguments
/// * `api_key` - Fugle API key for authentication
/// * `listener` - Callback interface for receiving WebSocket events
///
/// # Returns
/// A WebSocketClient instance wrapped in Arc for thread-safe access
#[uniffi::export]
pub fn new_websocket_client(
    api_key: String,
    listener: Arc<dyn WebSocketListener>,
) -> Arc<WebSocketClient> {
    WebSocketClient::new(api_key, listener)
}

/// Create a new WebSocket client for a specific endpoint
///
/// # Arguments
/// * `api_key` - Fugle API key for authentication
/// * `listener` - Callback interface for receiving WebSocket events
/// * `endpoint` - The market data endpoint (Stock or FutOpt)
///
/// # Returns
/// A WebSocketClient instance wrapped in Arc for thread-safe access
#[uniffi::export]
pub fn new_websocket_client_with_endpoint(
    api_key: String,
    listener: Arc<dyn WebSocketListener>,
    endpoint: WebSocketEndpoint,
) -> Arc<WebSocketClient> {
    WebSocketClient::new_with_endpoint(api_key, listener, endpoint)
}

/// Create a new WebSocket client with full configuration
///
/// # Arguments
/// * `api_key` - Fugle API key for authentication
/// * `listener` - Callback interface for receiving WebSocket events
/// * `endpoint` - The market data endpoint (Stock or FutOpt)
/// * `reconnect_config` - Optional reconnection configuration
/// * `health_check_config` - Optional health check configuration
///
/// # Returns
/// A WebSocketClient instance wrapped in Arc for thread-safe access
#[uniffi::export]
pub fn new_websocket_client_with_config(
    api_key: String,
    listener: Arc<dyn WebSocketListener>,
    endpoint: WebSocketEndpoint,
    reconnect_config: Option<ReconnectConfigRecord>,
    health_check_config: Option<HealthCheckConfigRecord>,
) -> Arc<WebSocketClient> {
    WebSocketClient::new_with_config(api_key, listener, endpoint, reconnect_config, health_check_config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Test listener that tracks callback invocations
    struct TestListener {
        connected_count: AtomicUsize,
        disconnected_count: AtomicUsize,
        message_count: AtomicUsize,
        error_count: AtomicUsize,
        reconnecting_count: AtomicUsize,
        reconnect_failed_count: AtomicUsize,
        last_error: Mutex<Option<String>>,
    }

    impl TestListener {
        fn new() -> Self {
            Self {
                connected_count: AtomicUsize::new(0),
                disconnected_count: AtomicUsize::new(0),
                message_count: AtomicUsize::new(0),
                error_count: AtomicUsize::new(0),
                reconnecting_count: AtomicUsize::new(0),
                reconnect_failed_count: AtomicUsize::new(0),
                last_error: Mutex::new(None),
            }
        }
    }

    impl WebSocketListener for TestListener {
        fn on_connected(&self) {
            self.connected_count.fetch_add(1, Ordering::SeqCst);
        }

        fn on_disconnected(&self) {
            self.disconnected_count.fetch_add(1, Ordering::SeqCst);
        }

        fn on_message(&self, _message: StreamMessage) {
            self.message_count.fetch_add(1, Ordering::SeqCst);
        }

        fn on_error(&self, error_message: String) {
            self.error_count.fetch_add(1, Ordering::SeqCst);
            if let Ok(mut guard) = self.last_error.lock() {
                *guard = Some(error_message);
            }
        }

        fn on_reconnecting(&self, _attempt: u32) {
            self.reconnecting_count.fetch_add(1, Ordering::SeqCst);
        }

        fn on_reconnect_failed(&self, _attempts: u32) {
            self.reconnect_failed_count.fetch_add(1, Ordering::SeqCst);
        }
    }

    // Use std::sync::Mutex for tests instead of tokio::sync::Mutex
    use std::sync::Mutex;

    #[test]
    fn test_websocket_client_creation() {
        let listener = Arc::new(TestListener::new());
        let client = WebSocketClient::new("test-key".to_string(), listener);
        assert!(!client.is_connected());
    }

    #[test]
    fn test_websocket_client_with_endpoint() {
        let listener = Arc::new(TestListener::new());
        let client = WebSocketClient::new_with_endpoint(
            "test-key".to_string(),
            listener,
            WebSocketEndpoint::FutOpt,
        );
        assert!(!client.is_connected());
    }

    #[test]
    fn test_websocket_listener_receives_message() {
        // This test verifies the callback wiring works
        let listener = Arc::new(TestListener::new());

        // Simulate calling on_message
        let test_msg = StreamMessage {
            event: "data".to_string(),
            channel: Some("trades".to_string()),
            symbol: Some("2330".to_string()),
            id: None,
            data_json: Some("{}".to_string()),
            error_code: None,
            error_message: None,
        };
        listener.on_message(test_msg);

        assert_eq!(
            listener.message_count.load(Ordering::SeqCst),
            1,
            "on_message callback should have been invoked"
        );
    }

    #[test]
    fn test_websocket_listener_lifecycle_callbacks() {
        let listener = Arc::new(TestListener::new());

        // Simulate connection lifecycle
        listener.on_connected();
        assert_eq!(listener.connected_count.load(Ordering::SeqCst), 1);

        listener.on_disconnected();
        assert_eq!(listener.disconnected_count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_websocket_listener_error_callback() {
        let listener = Arc::new(TestListener::new());

        listener.on_error("Test error".to_string());
        assert_eq!(listener.error_count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_factory_functions() {
        let listener: Arc<dyn WebSocketListener> = Arc::new(TestListener::new());
        let _client = new_websocket_client("test-key".to_string(), Arc::clone(&listener));

        let listener2: Arc<dyn WebSocketListener> = Arc::new(TestListener::new());
        let _client2 = new_websocket_client_with_endpoint(
            "test-key".to_string(),
            listener2,
            WebSocketEndpoint::Stock,
        );
    }
}
