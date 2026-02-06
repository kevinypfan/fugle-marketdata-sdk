//! WebSocket client wrapper for JavaScript
//!
//! This module provides JavaScript-facing WebSocket clients using ThreadsafeFunction
//! for cross-thread callback invocation without blocking the Node.js event loop.
//!
//! Architecture:
//! - WebSocket connection runs in a dedicated background thread with its own tokio runtime
//! - Commands (connect, subscribe, disconnect) are sent via crossbeam channel
//! - Events (message, connect, disconnect, error) are delivered via ThreadsafeFunction callbacks

use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi_derive::napi;
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::Duration;

/// Type alias for JavaScript callback
/// In napi-rs 3.x, ThreadsafeFunction uses const generics instead of ErrorStrategy type.
/// Default CalleeHandled = true means the callee (JS function) handles errors.
/// We use Arc<ThreadsafeFunction> to allow cloning for use across threads.
pub type JsCallback = Arc<ThreadsafeFunction<String>>;

/// Reconnection options for WebSocket clients
///
/// All fields are optional - defaults are applied when not specified:
/// - maxAttempts: 5
/// - initialDelayMs: 1000
/// - maxDelayMs: 60000
#[napi(object)]
#[derive(Debug, Clone, Default)]
pub struct ReconnectOptions {
    /// Maximum reconnection attempts (default: 5, min: 1)
    pub max_attempts: Option<u32>,
    /// Initial reconnection delay in milliseconds (default: 1000, min: 100)
    pub initial_delay_ms: Option<f64>,
    /// Maximum reconnection delay in milliseconds (default: 60000)
    pub max_delay_ms: Option<f64>,
}

/// Health check options for WebSocket connections
///
/// All fields are optional - defaults are applied when not specified:
/// - enabled: false
/// - intervalMs: 30000
/// - maxMissedPongs: 2
#[napi(object)]
#[derive(Debug, Clone, Default)]
pub struct HealthCheckOptions {
    /// Whether health check is enabled (default: false)
    pub enabled: Option<bool>,
    /// Interval between ping messages in milliseconds (default: 30000, min: 5000)
    pub interval_ms: Option<f64>,
    /// Maximum missed pongs before disconnect (default: 2, min: 1)
    pub max_missed_pongs: Option<f64>,
}

/// REST client options
///
/// Exactly ONE of apiKey, bearerToken, or sdkToken must be provided.
/// baseUrl is optional for custom endpoint override.
#[napi(object)]
#[derive(Debug, Clone, Default)]
pub struct RestClientOptions {
    /// API key for authentication
    pub api_key: Option<String>,
    /// Bearer token for authentication
    pub bearer_token: Option<String>,
    /// SDK token for authentication
    pub sdk_token: Option<String>,
    /// Override base URL (optional)
    pub base_url: Option<String>,
}

/// WebSocket client options
///
/// Exactly ONE of apiKey, bearerToken, or sdkToken must be provided.
/// reconnect and healthCheck are optional configuration objects.
#[napi(object)]
#[derive(Debug, Clone, Default)]
pub struct WebSocketClientOptions {
    /// API key for authentication
    pub api_key: Option<String>,
    /// Bearer token for authentication
    pub bearer_token: Option<String>,
    /// SDK token for authentication
    pub sdk_token: Option<String>,
    /// Override base URL (optional)
    pub base_url: Option<String>,
    /// Reconnection configuration (optional)
    pub reconnect: Option<ReconnectOptions>,
    /// Health check configuration (optional)
    pub health_check: Option<HealthCheckOptions>,
}

/// Command sent to WebSocket worker thread
#[derive(Debug)]
enum WsCommand {
    Subscribe { channel: String, symbol: String, extra: Option<bool> },
    Unsubscribe { id: String },
    Disconnect,
}

/// Callback storage for event handlers
struct EventCallbacks {
    message: Option<JsCallback>,
    connect: Option<JsCallback>,
    disconnect: Option<JsCallback>,
    reconnect: Option<JsCallback>,
    error: Option<JsCallback>,
}

impl Default for EventCallbacks {
    fn default() -> Self {
        Self {
            message: None,
            connect: None,
            disconnect: None,
            reconnect: None,
            error: None,
        }
    }
}

/// WebSocket client for real-time market data (JavaScript wrapper)
///
/// # JavaScript Usage
///
/// ```javascript
/// const { WebSocketClient } = require('@fubon/marketdata-js');
///
/// // Create client with API key
/// const ws = new WebSocketClient('your-api-key');
///
/// // Register event handlers for stock data
/// ws.stock.on('message', (data) => console.log(JSON.parse(data)));
/// ws.stock.on('connect', () => console.log('Connected!'));
/// ws.stock.on('error', (err) => console.error(err));
///
/// // Connect and subscribe
/// ws.stock.connect();
/// ws.stock.subscribe({ channel: 'trades', symbol: '2330' });
/// ```
#[napi]
pub struct WebSocketClient {
    api_key: String,
}

#[napi]
impl WebSocketClient {
    /// Create a new WebSocket client with API key authentication
    ///
    /// @param apiKey - Your Fugle API key
    #[napi(constructor)]
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }

    /// Get the stock WebSocket client for real-time stock data
    #[napi(getter)]
    pub fn stock(&self) -> StockWebSocketClient {
        StockWebSocketClient::new(self.api_key.clone())
    }

    /// Get the FutOpt WebSocket client for real-time futures/options data
    #[napi(getter)]
    pub fn futopt(&self) -> FutOptWebSocketClient {
        FutOptWebSocketClient::new(self.api_key.clone())
    }
}

/// Stock WebSocket client for real-time stock market data
///
/// # JavaScript Usage
///
/// ```javascript
/// // Event handlers
/// ws.stock.on('message', (data) => {
///   const msg = JSON.parse(data);
///   console.log(msg);
/// });
/// ws.stock.on('connect', () => console.log('Stock WebSocket connected'));
/// ws.stock.on('disconnect', (reason) => console.log('Disconnected:', reason));
/// ws.stock.on('reconnect', (info) => console.log('Reconnecting:', info));
/// ws.stock.on('error', (err) => console.error('Error:', err));
///
/// // Connect
/// ws.stock.connect();
///
/// // Subscribe to channels
/// ws.stock.subscribe({ channel: 'trades', symbol: '2330' });
/// ws.stock.subscribe({ channel: 'candles', symbol: '2330' });
/// ```
#[napi]
pub struct StockWebSocketClient {
    api_key: String,
    callbacks: Arc<Mutex<EventCallbacks>>,
    connected: Arc<AtomicBool>,
    closed: Arc<AtomicBool>,
    command_tx: Arc<Mutex<Option<std::sync::mpsc::Sender<WsCommand>>>>,
}

#[napi]
impl StockWebSocketClient {
    /// Create a new stock WebSocket client (internal use)
    fn new(api_key: String) -> Self {
        Self {
            api_key,
            callbacks: Arc::new(Mutex::new(EventCallbacks::default())),
            connected: Arc::new(AtomicBool::new(false)),
            closed: Arc::new(AtomicBool::new(false)),
            command_tx: Arc::new(Mutex::new(None)),
        }
    }

    /// Register an event handler
    ///
    /// @param event - Event type: "message", "connect", "disconnect", "reconnect", "error"
    /// @param callback - JavaScript callback function receiving string data
    ///
    /// @example
    /// ```javascript
    /// ws.stock.on('message', (data) => console.log(data));
    /// ws.stock.on('connect', () => console.log('Connected'));
    /// ws.stock.on('error', (err) => console.error(err));
    /// ```
    #[napi(ts_args_type = "event: WebSocketEvent, callback: (data: string) => void")]
    pub fn on(&self, event: String, callback: ThreadsafeFunction<String>) -> napi::Result<()> {
        let mut callbacks = self
            .callbacks
            .lock()
            .map_err(|e| napi::Error::from_reason(format!("Lock error: {}", e)))?;

        // Wrap in Arc for thread-safe sharing (napi-rs 3.x pattern)
        let arc_callback = Arc::new(callback);

        match event.as_str() {
            "message" => callbacks.message = Some(arc_callback),
            "connect" => callbacks.connect = Some(arc_callback),
            "disconnect" => callbacks.disconnect = Some(arc_callback),
            "reconnect" => callbacks.reconnect = Some(arc_callback),
            "error" => callbacks.error = Some(arc_callback),
            _ => {
                return Err(napi::Error::from_reason(format!(
                    "Unknown event type: {}. Valid events: message, connect, disconnect, reconnect, error",
                    event
                )))
            }
        }
        Ok(())
    }

    /// Connect to the stock WebSocket server
    ///
    /// This method spawns a background thread that manages the WebSocket connection.
    /// Connection result will be delivered via 'connect' or 'error' callbacks.
    #[napi]
    pub fn connect(&self) -> napi::Result<()> {
        // Create command channel
        let (cmd_tx, cmd_rx) = std::sync::mpsc::channel::<WsCommand>();

        // Store command sender
        {
            let mut tx_guard = self.command_tx.lock().map_err(|e| {
                napi::Error::from_reason(format!("Lock error: {}", e))
            })?;
            *tx_guard = Some(cmd_tx);
        }

        // Clone data for the worker thread
        let api_key = self.api_key.clone();
        let callbacks = Arc::clone(&self.callbacks);
        let connected = Arc::clone(&self.connected);
        let closed = Arc::clone(&self.closed);

        // Spawn worker thread that owns WebSocketClient
        thread::Builder::new()
            .name("stock_ws_worker".to_string())
            .spawn(move || {
                use marketdata_core::websocket::{ConnectionConfig, WebSocketClient as CoreClient};
                use marketdata_core::AuthRequest;
                use marketdata_core::models::Channel;
                use marketdata_core::websocket::channels::StockSubscription;

                // Create tokio runtime
                let rt = match tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                {
                    Ok(rt) => rt,
                    Err(e) => {
                        fire_callback(&callbacks, "error", format!("Failed to create runtime: {}", e));
                        return;
                    }
                };

                // Create WebSocket client with stock endpoint
                let config = ConnectionConfig::fugle_stock(AuthRequest::with_api_key(&api_key));
                let client = CoreClient::new(config);

                // Connect
                let connect_result = rt.block_on(async {
                    client.connect().await
                });

                if let Err(e) = connect_result {
                    fire_callback(&callbacks, "error", format!("[{}] {}", e.to_error_code(), e));
                    return;
                }

                // Mark as connected
                connected.store(true, Ordering::SeqCst);
                fire_callback(&callbacks, "connect", "connected".to_string());

                // Get message receiver
                let receiver = client.messages();

                // Main event loop
                loop {
                    // Check for commands (non-blocking)
                    match cmd_rx.try_recv() {
                        Ok(WsCommand::Subscribe { channel, symbol, extra }) => {
                            let channel_enum = match channel.to_lowercase().as_str() {
                                "trades" => Channel::Trades,
                                "candles" => Channel::Candles,
                                "books" => Channel::Books,
                                "aggregates" => Channel::Aggregates,
                                "indices" => Channel::Indices,
                                _ => continue,
                            };
                            let odd_lot = extra.unwrap_or(false);
                            let sub = StockSubscription::new(channel_enum, &symbol).with_odd_lot(odd_lot);
                            let _ = rt.block_on(client.subscribe_channel(sub));
                        }
                        Ok(WsCommand::Unsubscribe { id }) => {
                            let _ = rt.block_on(client.unsubscribe_by_id(&id));
                        }
                        Ok(WsCommand::Disconnect) => {
                            let _ = rt.block_on(client.disconnect());
                            connected.store(false, Ordering::SeqCst);
                            closed.store(true, Ordering::SeqCst);
                            fire_callback(&callbacks, "disconnect", "disconnected".to_string());
                            break;
                        }
                        Err(std::sync::mpsc::TryRecvError::Empty) => {}
                        Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                            // Command channel closed, cleanup
                            let _ = rt.block_on(client.disconnect());
                            connected.store(false, Ordering::SeqCst);
                            closed.store(true, Ordering::SeqCst);
                            break;
                        }
                    }

                    // Check for messages (with timeout)
                    match receiver.receive_timeout(Duration::from_millis(50)) {
                        Ok(Some(msg)) => {
                            if let Ok(json_str) = serde_json::to_string(&msg) {
                                fire_callback(&callbacks, "message", json_str);
                            }
                        }
                        Ok(None) => {
                            // Timeout, continue loop
                        }
                        Err(_) => {
                            // Channel closed
                            connected.store(false, Ordering::SeqCst);
                            closed.store(true, Ordering::SeqCst);
                            fire_callback(&callbacks, "disconnect", "channel_closed".to_string());
                            break;
                        }
                    }
                }
            })
            .map_err(|e| napi::Error::from_reason(format!("Failed to spawn worker thread: {}", e)))?;

        Ok(())
    }

    /// Subscribe to a channel
    ///
    /// @param options - Subscription options: { channel: string, symbol: string, intradayOddLot?: boolean }
    #[napi(ts_args_type = "options: StockSubscribeOptions")]
    pub fn subscribe(&self, options: serde_json::Value) -> napi::Result<()> {
        let channel_str = options
            .get("channel")
            .and_then(|v| v.as_str())
            .ok_or_else(|| napi::Error::from_reason("Missing 'channel' field"))?;

        let symbol = options
            .get("symbol")
            .and_then(|v| v.as_str())
            .ok_or_else(|| napi::Error::from_reason("Missing 'symbol' field"))?;

        let odd_lot = options
            .get("intradayOddLot")
            .and_then(|v| v.as_bool());

        // Send command to worker thread
        let tx_guard = self.command_tx.lock().map_err(|e| {
            napi::Error::from_reason(format!("Lock error: {}", e))
        })?;

        if let Some(ref tx) = *tx_guard {
            tx.send(WsCommand::Subscribe {
                channel: channel_str.to_string(),
                symbol: symbol.to_string(),
                extra: odd_lot,
            }).map_err(|_| napi::Error::from_reason("Failed to send subscribe command"))?;
        } else {
            return Err(napi::Error::from_reason("Not connected. Call connect() first."));
        }

        Ok(())
    }

    /// Unsubscribe from a channel by subscription ID
    ///
    /// @param subscriptionId - The subscription ID returned from subscribed event
    #[napi]
    pub fn unsubscribe(&self, subscription_id: String) -> napi::Result<()> {
        let tx_guard = self.command_tx.lock().map_err(|e| {
            napi::Error::from_reason(format!("Lock error: {}", e))
        })?;

        if let Some(ref tx) = *tx_guard {
            tx.send(WsCommand::Unsubscribe { id: subscription_id })
                .map_err(|_| napi::Error::from_reason("Failed to send unsubscribe command"))?;
        } else {
            return Err(napi::Error::from_reason("Not connected. Call connect() first."));
        }

        Ok(())
    }

    /// Disconnect from the WebSocket server
    #[napi]
    pub fn disconnect(&self) -> napi::Result<()> {
        let tx_guard = self.command_tx.lock().map_err(|e| {
            napi::Error::from_reason(format!("Lock error: {}", e))
        })?;

        if let Some(ref tx) = *tx_guard {
            let _ = tx.send(WsCommand::Disconnect);
        }

        Ok(())
    }

    /// Check if connected
    #[napi(getter)]
    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::SeqCst)
    }

    /// Check if client has been closed
    ///
    /// Returns true if disconnect() has been called and client is closed.
    /// Once closed, the client cannot be reused - create a new instance.
    #[napi(getter)]
    pub fn is_closed(&self) -> bool {
        self.closed.load(Ordering::SeqCst)
    }
}

/// FutOpt WebSocket client for real-time futures/options market data
///
/// # JavaScript Usage
///
/// ```javascript
/// // Event handlers
/// ws.futopt.on('message', (data) => {
///   const msg = JSON.parse(data);
///   console.log(msg);
/// });
/// ws.futopt.on('connect', () => console.log('FutOpt WebSocket connected'));
///
/// // Connect
/// ws.futopt.connect();
///
/// // Subscribe to channels
/// ws.futopt.subscribe({ channel: 'trades', symbol: 'TXFC4' });
/// ws.futopt.subscribe({ channel: 'books', symbol: 'MXFB4', afterHours: true });
/// ```
#[napi]
pub struct FutOptWebSocketClient {
    api_key: String,
    callbacks: Arc<Mutex<EventCallbacks>>,
    connected: Arc<AtomicBool>,
    closed: Arc<AtomicBool>,
    command_tx: Arc<Mutex<Option<std::sync::mpsc::Sender<WsCommand>>>>,
}

#[napi]
impl FutOptWebSocketClient {
    /// Create a new FutOpt WebSocket client (internal use)
    fn new(api_key: String) -> Self {
        Self {
            api_key,
            callbacks: Arc::new(Mutex::new(EventCallbacks::default())),
            connected: Arc::new(AtomicBool::new(false)),
            closed: Arc::new(AtomicBool::new(false)),
            command_tx: Arc::new(Mutex::new(None)),
        }
    }

    /// Register an event handler
    ///
    /// @param event - Event type: "message", "connect", "disconnect", "reconnect", "error"
    /// @param callback - JavaScript callback function receiving string data
    #[napi(ts_args_type = "event: WebSocketEvent, callback: (data: string) => void")]
    pub fn on(&self, event: String, callback: ThreadsafeFunction<String>) -> napi::Result<()> {
        let mut callbacks = self
            .callbacks
            .lock()
            .map_err(|e| napi::Error::from_reason(format!("Lock error: {}", e)))?;

        // Wrap in Arc for thread-safe sharing (napi-rs 3.x pattern)
        let arc_callback = Arc::new(callback);

        match event.as_str() {
            "message" => callbacks.message = Some(arc_callback),
            "connect" => callbacks.connect = Some(arc_callback),
            "disconnect" => callbacks.disconnect = Some(arc_callback),
            "reconnect" => callbacks.reconnect = Some(arc_callback),
            "error" => callbacks.error = Some(arc_callback),
            _ => {
                return Err(napi::Error::from_reason(format!(
                    "Unknown event type: {}. Valid events: message, connect, disconnect, reconnect, error",
                    event
                )))
            }
        }
        Ok(())
    }

    /// Connect to the FutOpt WebSocket server
    ///
    /// This method spawns a background thread that manages the WebSocket connection.
    /// Connection result will be delivered via 'connect' or 'error' callbacks.
    #[napi]
    pub fn connect(&self) -> napi::Result<()> {
        // Create command channel
        let (cmd_tx, cmd_rx) = std::sync::mpsc::channel::<WsCommand>();

        // Store command sender
        {
            let mut tx_guard = self.command_tx.lock().map_err(|e| {
                napi::Error::from_reason(format!("Lock error: {}", e))
            })?;
            *tx_guard = Some(cmd_tx);
        }

        // Clone data for the worker thread
        let api_key = self.api_key.clone();
        let callbacks = Arc::clone(&self.callbacks);
        let connected = Arc::clone(&self.connected);
        let closed = Arc::clone(&self.closed);

        // Spawn worker thread that owns WebSocketClient
        thread::Builder::new()
            .name("futopt_ws_worker".to_string())
            .spawn(move || {
                use marketdata_core::websocket::{ConnectionConfig, WebSocketClient as CoreClient};
                use marketdata_core::AuthRequest;
                use marketdata_core::models::futopt::FutOptChannel;
                use marketdata_core::websocket::channels::FutOptSubscription;

                // Create tokio runtime
                let rt = match tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                {
                    Ok(rt) => rt,
                    Err(e) => {
                        fire_callback(&callbacks, "error", format!("Failed to create runtime: {}", e));
                        return;
                    }
                };

                // Create WebSocket client with FutOpt endpoint
                let config = ConnectionConfig::fugle_futopt(AuthRequest::with_api_key(&api_key));
                let client = CoreClient::new(config);

                // Connect
                let connect_result = rt.block_on(async {
                    client.connect().await
                });

                if let Err(e) = connect_result {
                    fire_callback(&callbacks, "error", format!("[{}] {}", e.to_error_code(), e));
                    return;
                }

                // Mark as connected
                connected.store(true, Ordering::SeqCst);
                fire_callback(&callbacks, "connect", "connected".to_string());

                // Get message receiver
                let receiver = client.messages();

                // Main event loop
                loop {
                    // Check for commands (non-blocking)
                    match cmd_rx.try_recv() {
                        Ok(WsCommand::Subscribe { channel, symbol, extra }) => {
                            let channel_enum = match channel.to_lowercase().as_str() {
                                "trades" => FutOptChannel::Trades,
                                "candles" => FutOptChannel::Candles,
                                "books" => FutOptChannel::Books,
                                "aggregates" => FutOptChannel::Aggregates,
                                _ => continue,
                            };
                            let after_hours = extra.unwrap_or(false);
                            let sub = FutOptSubscription::new(channel_enum, &symbol).with_after_hours(after_hours);

                            // Build subscribe request and send
                            let request = sub.to_subscribe_request();
                            if let Ok(request_str) = serde_json::to_string(&request) {
                                if let Ok(ws_req) = serde_json::from_str::<marketdata_core::models::WebSocketRequest>(&request_str) {
                                    let _ = rt.block_on(client.send(ws_req));
                                }
                            }
                        }
                        Ok(WsCommand::Unsubscribe { id }) => {
                            let _ = rt.block_on(client.unsubscribe_by_id(&id));
                        }
                        Ok(WsCommand::Disconnect) => {
                            let _ = rt.block_on(client.disconnect());
                            connected.store(false, Ordering::SeqCst);
                            closed.store(true, Ordering::SeqCst);
                            fire_callback(&callbacks, "disconnect", "disconnected".to_string());
                            break;
                        }
                        Err(std::sync::mpsc::TryRecvError::Empty) => {}
                        Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                            // Command channel closed, cleanup
                            let _ = rt.block_on(client.disconnect());
                            connected.store(false, Ordering::SeqCst);
                            closed.store(true, Ordering::SeqCst);
                            break;
                        }
                    }

                    // Check for messages (with timeout)
                    match receiver.receive_timeout(Duration::from_millis(50)) {
                        Ok(Some(msg)) => {
                            if let Ok(json_str) = serde_json::to_string(&msg) {
                                fire_callback(&callbacks, "message", json_str);
                            }
                        }
                        Ok(None) => {
                            // Timeout, continue loop
                        }
                        Err(_) => {
                            // Channel closed
                            connected.store(false, Ordering::SeqCst);
                            closed.store(true, Ordering::SeqCst);
                            fire_callback(&callbacks, "disconnect", "channel_closed".to_string());
                            break;
                        }
                    }
                }
            })
            .map_err(|e| napi::Error::from_reason(format!("Failed to spawn worker thread: {}", e)))?;

        Ok(())
    }

    /// Subscribe to a channel
    ///
    /// @param options - Subscription options: { channel: string, symbol: string, afterHours?: boolean }
    #[napi(ts_args_type = "options: FutOptSubscribeOptions")]
    pub fn subscribe(&self, options: serde_json::Value) -> napi::Result<()> {
        let channel_str = options
            .get("channel")
            .and_then(|v| v.as_str())
            .ok_or_else(|| napi::Error::from_reason("Missing 'channel' field"))?;

        let symbol = options
            .get("symbol")
            .and_then(|v| v.as_str())
            .ok_or_else(|| napi::Error::from_reason("Missing 'symbol' field"))?;

        let after_hours = options
            .get("afterHours")
            .and_then(|v| v.as_bool());

        // Send command to worker thread
        let tx_guard = self.command_tx.lock().map_err(|e| {
            napi::Error::from_reason(format!("Lock error: {}", e))
        })?;

        if let Some(ref tx) = *tx_guard {
            tx.send(WsCommand::Subscribe {
                channel: channel_str.to_string(),
                symbol: symbol.to_string(),
                extra: after_hours,
            }).map_err(|_| napi::Error::from_reason("Failed to send subscribe command"))?;
        } else {
            return Err(napi::Error::from_reason("Not connected. Call connect() first."));
        }

        Ok(())
    }

    /// Unsubscribe from a channel by subscription ID
    ///
    /// @param subscriptionId - The subscription ID returned from subscribed event
    #[napi]
    pub fn unsubscribe(&self, subscription_id: String) -> napi::Result<()> {
        let tx_guard = self.command_tx.lock().map_err(|e| {
            napi::Error::from_reason(format!("Lock error: {}", e))
        })?;

        if let Some(ref tx) = *tx_guard {
            tx.send(WsCommand::Unsubscribe { id: subscription_id })
                .map_err(|_| napi::Error::from_reason("Failed to send unsubscribe command"))?;
        } else {
            return Err(napi::Error::from_reason("Not connected. Call connect() first."));
        }

        Ok(())
    }

    /// Disconnect from the WebSocket server
    #[napi]
    pub fn disconnect(&self) -> napi::Result<()> {
        let tx_guard = self.command_tx.lock().map_err(|e| {
            napi::Error::from_reason(format!("Lock error: {}", e))
        })?;

        if let Some(ref tx) = *tx_guard {
            let _ = tx.send(WsCommand::Disconnect);
        }

        Ok(())
    }

    /// Check if connected
    #[napi(getter)]
    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::SeqCst)
    }

    /// Check if client has been closed
    ///
    /// Returns true if disconnect() has been called and client is closed.
    /// Once closed, the client cannot be reused - create a new instance.
    #[napi(getter)]
    pub fn is_closed(&self) -> bool {
        self.closed.load(Ordering::SeqCst)
    }
}

/// Helper function to fire callbacks from worker thread
///
/// Uses Arc::clone to get a reference to the callback, allowing safe use from
/// any thread. The ThreadsafeFunction with CalleeHandled=true (default) uses
/// call(Ok(data), mode) to pass data to JavaScript.
fn fire_callback(callbacks: &Arc<Mutex<EventCallbacks>>, event: &str, data: String) {
    if let Ok(cb) = callbacks.lock() {
        let callback = match event {
            "message" => cb.message.as_ref(),
            "connect" => cb.connect.as_ref(),
            "disconnect" => cb.disconnect.as_ref(),
            "reconnect" => cb.reconnect.as_ref(),
            "error" => cb.error.as_ref(),
            _ => None,
        };

        if let Some(callback) = callback {
            // Clone the Arc (not the callback itself) for thread-safe access
            let callback_ref = Arc::clone(callback);
            // In napi-rs 3.x with CalleeHandled=true (default), call() takes Result<T, ErrorStatus>
            callback_ref.call(Ok(data), ThreadsafeFunctionCallMode::NonBlocking);
        }
    }
}

// Unit tests are disabled because ThreadsafeFunction requires Node.js runtime
// Integration tests are done via JavaScript (test_websocket.js)
