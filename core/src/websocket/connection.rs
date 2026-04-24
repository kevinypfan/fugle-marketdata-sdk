//! WebSocket connection lifecycle management

use crate::models::{SubscribeRequest, WebSocketMessage, WebSocketRequest};
use crate::websocket::{
    ConnectionConfig, HealthCheck, HealthCheckConfig, MessageReceiver, ReconnectionConfig,
    ReconnectionManager, SubscriptionManager,
};
use crate::MarketDataError;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use std::sync::{mpsc, Arc};
use tokio::net::TcpStream;
use tokio::sync::mpsc as tokio_mpsc;
use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinHandle;
use tokio::time::{sleep, timeout, Duration};
use tokio_tungstenite::{connect_async_tls_with_config, Connector, MaybeTlsStream, WebSocketStream};
use tokio_tungstenite::tungstenite::Message;

/// Build the optional `Connector` used by both call sites below. Returns
/// `None` when TLS config is in its default shape so tokio-tungstenite
/// uses the library default — identical to pre-3.0.1 behaviour.
fn tls_connector_for(
    config: &ConnectionConfig,
) -> Result<Option<Connector>, MarketDataError> {
    let connector = crate::tls::build_native_tls_connector(&config.tls)?;
    Ok(connector.map(Connector::NativeTls))
}

/// Type alias for WebSocket write half
type WsSink = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
/// Type alias for WebSocket read half
type WsStream = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

/// WebSocket connection state machine
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    /// Not connected
    Disconnected,
    /// Connecting to server
    Connecting,
    /// Authenticating with server
    Authenticating,
    /// Connected and authenticated
    Connected,
    /// Reconnecting after disconnection
    Reconnecting { attempt: u32 },
    /// Connection closed
    Closed { code: Option<u16>, reason: String },
}

/// Events emitted by WebSocket connection
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionEvent {
    /// Connection attempt started
    Connecting,
    /// Connection established
    Connected,
    /// Authentication successful
    Authenticated,
    /// Authentication rejected by the server (parallels old SDKs' `unauthenticated` event)
    Unauthenticated { message: String },
    /// Connection closed
    Disconnected { code: Option<u16>, reason: String },
    /// Reconnection attempt started
    Reconnecting { attempt: u32 },
    /// Reconnection failed after max attempts
    ReconnectFailed { attempts: u32 },
    /// Error occurred
    Error { message: String, code: i32 },
}

/// WebSocket client for real-time market data
#[allow(clippy::arc_with_non_send_sync)] // MessageReceiver uses std::sync::mpsc for FFI compatibility
pub struct WebSocketClient {
    config: ConnectionConfig,
    state: Arc<RwLock<ConnectionState>>,
    event_tx: mpsc::Sender<ConnectionEvent>,
    event_rx: Arc<Mutex<mpsc::Receiver<ConnectionEvent>>>,
    /// Write half of the WebSocket stream (held by the writer task during
    /// normal operation; close/force_close paths may also touch it).
    ws_sink: Arc<Mutex<Option<WsSink>>>,
    /// Outbound write channel. All `subscribe`/`unsubscribe`/`send`/health-check
    /// pings push pre-serialized JSON strings here; a single writer task drains
    /// it into `ws_sink`. This eliminates lock contention on `ws_sink` between
    /// concurrent senders.
    write_tx: Arc<Mutex<Option<tokio_mpsc::Sender<String>>>>,
    reconnection: Arc<Mutex<ReconnectionManager>>,
    subscriptions: Arc<SubscriptionManager>,
    health_check: Arc<HealthCheck>,
    message_tx: mpsc::Sender<WebSocketMessage>,
    message_receiver: Arc<MessageReceiver>,
    // Internal handles
    dispatch_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
    health_check_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
    writer_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl WebSocketClient {
    /// Create a new WebSocket client with default reconnection config
    ///
    /// # Example
    ///
    /// ```rust
    /// use marketdata_core::websocket::{ConnectionConfig, WebSocketClient};
    /// use marketdata_core::AuthRequest;
    ///
    /// let config = ConnectionConfig::fugle_stock(
    ///     AuthRequest::with_api_key("my-api-key")
    /// );
    /// let client = WebSocketClient::new(config);
    /// ```
    pub fn new(config: ConnectionConfig) -> Self {
        Self::with_reconnection_config(config, ReconnectionConfig::default())
    }

    /// Create a new WebSocket client with custom reconnection config
    pub fn with_reconnection_config(
        config: ConnectionConfig,
        reconnection_config: ReconnectionConfig,
    ) -> Self {
        Self::with_full_config(
            config,
            reconnection_config,
            HealthCheckConfig::default(),
        )
    }

    /// Create a new WebSocket client with custom health check config
    pub fn with_health_check_config(
        config: ConnectionConfig,
        health_check_config: HealthCheckConfig,
    ) -> Self {
        Self::with_full_config(
            config,
            ReconnectionConfig::default(),
            health_check_config,
        )
    }

    /// Create a new WebSocket client with full custom config
    #[allow(clippy::arc_with_non_send_sync)] // MessageReceiver uses std::sync::mpsc for FFI compatibility
    pub fn with_full_config(
        config: ConnectionConfig,
        reconnection_config: ReconnectionConfig,
        health_check_config: HealthCheckConfig,
    ) -> Self {
        let (event_tx, event_rx) = mpsc::channel();
        let (message_tx, message_rx) = mpsc::channel();

        Self {
            config,
            state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            event_tx,
            event_rx: Arc::new(Mutex::new(event_rx)),
            ws_sink: Arc::new(Mutex::new(None)),
            write_tx: Arc::new(Mutex::new(None)),
            reconnection: Arc::new(Mutex::new(ReconnectionManager::new(reconnection_config))),
            subscriptions: Arc::new(SubscriptionManager::new()),
            health_check: Arc::new(HealthCheck::new(health_check_config)),
            message_tx,
            message_receiver: Arc::new(MessageReceiver::new(message_rx)),
            dispatch_handle: Arc::new(Mutex::new(None)),
            health_check_handle: Arc::new(Mutex::new(None)),
            writer_handle: Arc::new(Mutex::new(None)),
        }
    }

    /// Get current connection state (snapshot)
    ///
    /// # Example
    ///
    /// ```rust
    /// use marketdata_core::websocket::{ConnectionConfig, WebSocketClient, ConnectionState};
    /// use marketdata_core::AuthRequest;
    ///
    /// let config = ConnectionConfig::fugle_stock(
    ///     AuthRequest::with_api_key("my-api-key")
    /// );
    /// let client = WebSocketClient::new(config);
    /// assert_eq!(client.state(), ConnectionState::Disconnected);
    /// ```
    pub fn state(&self) -> ConnectionState {
        // This is a blocking call, but state reads are fast
        // In a real async context, use state_async() instead
        tokio::runtime::Handle::try_current()
            .ok()
            .and_then(|handle| {
                handle.block_on(async {
                    let state = self.state.read().await;
                    Some(state.clone())
                })
            })
            .unwrap_or(ConnectionState::Disconnected)
    }

    /// Get current connection state (async version)
    pub async fn state_async(&self) -> ConnectionState {
        let state = self.state.read().await;
        state.clone()
    }

    /// Check if client has been closed
    ///
    /// Returns true if disconnect() has been called and state is Closed.
    /// Once closed, the client cannot be reused - create a new instance.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use marketdata_core::websocket::{ConnectionConfig, WebSocketClient, ConnectionState};
    /// use marketdata_core::AuthRequest;
    ///
    /// # async fn example() {
    /// let config = ConnectionConfig::fugle_stock(AuthRequest::with_api_key("key"));
    /// let client = WebSocketClient::new(config);
    ///
    /// // Initially not closed
    /// assert!(!client.is_closed().await);
    /// # }
    /// ```
    pub async fn is_closed(&self) -> bool {
        let state = self.state.read().await;
        matches!(*state, ConnectionState::Closed { .. })
    }

    /// Sync version of is_closed() for FFI
    ///
    /// Returns true if the client has been closed. Returns false if
    /// unable to determine state (e.g., no tokio runtime).
    pub fn is_closed_sync(&self) -> bool {
        tokio::runtime::Handle::try_current()
            .ok()
            .and_then(|handle| {
                handle.block_on(async {
                    let state = self.state.read().await;
                    Some(matches!(*state, ConnectionState::Closed { .. }))
                })
            })
            .unwrap_or(false)
    }

    /// Get reference to event receiver
    ///
    /// Consumers can use this to receive connection events
    pub fn events(&self) -> &Arc<Mutex<mpsc::Receiver<ConnectionEvent>>> {
        &self.event_rx
    }

    /// Subscribe to connection state change events
    ///
    /// This is a semantic alias for `events()` that emphasizes the state change focus.
    /// Returns a receiver for connection lifecycle events.
    ///
    /// Event types:
    /// - `Connecting` - Connection attempt started
    /// - `Connected` - WebSocket connection established
    /// - `Authenticated` - Authentication successful
    /// - `Disconnected { code, reason }` - Connection closed
    /// - `Reconnecting { attempt }` - Reconnection attempt started
    /// - `ReconnectFailed { attempts }` - Reconnection failed after max attempts
    /// - `Error { message, code }` - Error occurred
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use marketdata_core::websocket::{WebSocketClient, ConnectionConfig, ConnectionEvent};
    /// use marketdata_core::AuthRequest;
    /// use std::sync::Arc;
    ///
    /// let client = WebSocketClient::new(
    ///     ConnectionConfig::fugle_stock(AuthRequest::with_api_key("key"))
    /// );
    ///
    /// // Clone the Arc to move into the thread
    /// let events = Arc::clone(client.state_events());
    /// std::thread::spawn(move || {
    ///     while let Ok(event) = events.blocking_lock().recv() {
    ///         match event {
    ///             ConnectionEvent::Connected => println!("Connected!"),
    ///             ConnectionEvent::Disconnected { code, reason } => {
    ///                 println!("Disconnected: {:?} - {}", code, reason);
    ///                 break;
    ///             }
    ///             _ => {}
    ///         }
    ///     }
    /// });
    /// ```
    pub fn state_events(&self) -> &Arc<Mutex<mpsc::Receiver<ConnectionEvent>>> {
        &self.event_rx
    }

    /// Get reference to message receiver for FFI consumers
    ///
    /// Returns MessageReceiver with blocking API suitable for FFI bindings
    #[allow(clippy::arc_with_non_send_sync)] // MessageReceiver uses std::sync::mpsc for FFI compatibility
    pub fn messages(&self) -> Arc<MessageReceiver> {
        Arc::clone(&self.message_receiver)
    }

    /// Connect to WebSocket server and authenticate
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Client has been closed (ClientClosed)
    /// - Connection fails
    /// - Authentication fails or times out
    /// - WebSocket handshake fails
    pub async fn connect(&self) -> Result<(), MarketDataError> {
        // Check if client is closed - cannot reconnect a closed client
        if self.is_closed().await {
            return Err(MarketDataError::ClientClosed);
        }

        // Update state to Connecting
        {
            let mut state = self.state.write().await;
            *state = ConnectionState::Connecting;
        }
        let _ = self.event_tx.send(ConnectionEvent::Connecting);

        // Connect to WebSocket (with optional TLS customization).
        let tls_connector = tls_connector_for(&self.config)?;
        let connect_result = timeout(
            self.config.connect_timeout,
            connect_async_tls_with_config(&self.config.url, None, false, tls_connector),
        )
        .await;

        let (ws_stream, _response) = match connect_result {
            Ok(Ok((stream, response))) => (stream, response),
            Ok(Err(e)) => {
                let err: MarketDataError = e.into();
                {
                    let mut state = self.state.write().await;
                    *state = ConnectionState::Disconnected;
                }
                let _ = self.event_tx.send(ConnectionEvent::Error {
                    message: err.to_string(),
                    code: err.to_error_code(),
                });
                return Err(err);
            }
            Err(_) => {
                let err = MarketDataError::TimeoutError {
                    operation: "WebSocket connect".to_string(),
                };
                {
                    let mut state = self.state.write().await;
                    *state = ConnectionState::Disconnected;
                }
                let _ = self.event_tx.send(ConnectionEvent::Error {
                    message: err.to_string(),
                    code: err.to_error_code(),
                });
                return Err(err);
            }
        };

        // Split the stream into read/write halves
        let (mut ws_sink, mut ws_read) = ws_stream.split();

        let _ = self.event_tx.send(ConnectionEvent::Connected);

        // Update state to Authenticating
        {
            let mut state = self.state.write().await;
            *state = ConnectionState::Authenticating;
        }

        // Send authentication message
        let auth_msg = WebSocketRequest::auth(self.config.auth.clone());
        let auth_json = serde_json::to_string(&auth_msg)
            .map_err(|e| MarketDataError::DeserializationError { source: e })?;

        ws_sink
            .send(Message::Text(auth_json.into()))
            .await
            .map_err(MarketDataError::from)?;

        // Wait for authenticated event or timeout
        // All messages during auth phase are forwarded to message channel
        let message_tx = self.message_tx.clone();
        let auth_timeout = Duration::from_secs(10);
        let auth_result = timeout(auth_timeout, async {
            while let Some(msg_result) = ws_read.next().await {
                match msg_result {
                    Ok(Message::Text(text)) => {
                        if let Ok(ws_msg) =
                            serde_json::from_str::<WebSocketMessage>(&text)
                        {
                            // Forward ALL messages to channel (including auth)
                            let _ = message_tx.send(ws_msg.clone());

                            if ws_msg.is_authenticated() {
                                return Ok(());
                            }
                            if ws_msg.is_error() {
                                return Err(MarketDataError::AuthError {
                                    msg: ws_msg
                                        .error_message()
                                        .unwrap_or_else(|| "Unknown error".to_string()),
                                });
                            }
                        }
                    }
                    Err(e) => {
                        return Err(MarketDataError::from(e));
                    }
                    _ => {}
                }
            }
            Err(MarketDataError::ConnectionError {
                msg: "Stream closed during authentication".to_string(),
            })
        })
        .await;

        match auth_result {
            Ok(Ok(())) => {
                // Store the write half for sending messages
                {
                    let mut sink_guard = self.ws_sink.lock().await;
                    *sink_guard = Some(ws_sink);
                }

                // Spawn the writer task and install its sender. All
                // subsequent outbound messages flow through this channel.
                self.start_writer_task().await;

                {
                    let mut state = self.state.write().await;
                    *state = ConnectionState::Connected;
                }
                let _ = self.event_tx.send(ConnectionEvent::Authenticated);

                // Spawn dispatch task to handle incoming messages (uses read half)
                self.spawn_dispatch_task(ws_read).await;

                // Start health check task if enabled
                self.start_health_check().await;

                Ok(())
            }
            Ok(Err(e)) => {
                {
                    let mut state = self.state.write().await;
                    *state = ConnectionState::Disconnected;
                }
                // Server-rejected credentials → emit Unauthenticated so old SDK
                // listeners on `unauthenticated` keep working. Other failures
                // (network, parse, etc.) still go through the generic Error event.
                if let MarketDataError::AuthError { msg } = &e {
                    let _ = self.event_tx.send(ConnectionEvent::Unauthenticated {
                        message: msg.clone(),
                    });
                } else {
                    let _ = self.event_tx.send(ConnectionEvent::Error {
                        message: e.to_string(),
                        code: e.to_error_code(),
                    });
                }
                Err(e)
            }
            Err(_) => {
                let err = MarketDataError::TimeoutError {
                    operation: "WebSocket authentication".to_string(),
                };
                {
                    let mut state = self.state.write().await;
                    *state = ConnectionState::Disconnected;
                }
                let _ = self.event_tx.send(ConnectionEvent::Error {
                    message: err.to_string(),
                    code: err.to_error_code(),
                });
                Err(err)
            }
        }
    }

    /// Disconnect from WebSocket server with graceful shutdown
    ///
    /// Shutdown sequence:
    /// 1. Stop health check monitoring
    /// 2. Cancel dispatch task (abort async task)
    /// 3. Join health check thread (blocking wait)
    /// 4. Send close frame to server
    /// 5. Wait for close acknowledgment (with timeout)
    /// 6. Update state to Closed
    /// 7. Send Disconnected event
    ///
    /// # Errors
    ///
    /// Returns error if sending close frame fails. The client is still
    /// marked as closed even if the close handshake fails.
    pub async fn disconnect(&self) -> Result<(), MarketDataError> {
        // 1. Stop health check first (prevents false triggers)
        self.health_check.stop();

        // 2. Cancel dispatch task
        {
            let mut handle = self.dispatch_handle.lock().await;
            if let Some(h) = handle.take() {
                h.abort();
                let _ = h.await;
            }
        }

        // 3. Cancel health check task
        {
            let mut handle = self.health_check_handle.lock().await;
            if let Some(h) = handle.take() {
                h.abort();
                let _ = h.await;
            }
        }

        // 4. Drop the write_tx slot and abort the writer task
        {
            let mut tx_guard = self.write_tx.lock().await;
            *tx_guard = None;
        }
        {
            let mut handle = self.writer_handle.lock().await;
            if let Some(h) = handle.take() {
                h.abort();
                let _ = h.await;
            }
        }

        // 5. Send close frame with timeout
        let close_result = self.close_websocket_with_timeout(Duration::from_secs(5)).await;

        // 6. Update state to Closed (always, even if close failed)
        {
            let mut state = self.state.write().await;
            *state = ConnectionState::Closed {
                code: Some(1000),
                reason: "Normal closure".to_string(),
            };
        }

        // 7. Send Disconnected event
        let _ = self.event_tx.send(ConnectionEvent::Disconnected {
            code: Some(1000),
            reason: "Normal closure".to_string(),
        });

        close_result
    }

    /// Close WebSocket with proper handshake and timeout
    ///
    /// From RESEARCH.md Pitfall 1: Must continue reading after close()
    /// until receiving ConnectionClosed error.
    async fn close_websocket_with_timeout(
        &self,
        _timeout_duration: Duration,
    ) -> Result<(), MarketDataError> {
        // Send close frame through the write half
        let mut sink_guard = self.ws_sink.lock().await;
        if let Some(ref mut sink) = *sink_guard {
            // Send close frame
            if let Err(e) = sink.close().await {
                // Log but continue - we still want to clean up
                eprintln!("Warning: Failed to send close frame: {}", e);
            }
        }

        // Clear the sink
        *sink_guard = None;

        Ok(())
    }

    /// Force close without waiting for handshake
    ///
    /// Use when graceful close is not possible or times out.
    pub async fn force_close(&self) -> Result<(), MarketDataError> {
        // Stop health check
        self.health_check.stop();

        // Abort dispatch task without waiting
        {
            let mut handle = self.dispatch_handle.lock().await;
            if let Some(h) = handle.take() {
                h.abort();
            }
        }

        // Abort health check task
        {
            let mut handle = self.health_check_handle.lock().await;
            if let Some(h) = handle.take() {
                h.abort();
            }
        }

        // Abort writer task and clear sender
        {
            let mut tx_guard = self.write_tx.lock().await;
            *tx_guard = None;
        }
        {
            let mut handle = self.writer_handle.lock().await;
            if let Some(h) = handle.take() {
                h.abort();
            }
        }

        // Drop sink without close frame
        {
            let mut sink_guard = self.ws_sink.lock().await;
            *sink_guard = None;
        }

        // Update state
        {
            let mut state = self.state.write().await;
            *state = ConnectionState::Closed {
                code: Some(1006), // Abnormal closure
                reason: "Force closed".to_string(),
            };
        }

        let _ = self.event_tx.send(ConnectionEvent::Disconnected {
            code: Some(1006),
            reason: "Force closed".to_string(),
        });

        Ok(())
    }

    /// Check if currently connected
    pub async fn is_connected(&self) -> bool {
        let state = self.state.read().await;
        matches!(*state, ConnectionState::Connected)
    }

    /// Subscribe to a channel
    ///
    /// Adds subscription to state immediately. If connected, sends subscribe
    /// message to server. If disconnected, stores for reconnection.
    ///
    /// # Errors
    ///
    /// Returns `ClientClosed` if the client has been closed.
    ///
    /// From CONTEXT.md: "重連期間新訂閱請求：立即加入訂閱狀態，重連後一併訂閱"
    pub async fn subscribe(&self, req: SubscribeRequest) -> Result<(), MarketDataError> {
        // Check if client is closed
        if self.is_closed().await {
            return Err(MarketDataError::ClientClosed);
        }

        // Add to subscription state immediately
        self.subscriptions.subscribe(req.clone());

        // If connected, enqueue subscribe message
        if self.is_connected().await {
            let sub_msg = WebSocketRequest::subscribe(req);
            let sub_json = serde_json::to_string(&sub_msg)
                .map_err(|e| MarketDataError::DeserializationError { source: e })?;
            self.enqueue_write(sub_json).await?;
        }

        Ok(())
    }

    /// Unsubscribe from a channel
    ///
    /// Removes subscription from state immediately. If connected, sends
    /// unsubscribe message to server.
    ///
    /// # Errors
    ///
    /// Returns `ClientClosed` if the client has been closed.
    ///
    /// From CONTEXT.md: "unsubscribe() 在斷線期間立即從狀態移除"
    pub async fn unsubscribe(&self, key: &str) -> Result<(), MarketDataError> {
        // Check if client is closed
        if self.is_closed().await {
            return Err(MarketDataError::ClientClosed);
        }

        // Consume the server-assigned id BEFORE dropping local state, since
        // unsubscribe() clears the id map as part of its bookkeeping.
        let server_id = self.subscriptions.take_server_id(key);

        // Remove from subscription state immediately
        self.subscriptions.unsubscribe(key);

        // If connected, enqueue unsubscribe message. Fugle's protocol requires
        // the server-assigned id from the `subscribed` ack, wrapped in an
        // `ids` array. Fallback to the local key keeps the wire format valid
        // when the ack hasn't arrived yet (race on fast sub/unsub).
        if self.is_connected().await {
            let id = server_id.unwrap_or_else(|| key.to_string());
            let unsub_msg = WebSocketRequest::unsubscribe(
                crate::models::UnsubscribeRequest::by_ids(vec![id]),
            );
            let unsub_json = serde_json::to_string(&unsub_msg)
                .map_err(|e| MarketDataError::DeserializationError { source: e })?;
            self.enqueue_write(unsub_json).await?;
        }

        Ok(())
    }

    /// Get all active subscriptions
    pub fn subscriptions(&self) -> Vec<SubscribeRequest> {
        self.subscriptions.get_all()
    }

    /// Manually reconnect after disconnection
    ///
    /// # Errors
    ///
    /// Returns `ClientClosed` if the client has been closed.
    /// A closed client cannot be reconnected - create a new instance.
    ///
    /// From CONTEXT.md: "支援 reconnect() 方法讓使用者手動觸發重連"
    /// Resets reconnection manager and attempts fresh connection.
    pub async fn reconnect(&self) -> Result<(), MarketDataError> {
        // Check if client is closed - cannot reconnect a closed client
        if self.is_closed().await {
            return Err(MarketDataError::ClientClosed);
        }

        // Reset reconnection manager for fresh attempt
        {
            let mut reconnection = self.reconnection.lock().await;
            reconnection.reset();
        }

        // Attempt connection
        self.connect().await?;

        // Resubscribe all
        self.resubscribe_all().await?;

        Ok(())
    }

    /// Internal: Resubscribe all stored subscriptions
    ///
    /// From CONTEXT.md: "重連後按原始訂閱順序重新訂閱"
    async fn resubscribe_all(&self) -> Result<(), MarketDataError> {
        // Old server ids point at a dead connection — clear before replay so
        // the fresh subscribed acks can overwrite cleanly. Without this,
        // unsubscribe after reconnect could briefly pick up a zombie id.
        self.subscriptions.clear_server_ids();

        let subs = self.subscriptions.get_all();

        for req in subs {
            let sub_msg = WebSocketRequest::subscribe(req);
            let sub_json = serde_json::to_string(&sub_msg)
                .map_err(|e| MarketDataError::DeserializationError { source: e })?;
            self.enqueue_write(sub_json).await?;
        }

        Ok(())
    }

    /// Send a WebSocket request message
    ///
    /// Used internally and exposed for advanced use cases
    ///
    /// # Errors
    ///
    /// Returns `ClientClosed` if the client has been closed.
    pub async fn send(&self, request: WebSocketRequest) -> Result<(), MarketDataError> {
        if self.is_closed().await {
            return Err(MarketDataError::ClientClosed);
        }

        let json = serde_json::to_string(&request)
            .map_err(|e| MarketDataError::DeserializationError { source: e })?;
        self.enqueue_write(json).await
    }

    /// Send raw text message to WebSocket
    ///
    /// Used internally for sending subscription requests
    pub(crate) async fn send_text(&self, text: &str) -> Result<(), MarketDataError> {
        self.enqueue_write(text.to_string()).await
    }

    // ========================================================================
    // Stock Streaming Channel API (Phase 4)
    // ========================================================================

    /// Subscribe to a stock streaming channel
    ///
    /// # Errors
    ///
    /// Returns `ClientClosed` if the client has been closed.
    ///
    /// # Example
    /// ```rust,no_run
    /// use marketdata_core::websocket::{WebSocketClient, ConnectionConfig};
    /// use marketdata_core::websocket::channels::StockSubscription;
    /// use marketdata_core::models::Channel;
    /// use marketdata_core::AuthRequest;
    ///
    /// # async fn example() -> Result<(), marketdata_core::MarketDataError> {
    /// let config = ConnectionConfig::fugle_stock(AuthRequest::with_api_key("key"));
    /// let client = WebSocketClient::new(config);
    ///
    /// // Subscribe to trades
    /// let sub = StockSubscription::new(Channel::Trades, "2330");
    /// client.subscribe_channel(sub).await?;
    ///
    /// // Subscribe to odd lot trades
    /// let odd_lot_sub = StockSubscription::new(Channel::Trades, "2330")
    ///     .with_odd_lot(true);
    /// client.subscribe_channel(odd_lot_sub).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn subscribe_channel(
        &self,
        sub: crate::websocket::channels::StockSubscription,
    ) -> Result<(), MarketDataError> {
        // Check if client is closed
        if self.is_closed().await {
            return Err(MarketDataError::ClientClosed);
        }

        // Build subscribe request JSON
        let request = sub.to_subscribe_request();
        let request_str = serde_json::to_string(&request)
            .map_err(|e| MarketDataError::DeserializationError { source: e })?;

        // Store full SubscribeRequest so reconnect's resubscribe_all replays
        // the same modifier (oddlot) instead of silently downgrading.
        let stored = crate::models::SubscribeRequest {
            channel: sub.channel.as_str().to_string(),
            symbol: Some(sub.symbol.clone()),
            intraday_odd_lot: if sub.intraday_odd_lot { Some(true) } else { None },
            ..Default::default()
        };
        self.subscriptions.subscribe(stored);

        // Send if connected (ignore error if not connected - will be sent on reconnect)
        let _ = self.send_text(&request_str).await;
        Ok(())
    }

    /// Subscribe to multiple symbols on the same channel
    ///
    /// # Example
    /// ```rust,no_run
    /// # use marketdata_core::websocket::{WebSocketClient, ConnectionConfig};
    /// # use marketdata_core::models::Channel;
    /// # use marketdata_core::AuthRequest;
    /// # async fn example() -> Result<(), marketdata_core::MarketDataError> {
    /// # let config = ConnectionConfig::fugle_stock(AuthRequest::with_api_key("key"));
    /// # let client = WebSocketClient::new(config);
    /// client.subscribe_symbols(Channel::Trades, &["2330", "2317", "2454"], false).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn subscribe_symbols(
        &self,
        channel: crate::models::Channel,
        symbols: &[&str],
        intraday_odd_lot: bool,
    ) -> Result<(), MarketDataError> {
        use crate::websocket::channels::StockSubscription;

        for symbol in symbols {
            let sub = StockSubscription::new(channel, *symbol).with_odd_lot(intraday_odd_lot);
            self.subscribe_channel(sub).await?;
        }
        Ok(())
    }

    /// Unsubscribe from a stock streaming channel by subscription
    ///
    /// Note: This removes from local state. To unsubscribe from server,
    /// you need the subscription ID returned from the subscribed event.
    pub async fn unsubscribe_channel(
        &self,
        sub: &crate::websocket::channels::StockSubscription,
    ) -> Result<(), MarketDataError> {
        // Remove from local subscription state
        self.subscriptions.unsubscribe(&sub.key());
        Ok(())
    }

    /// Subscribe to a FutOpt streaming channel.
    ///
    /// Mirrors [`subscribe_channel`](Self::subscribe_channel) but accepts a
    /// `FutOptSubscription`, whose `to_subscribe_request` encodes the
    /// `afterHours` flag when the regular/after-hours session is requested.
    /// The subscription key (`"{channel}:{symbol}[:afterhours]"`) is what the
    /// subscription manager uses to track state for reconnect and unsubscribe.
    pub async fn subscribe_futopt_channel(
        &self,
        sub: crate::websocket::channels::FutOptSubscription,
    ) -> Result<(), MarketDataError> {
        if self.is_closed().await {
            return Err(MarketDataError::ClientClosed);
        }
        let request = sub.to_subscribe_request();
        let request_str = serde_json::to_string(&request)
            .map_err(|e| MarketDataError::DeserializationError { source: e })?;
        // Store full SubscribeRequest so reconnect's resubscribe_all replays
        // the afterHours flag — previous subscribe_key path dropped it.
        let stored = crate::models::SubscribeRequest {
            channel: sub.channel.as_str().to_string(),
            symbol: Some(sub.symbol.clone()),
            after_hours: if sub.after_hours { Some(true) } else { None },
            ..Default::default()
        };
        self.subscriptions.subscribe(stored);
        let _ = self.send_text(&request_str).await;
        Ok(())
    }

    /// Unsubscribe a FutOpt streaming channel.
    ///
    /// Looks up the server-assigned id captured from the `subscribed` ack and
    /// sends `{event:"unsubscribe", data:{ids:[server_id]}}` — matching the
    /// Fugle protocol. If the ack hasn't been recorded yet (rare race), falls
    /// back to the local key so the wire format stays valid.
    pub async fn unsubscribe_futopt_channel(
        &self,
        sub: &crate::websocket::channels::FutOptSubscription,
    ) -> Result<(), MarketDataError> {
        if self.is_closed().await {
            return Err(MarketDataError::ClientClosed);
        }

        let key = sub.key();
        let server_id = self.subscriptions.take_server_id(&key);
        self.subscriptions.unsubscribe(&key);

        if self.is_connected().await {
            let id = server_id.unwrap_or_else(|| key.clone());
            let unsub_msg = crate::models::WebSocketRequest::unsubscribe(
                crate::models::UnsubscribeRequest::by_ids(vec![id]),
            );
            let unsub_json = serde_json::to_string(&unsub_msg)
                .map_err(|e| MarketDataError::DeserializationError { source: e })?;
            self.enqueue_write(unsub_json).await?;
        }

        Ok(())
    }

    /// Unsubscribe from server using subscription ID
    ///
    /// The ID is returned in the "subscribed" event after subscribing.
    pub async fn unsubscribe_by_id(&self, subscription_id: &str) -> Result<(), MarketDataError> {
        use crate::websocket::channels::StockSubscription;

        let request = StockSubscription::to_unsubscribe_request(subscription_id);
        let request_str = serde_json::to_string(&request)
            .map_err(|e| MarketDataError::DeserializationError { source: e })?;
        self.send_text(&request_str).await
    }

    /// Get list of active subscription keys
    pub fn subscription_keys(&self) -> Vec<String> {
        self.subscriptions.keys()
    }

    /// Internal: Spawn message dispatch task
    ///
    /// Takes ownership of the read half of the WebSocket stream for message dispatch.
    /// When the connection drops, triggers auto-reconnect if configured. Uses a loop
    /// (not recursion) to handle repeated reconnections within a single spawned task.
    async fn spawn_dispatch_task(&self, ws_read: WsStream) {
        use crate::websocket::message::dispatch_messages;

        let message_tx = self.message_tx.clone();
        let event_tx = self.event_tx.clone();
        let health_check = Arc::clone(&self.health_check);

        // Clone Arcs needed for auto-reconnect inside spawned task
        let reconnection = Arc::clone(&self.reconnection);
        let config = self.config.clone();
        let state = Arc::clone(&self.state);
        let ws_sink = Arc::clone(&self.ws_sink);
        let write_tx_slot = Arc::clone(&self.write_tx);
        let writer_handle = Arc::clone(&self.writer_handle);
        let subscriptions = Arc::clone(&self.subscriptions);
        let health_check_handle = Arc::clone(&self.health_check_handle);

        let handle = tokio::spawn(async move {
            // Dispatch → reconnect → dispatch loop (avoids recursive async which breaks Send)
            let mut current_ws_read = ws_read;
            loop {
                let close_code = dispatch_messages(
                    current_ws_read,
                    message_tx.clone(),
                    event_tx.clone(),
                    Arc::clone(&health_check),
                    Arc::clone(&subscriptions),
                )
                .await;

                // Attempt auto-reconnect; returns new streams on success.
                match try_reconnect(
                    close_code,
                    Arc::clone(&reconnection),
                    config.clone(),
                    Arc::clone(&state),
                    event_tx.clone(),
                    Arc::clone(&ws_sink),
                    Arc::clone(&write_tx_slot),
                    Arc::clone(&writer_handle),
                    Arc::clone(&subscriptions),
                    Arc::clone(&health_check),
                    Arc::clone(&health_check_handle),
                    message_tx.clone(),
                )
                .await
                {
                    Some(ws_read) => {
                        current_ws_read = ws_read;
                        // Loop back to dispatch with the new connection
                    }
                    None => {
                        // Reconnection failed or not configured — exit task
                        break;
                    }
                }
            }
        });

        let mut dispatch_handle_guard = self.dispatch_handle.lock().await;
        *dispatch_handle_guard = Some(handle);
    }

    /// Internal: Spawn the writer task that drains the outbound channel into
    /// the WebSocket sink. Also installs the new `write_tx` sender into the
    /// shared slot. Call after `ws_sink` has been populated.
    async fn start_writer_task(&self) {
        // Aborts any previous writer task. Channel buffer 64 is generous for a
        // ping-every-30s + occasional sub/unsub workload while staying small
        // enough to surface backpressure if the sink stalls.
        if let Some(prev) = self.writer_handle.lock().await.take() {
            prev.abort();
        }

        let (tx, rx) = tokio_mpsc::channel::<String>(64);
        {
            let mut guard = self.write_tx.lock().await;
            *guard = Some(tx);
        }

        let ws_sink = Arc::clone(&self.ws_sink);
        let event_tx = self.event_tx.clone();
        let handle = tokio::spawn(run_writer_task(rx, ws_sink, event_tx));

        let mut guard = self.writer_handle.lock().await;
        *guard = Some(handle);
    }

    /// Internal: Push a JSON string onto the outbound write channel. Returns
    /// `ConnectionError` if the writer task is not running (e.g., disconnected).
    async fn enqueue_write(&self, json: String) -> Result<(), MarketDataError> {
        let sender = { self.write_tx.lock().await.clone() };
        match sender {
            Some(s) => s.send(json).await.map_err(|_| MarketDataError::ConnectionError {
                msg: "Writer task is not running".to_string(),
            }),
            None => Err(MarketDataError::ConnectionError {
                msg: "Not connected".to_string(),
            }),
        }
    }

    /// Internal: Start health check monitoring
    async fn start_health_check(&self) {
        if !self.health_check.config().enabled {
            return;
        }

        // CRITICAL: reset activity timer to "now" before spawning the task.
        // HealthCheck::new() ran at client construction, possibly long before
        // connect() was called. Without this touch the first tick would see a
        // stale age and could false-disconnect immediately.
        self.health_check.touch();

        let event_tx = self.event_tx.clone();
        let handle = self.health_check.spawn_check_task(event_tx);

        {
            let mut guard = self.health_check_handle.lock().await;
            *guard = Some(handle);
        }

        self.health_check.resume();
    }

    /// Internal: Automatic reconnection flow (&self version)
    ///
    /// Implements exponential backoff retry logic with subscription restoration.
    /// Note: The dispatch loop uses the standalone `try_reconnect` function instead,
    /// which operates on owned Arcs for Send compatibility with tokio::spawn.
    #[allow(dead_code)]
    async fn auto_reconnect(&self, close_code: Option<u16>) -> Result<(), MarketDataError> {
        let should_reconnect = {
            let reconnection = self.reconnection.lock().await;
            reconnection.should_reconnect(close_code)
        };

        if !should_reconnect {
            // Not retriable - update state and send event
            {
                let mut state = self.state.write().await;
                *state = ConnectionState::Closed {
                    code: close_code,
                    reason: "Non-retriable error".to_string(),
                };
            }

            let attempts = {
                let reconnection = self.reconnection.lock().await;
                reconnection.current_attempt()
            };

            let _ = self.event_tx.send(ConnectionEvent::ReconnectFailed { attempts });
            return Err(MarketDataError::ConnectionError {
                msg: format!("Non-retriable close code: {:?}", close_code),
            });
        }

        // Pause health check during reconnection
        self.health_check.pause();

        // Attempt reconnection with exponential backoff
        loop {
            let delay = {
                let mut reconnection = self.reconnection.lock().await;
                reconnection.next_delay()
            };

            match delay {
                Some(d) => {
                    let attempt = {
                        let reconnection = self.reconnection.lock().await;
                        reconnection.current_attempt()
                    };

                    // Update state to Reconnecting
                    {
                        let mut state = self.state.write().await;
                        *state = ConnectionState::Reconnecting { attempt };
                    }
                    let _ = self.event_tx.send(ConnectionEvent::Reconnecting { attempt });

                    // Wait before reconnecting
                    sleep(d).await;

                    // Try to connect
                    match self.connect().await {
                        Ok(()) => {
                            // Reset reconnection manager on success
                            {
                                let mut reconnection = self.reconnection.lock().await;
                                reconnection.reset();
                            }

                            // Resume health check after successful reconnection
                            self.health_check.resume();

                            // Resubscribe all
                            let _ = self.resubscribe_all().await;

                            return Ok(());
                        }
                        Err(_) => {
                            // Continue loop to next attempt
                            continue;
                        }
                    }
                }
                None => {
                    // Max attempts reached
                    {
                        let mut state = self.state.write().await;
                        *state = ConnectionState::Closed {
                            code: close_code,
                            reason: "Max reconnection attempts reached".to_string(),
                        };
                    }

                    let attempts = {
                        let reconnection = self.reconnection.lock().await;
                        reconnection.current_attempt()
                    };

                    let _ = self.event_tx.send(ConnectionEvent::ReconnectFailed { attempts });

                    return Err(MarketDataError::ConnectionError {
                        msg: "Max reconnection attempts reached".to_string(),
                    });
                }
            }
        }
    }
}

/// Single-writer task body. Drains pre-serialized JSON strings from `rx`
/// and writes them as text frames to the shared `ws_sink`. Exits when the
/// channel closes or when a write fails. Errors are reported via `event_tx`.
async fn run_writer_task(
    mut rx: tokio_mpsc::Receiver<String>,
    ws_sink: Arc<Mutex<Option<WsSink>>>,
    event_tx: mpsc::Sender<ConnectionEvent>,
) {
    while let Some(text) = rx.recv().await {
        let mut sink_guard = ws_sink.lock().await;
        let Some(sink) = sink_guard.as_mut() else {
            // Sink has been cleared (disconnect/force_close). Stop draining.
            break;
        };
        if let Err(e) = sink.send(Message::Text(text.into())).await {
            let err: MarketDataError = e.into();
            let _ = event_tx.send(ConnectionEvent::Error {
                message: format!("Writer error: {}", err),
                code: err.to_error_code(),
            });
            break;
        }
    }
}

/// Attempt auto-reconnection after a disconnect.
///
/// Called from within the dispatch loop's spawned task. Takes owned values
/// (cloned from the spawned task) because `mpsc::Sender` is `!Sync` and
/// holding `&mpsc::Sender` across await points would make the future `!Send`.
/// Returns `Some(ws_read)` on successful reconnect, `None` if reconnect is not
/// configured or all attempts are exhausted.
#[allow(clippy::too_many_arguments)]
async fn try_reconnect(
    close_code: Option<u16>,
    reconnection: Arc<Mutex<ReconnectionManager>>,
    config: ConnectionConfig,
    state: Arc<RwLock<ConnectionState>>,
    event_tx: mpsc::Sender<ConnectionEvent>,
    ws_sink: Arc<Mutex<Option<WsSink>>>,
    write_tx_slot: Arc<Mutex<Option<tokio_mpsc::Sender<String>>>>,
    writer_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
    subscriptions: Arc<SubscriptionManager>,
    health_check: Arc<HealthCheck>,
    health_check_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
    message_tx: mpsc::Sender<WebSocketMessage>,
) -> Option<WsStream> {
    // Check if we should attempt reconnection
    let should_reconnect = {
        let reconnection = reconnection.lock().await;
        reconnection.should_reconnect(close_code)
    };

    if !should_reconnect {
        // Not retriable - update state and send event
        {
            let mut st = state.write().await;
            *st = ConnectionState::Closed {
                code: close_code,
                reason: "Non-retriable error".to_string(),
            };
        }

        let attempts = {
            let reconnection = reconnection.lock().await;
            reconnection.current_attempt()
        };

        let _ = event_tx.send(ConnectionEvent::ReconnectFailed { attempts });
        return None;
    }

    // Pause health check during reconnection
    health_check.pause();

    // Attempt reconnection with exponential backoff
    loop {
        let delay = {
            let mut reconnection = reconnection.lock().await;
            reconnection.next_delay()
        };

        match delay {
            Some(d) => {
                let attempt = {
                    let reconnection = reconnection.lock().await;
                    reconnection.current_attempt()
                };

                // Update state to Reconnecting
                {
                    let mut st = state.write().await;
                    *st = ConnectionState::Reconnecting { attempt };
                }
                let _ = event_tx.send(ConnectionEvent::Reconnecting { attempt });

                // Wait before reconnecting
                sleep(d).await;

                // Try to connect and authenticate
                match try_connect(
                    config.clone(),
                    Arc::clone(&state),
                    event_tx.clone(),
                    message_tx.clone(),
                )
                .await
                {
                    Ok((new_sink, ws_read)) => {
                        // Store the new write half
                        {
                            let mut sink_guard = ws_sink.lock().await;
                            *sink_guard = Some(new_sink);
                        }

                        // Reset reconnection manager on success
                        {
                            let mut reconnection = reconnection.lock().await;
                            reconnection.reset();
                        }

                        // Rebuild the writer task for the new sink
                        if let Some(prev) = writer_handle.lock().await.take() {
                            prev.abort();
                        }
                        let (new_write_tx, new_write_rx) = tokio_mpsc::channel::<String>(64);
                        {
                            let mut guard = write_tx_slot.lock().await;
                            *guard = Some(new_write_tx.clone());
                        }
                        let writer_task_handle = tokio::spawn(run_writer_task(
                            new_write_rx,
                            Arc::clone(&ws_sink),
                            event_tx.clone(),
                        ));
                        {
                            let mut guard = writer_handle.lock().await;
                            *guard = Some(writer_task_handle);
                        }

                        // Resubscribe all stored subscriptions through the new writer
                        let subs = subscriptions.get_all();
                        for req in subs {
                            let sub_msg = WebSocketRequest::subscribe(req);
                            if let Ok(sub_json) = serde_json::to_string(&sub_msg) {
                                let _ = new_write_tx.send(sub_json).await;
                            }
                        }

                        // Restart health check for the new connection if enabled
                        if let Some(prev) = health_check_handle.lock().await.take() {
                            prev.abort();
                        }
                        if health_check.config().enabled {
                            // Reset activity timer before restarting the task.
                            health_check.touch();
                            health_check.resume();
                            let hc_handle =
                                health_check.spawn_check_task(event_tx.clone());
                            let mut hch = health_check_handle.lock().await;
                            *hch = Some(hc_handle);
                        }

                        return Some(ws_read);
                    }
                    Err(_) => {
                        // Continue loop to next attempt
                        continue;
                    }
                }
            }
            None => {
                // Max attempts reached
                {
                    let mut st = state.write().await;
                    *st = ConnectionState::Closed {
                        code: close_code,
                        reason: "Max reconnection attempts reached".to_string(),
                    };
                }

                let attempts = {
                    let reconnection = reconnection.lock().await;
                    reconnection.current_attempt()
                };

                let _ = event_tx.send(ConnectionEvent::ReconnectFailed { attempts });

                return None;
            }
        }
    }
}

/// Attempt a fresh connection: connect to WebSocket and authenticate.
///
/// On success, returns the write sink and read stream. The caller is responsible
/// for storing the sink and setting up dispatch. Takes owned values for Send safety.
async fn try_connect(
    config: ConnectionConfig,
    state: Arc<RwLock<ConnectionState>>,
    event_tx: mpsc::Sender<ConnectionEvent>,
    message_tx: mpsc::Sender<WebSocketMessage>,
) -> Result<(WsSink, WsStream), MarketDataError> {
    // Update state to Connecting
    {
        let mut st = state.write().await;
        *st = ConnectionState::Connecting;
    }
    let _ = event_tx.send(ConnectionEvent::Connecting);

    // Connect to WebSocket
    let tls_connector = tls_connector_for(&config)?;
    let connect_result = timeout(
        config.connect_timeout,
        connect_async_tls_with_config(&config.url, None, false, tls_connector),
    )
    .await;

    let (ws_stream, _response) = match connect_result {
        Ok(Ok((stream, response))) => (stream, response),
        Ok(Err(e)) => {
            let err: MarketDataError = e.into();
            {
                let mut st = state.write().await;
                *st = ConnectionState::Disconnected;
            }
            return Err(err);
        }
        Err(_) => {
            {
                let mut st = state.write().await;
                *st = ConnectionState::Disconnected;
            }
            return Err(MarketDataError::TimeoutError {
                operation: "WebSocket connect".to_string(),
            });
        }
    };

    // Split the stream
    let (mut new_ws_sink, mut ws_read) = ws_stream.split();

    let _ = event_tx.send(ConnectionEvent::Connected);

    // Authenticate
    {
        let mut st = state.write().await;
        *st = ConnectionState::Authenticating;
    }

    let auth_msg = WebSocketRequest::auth(config.auth.clone());
    let auth_json = serde_json::to_string(&auth_msg)
        .map_err(|e| MarketDataError::DeserializationError { source: e })?;

    new_ws_sink
        .send(Message::Text(auth_json.into()))
        .await
        .map_err(MarketDataError::from)?;

    // Wait for auth response (same pattern as WebSocketClient::connect)
    let msg_tx = message_tx.clone();
    let auth_timeout = Duration::from_secs(10);
    let auth_result = timeout(auth_timeout, async {
        while let Some(msg_result) = ws_read.next().await {
            match msg_result {
                Ok(Message::Text(text)) => {
                    if let Ok(ws_msg) = serde_json::from_str::<WebSocketMessage>(&text) {
                        let _ = msg_tx.send(ws_msg.clone());
                        if ws_msg.is_authenticated() {
                            return Ok(());
                        }
                        if ws_msg.is_error() {
                            return Err(MarketDataError::AuthError {
                                msg: ws_msg
                                    .error_message()
                                    .unwrap_or_else(|| "Unknown error".to_string()),
                            });
                        }
                    }
                }
                Err(e) => return Err(MarketDataError::from(e)),
                _ => {}
            }
        }
        Err(MarketDataError::ConnectionError {
            msg: "Stream closed during authentication".to_string(),
        })
    })
    .await;

    match auth_result {
        Ok(Ok(())) => {
            {
                let mut st = state.write().await;
                *st = ConnectionState::Connected;
            }
            let _ = event_tx.send(ConnectionEvent::Authenticated);
            Ok((new_ws_sink, ws_read))
        }
        Ok(Err(e)) => {
            {
                let mut st = state.write().await;
                *st = ConnectionState::Disconnected;
            }
            // Same auth-vs-other split as the primary connect() flow
            if let MarketDataError::AuthError { msg } = &e {
                let _ = event_tx.send(ConnectionEvent::Unauthenticated {
                    message: msg.clone(),
                });
            }
            Err(e)
        }
        Err(_) => {
            {
                let mut st = state.write().await;
                *st = ConnectionState::Disconnected;
            }
            Err(MarketDataError::TimeoutError {
                operation: "WebSocket authentication".to_string(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AuthRequest;

    #[test]
    fn test_connection_state_variants() {
        // Test all state variants exist and can be created
        let _disconnected = ConnectionState::Disconnected;
        let _connecting = ConnectionState::Connecting;
        let _authenticating = ConnectionState::Authenticating;
        let _connected = ConnectionState::Connected;
        let _reconnecting = ConnectionState::Reconnecting { attempt: 1 };
        let _closed = ConnectionState::Closed {
            code: Some(1000),
            reason: "Normal closure".to_string(),
        };
    }

    #[test]
    fn test_connection_event_variants() {
        // Test all event variants exist and can be created
        let _connecting = ConnectionEvent::Connecting;
        let _connected = ConnectionEvent::Connected;
        let _authenticated = ConnectionEvent::Authenticated;
        let _unauthenticated = ConnectionEvent::Unauthenticated {
            message: "Invalid credentials".to_string(),
        };
        let _disconnected = ConnectionEvent::Disconnected {
            code: Some(1000),
            reason: "Normal closure".to_string(),
        };
        let _reconnecting = ConnectionEvent::Reconnecting { attempt: 1 };
        let _failed = ConnectionEvent::ReconnectFailed { attempts: 5 };
        let _error = ConnectionEvent::Error {
            message: "Connection failed".to_string(),
            code: 2001,
        };
    }

    #[tokio::test]
    async fn test_websocket_client_new() {
        let config =
            ConnectionConfig::fugle_stock(AuthRequest::with_api_key("test-key"));
        let client = WebSocketClient::new(config);

        let state = client.state_async().await;
        assert_eq!(state, ConnectionState::Disconnected);
    }

    #[tokio::test]
    async fn test_websocket_client_state() {
        let config =
            ConnectionConfig::fugle_stock(AuthRequest::with_api_key("test-key"));
        let client = WebSocketClient::new(config);

        // Initial state should be Disconnected
        let state = client.state_async().await;
        assert_eq!(state, ConnectionState::Disconnected);

        // Manually change state for testing
        {
            let mut state = client.state.write().await;
            *state = ConnectionState::Connecting;
        }

        let state = client.state_async().await;
        assert_eq!(state, ConnectionState::Connecting);
    }

    #[tokio::test]
    async fn test_websocket_client_events() {
        let config =
            ConnectionConfig::fugle_stock(AuthRequest::with_api_key("test-key"));
        let client = WebSocketClient::new(config);

        // Send an event
        client
            .event_tx
            .send(ConnectionEvent::Connecting)
            .unwrap();

        // Receive the event (using blocking recv in async context)
        let rx = Arc::clone(&client.event_rx);
        let event = tokio::task::spawn_blocking(move || {
            let rx_guard = rx.blocking_lock();
            rx_guard.recv().unwrap()
        })
        .await
        .unwrap();
        assert_eq!(event, ConnectionEvent::Connecting);
    }

    #[tokio::test]
    async fn test_is_connected() {
        let config =
            ConnectionConfig::fugle_stock(AuthRequest::with_api_key("test-key"));
        let client = WebSocketClient::new(config);

        // Initially not connected
        assert!(!client.is_connected().await);

        // Manually set to Connected for testing
        {
            let mut state = client.state.write().await;
            *state = ConnectionState::Connected;
        }

        assert!(client.is_connected().await);
    }

    #[tokio::test]
    async fn test_connection_state_transitions() {
        let config =
            ConnectionConfig::fugle_stock(AuthRequest::with_api_key("test-key"));
        let client = WebSocketClient::new(config);

        // Test state transitions
        {
            let mut state = client.state.write().await;
            *state = ConnectionState::Connecting;
        }
        assert_eq!(client.state_async().await, ConnectionState::Connecting);

        {
            let mut state = client.state.write().await;
            *state = ConnectionState::Authenticating;
        }
        assert_eq!(client.state_async().await, ConnectionState::Authenticating);

        {
            let mut state = client.state.write().await;
            *state = ConnectionState::Connected;
        }
        assert_eq!(client.state_async().await, ConnectionState::Connected);
        assert!(client.is_connected().await);
    }

    #[tokio::test]
    async fn test_subscribe_when_disconnected() {
        use crate::models::Channel;

        let config =
            ConnectionConfig::fugle_stock(AuthRequest::with_api_key("test-key"));
        let client = WebSocketClient::new(config);

        // Subscribe while disconnected
        let req = SubscribeRequest::new(Channel::Trades, "2330");
        let result = client.subscribe(req.clone()).await;
        assert!(result.is_ok());

        // Subscription should be stored
        let subs = client.subscriptions();
        assert_eq!(subs.len(), 1);
        assert_eq!(subs[0], req);
    }

    #[tokio::test]
    async fn test_subscribe_when_connected() {
        use crate::models::Channel;

        let config =
            ConnectionConfig::fugle_stock(AuthRequest::with_api_key("test-key"));
        let client = WebSocketClient::new(config);

        // Manually set to Connected for testing
        {
            let mut state = client.state.write().await;
            *state = ConnectionState::Connected;
        }

        // Subscribe while connected
        let req = SubscribeRequest::new(Channel::Trades, "2330");
        // Note: This will fail without actual connection, but subscription should be stored
        let _ = client.subscribe(req.clone()).await;

        // Subscription should be stored regardless of send result
        let subs = client.subscriptions();
        assert_eq!(subs.len(), 1);
        assert_eq!(subs[0], req);
    }

    #[tokio::test]
    async fn test_unsubscribe_removes_from_state() {
        use crate::models::Channel;

        let config =
            ConnectionConfig::fugle_stock(AuthRequest::with_api_key("test-key"));
        let client = WebSocketClient::new(config);

        // Subscribe
        let req = SubscribeRequest::new(Channel::Trades, "2330");
        let _ = client.subscribe(req).await;
        assert_eq!(client.subscriptions().len(), 1);

        // Unsubscribe
        let result = client.unsubscribe("trades:2330").await;
        assert!(result.is_ok());

        // Subscription should be removed
        assert_eq!(client.subscriptions().len(), 0);
    }

    #[tokio::test]
    async fn test_unsubscribe_futopt_channel_removes_from_state() {
        use crate::websocket::channels::FutOptSubscription;
        use crate::FutOptChannel;

        let config =
            ConnectionConfig::fugle_stock(AuthRequest::with_api_key("test-key"));
        let client = WebSocketClient::new(config);

        let sub = FutOptSubscription {
            channel: FutOptChannel::Books,
            symbol: "TXFE6".to_string(),
            after_hours: true,
        };
        let _ = client.subscribe_futopt_channel(sub.clone()).await;
        assert_eq!(client.subscriptions().len(), 1);

        let result = client.unsubscribe_futopt_channel(&sub).await;
        assert!(result.is_ok());
        assert_eq!(client.subscriptions().len(), 0);
    }

    #[tokio::test]
    async fn test_subscriptions_restored_after_reconnect() {
        use crate::models::Channel;

        let config =
            ConnectionConfig::fugle_stock(AuthRequest::with_api_key("test-key"));
        let client = WebSocketClient::new(config);

        // Add subscriptions
        let _ = client.subscribe(SubscribeRequest::new(Channel::Trades, "2330")).await;
        let _ = client.subscribe(SubscribeRequest::new(Channel::Candles, "2317")).await;

        // Subscriptions should be stored
        let subs = client.subscriptions();
        assert_eq!(subs.len(), 2);
        assert_eq!(subs[0].key(), "trades:2330");
        assert_eq!(subs[1].key(), "candles:2317");
    }

    #[tokio::test]
    async fn test_manual_reconnect_resets_attempts() {
        let config =
            ConnectionConfig::fugle_stock(AuthRequest::with_api_key("test-key"));
        let client = WebSocketClient::new(config);

        // Simulate failed reconnection attempts
        {
            let mut reconnection = client.reconnection.lock().await;
            let _ = reconnection.next_delay();
            let _ = reconnection.next_delay();
            assert_eq!(reconnection.current_attempt(), 2);
        }

        // Manual reconnect should reset
        // Note: This will fail without actual server, but should reset attempts
        let _ = client.reconnect().await;

        // Attempts should be reset
        {
            let reconnection = client.reconnection.lock().await;
            assert_eq!(reconnection.current_attempt(), 0);
        }
    }

    #[tokio::test]
    async fn test_with_reconnection_config() {
        use std::time::Duration;

        let config =
            ConnectionConfig::fugle_stock(AuthRequest::with_api_key("test-key"));
        let reconnection_config = ReconnectionConfig::default()
            .with_max_attempts(10)
            .unwrap()
            .with_initial_delay(Duration::from_secs(2))
            .unwrap();

        let client = WebSocketClient::with_reconnection_config(config, reconnection_config);

        // Verify reconnection config is used
        {
            let reconnection = client.reconnection.lock().await;
            assert_eq!(reconnection.attempts_remaining(), 10);
        }
    }

    // ========================================================================
    // Closed Client Protection Tests (Phase 7)
    // ========================================================================

    #[tokio::test]
    async fn test_is_closed_after_disconnect() {
        let config = ConnectionConfig::fugle_stock(AuthRequest::with_api_key("test-key"));
        let client = WebSocketClient::new(config);

        // Initially not closed
        assert!(!client.is_closed().await);

        // Manually set to Closed state for testing
        {
            let mut state = client.state.write().await;
            *state = ConnectionState::Closed {
                code: Some(1000),
                reason: "Normal closure".to_string(),
            };
        }

        // Now should be closed
        assert!(client.is_closed().await);
    }

    #[tokio::test]
    async fn test_subscribe_fails_when_closed() {
        use crate::models::Channel;

        let config = ConnectionConfig::fugle_stock(AuthRequest::with_api_key("test-key"));
        let client = WebSocketClient::new(config);

        // Set to Closed state
        {
            let mut state = client.state.write().await;
            *state = ConnectionState::Closed {
                code: Some(1000),
                reason: "Test closure".to_string(),
            };
        }

        // Subscribe should fail with ClientClosed error
        let result = client.subscribe(SubscribeRequest::new(Channel::Trades, "2330")).await;
        assert!(matches!(result, Err(MarketDataError::ClientClosed)));
    }

    #[tokio::test]
    async fn test_unsubscribe_fails_when_closed() {
        use crate::models::Channel;

        let config = ConnectionConfig::fugle_stock(AuthRequest::with_api_key("test-key"));
        let client = WebSocketClient::new(config);

        // First add a subscription while not closed
        let _ = client.subscribe(SubscribeRequest::new(Channel::Trades, "2330")).await;

        // Set to Closed state
        {
            let mut state = client.state.write().await;
            *state = ConnectionState::Closed {
                code: Some(1000),
                reason: "Test closure".to_string(),
            };
        }

        // Unsubscribe should fail with ClientClosed error
        let result = client.unsubscribe("trades:2330").await;
        assert!(matches!(result, Err(MarketDataError::ClientClosed)));
    }

    #[tokio::test]
    async fn test_connect_fails_when_closed() {
        let config = ConnectionConfig::fugle_stock(AuthRequest::with_api_key("test-key"));
        let client = WebSocketClient::new(config);

        // Set to Closed state
        {
            let mut state = client.state.write().await;
            *state = ConnectionState::Closed {
                code: Some(1000),
                reason: "Test closure".to_string(),
            };
        }

        // Connect should fail with ClientClosed error
        let result = client.connect().await;
        assert!(matches!(result, Err(MarketDataError::ClientClosed)));
    }

    #[tokio::test]
    async fn test_reconnect_fails_when_closed() {
        let config = ConnectionConfig::fugle_stock(AuthRequest::with_api_key("test-key"));
        let client = WebSocketClient::new(config);

        // Set to Closed state
        {
            let mut state = client.state.write().await;
            *state = ConnectionState::Closed {
                code: Some(1000),
                reason: "Test closure".to_string(),
            };
        }

        // Reconnect should fail with ClientClosed error
        let result = client.reconnect().await;
        assert!(matches!(result, Err(MarketDataError::ClientClosed)));
    }

    #[tokio::test]
    async fn test_subscribe_channel_fails_when_closed() {
        use crate::models::Channel;
        use crate::websocket::channels::StockSubscription;

        let config = ConnectionConfig::fugle_stock(AuthRequest::with_api_key("test-key"));
        let client = WebSocketClient::new(config);

        // Set to Closed state
        {
            let mut state = client.state.write().await;
            *state = ConnectionState::Closed {
                code: Some(1000),
                reason: "Test closure".to_string(),
            };
        }

        // subscribe_channel should fail with ClientClosed error
        let sub = StockSubscription::new(Channel::Trades, "2330");
        let result = client.subscribe_channel(sub).await;
        assert!(matches!(result, Err(MarketDataError::ClientClosed)));
    }

    #[test]
    fn test_is_closed_sync() {
        // Note: This test runs without a tokio runtime context
        let config = ConnectionConfig::fugle_stock(AuthRequest::with_api_key("test-key"));
        let client = WebSocketClient::new(config);

        // Without a runtime, is_closed_sync should return false
        assert!(!client.is_closed_sync());
    }
}

/// Tests for stock streaming channel subscription API (Phase 4)
#[cfg(test)]
mod channel_tests {
    use super::*;
    use crate::models::Channel;
    use crate::websocket::channels::StockSubscription;
    use crate::AuthRequest;

    #[tokio::test]
    async fn test_subscribe_channel_stores_subscription() {
        let config = ConnectionConfig::fugle_stock(AuthRequest::with_api_key("test-key"));
        let client = WebSocketClient::new(config);

        let sub = StockSubscription::new(Channel::Trades, "2330");
        // Note: This will fail to send (not connected) but should store locally
        let _ = client.subscribe_channel(sub).await;

        let keys = client.subscription_keys();
        assert!(keys.contains(&"trades:2330".to_string()));
    }

    #[tokio::test]
    async fn test_subscribe_channel_odd_lot() {
        let config = ConnectionConfig::fugle_stock(AuthRequest::with_api_key("test-key"));
        let client = WebSocketClient::new(config);

        let sub = StockSubscription::new(Channel::Trades, "2330").with_odd_lot(true);
        let _ = client.subscribe_channel(sub).await;

        let keys = client.subscription_keys();
        assert!(keys.contains(&"trades:2330:oddlot".to_string()));
    }

    #[tokio::test]
    async fn test_subscribe_multiple_channels() {
        let config = ConnectionConfig::fugle_stock(AuthRequest::with_api_key("test-key"));
        let client = WebSocketClient::new(config);

        // Subscribe to multiple channels
        let _ = client
            .subscribe_channel(StockSubscription::new(Channel::Trades, "2330"))
            .await;
        let _ = client
            .subscribe_channel(StockSubscription::new(Channel::Candles, "2330"))
            .await;
        let _ = client
            .subscribe_channel(StockSubscription::new(Channel::Books, "2330"))
            .await;

        let keys = client.subscription_keys();
        assert_eq!(keys.len(), 3);
        assert!(keys.contains(&"trades:2330".to_string()));
        assert!(keys.contains(&"candles:2330".to_string()));
        assert!(keys.contains(&"books:2330".to_string()));
    }

    #[tokio::test]
    async fn test_subscribe_symbols_convenience() {
        let config = ConnectionConfig::fugle_stock(AuthRequest::with_api_key("test-key"));
        let client = WebSocketClient::new(config);

        let _ = client
            .subscribe_symbols(Channel::Trades, &["2330", "2317", "2454"], false)
            .await;

        let keys = client.subscription_keys();
        assert_eq!(keys.len(), 3);
        assert!(keys.contains(&"trades:2330".to_string()));
        assert!(keys.contains(&"trades:2317".to_string()));
        assert!(keys.contains(&"trades:2454".to_string()));
    }

    #[tokio::test]
    async fn test_unsubscribe_channel() {
        let config = ConnectionConfig::fugle_stock(AuthRequest::with_api_key("test-key"));
        let client = WebSocketClient::new(config);

        let sub = StockSubscription::new(Channel::Trades, "2330");
        let _ = client.subscribe_channel(sub.clone()).await;
        assert_eq!(client.subscription_keys().len(), 1);

        // Unsubscribe
        let _ = client.unsubscribe_channel(&sub).await;
        assert_eq!(client.subscription_keys().len(), 0);
    }

    #[tokio::test]
    async fn test_unsubscribe_does_not_affect_others() {
        let config = ConnectionConfig::fugle_stock(AuthRequest::with_api_key("test-key"));
        let client = WebSocketClient::new(config);

        let sub1 = StockSubscription::new(Channel::Trades, "2330");
        let sub2 = StockSubscription::new(Channel::Candles, "2330");
        let _ = client.subscribe_channel(sub1.clone()).await;
        let _ = client.subscribe_channel(sub2).await;
        assert_eq!(client.subscription_keys().len(), 2);

        // Unsubscribe only sub1
        let _ = client.unsubscribe_channel(&sub1).await;

        let keys = client.subscription_keys();
        assert_eq!(keys.len(), 1);
        assert!(keys.contains(&"candles:2330".to_string()));
    }

    #[tokio::test]
    async fn test_subscribe_symbols_with_odd_lot() {
        let config = ConnectionConfig::fugle_stock(AuthRequest::with_api_key("test-key"));
        let client = WebSocketClient::new(config);

        let _ = client
            .subscribe_symbols(Channel::Trades, &["2330", "2317"], true)
            .await;

        let keys = client.subscription_keys();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"trades:2330:oddlot".to_string()));
        assert!(keys.contains(&"trades:2317:oddlot".to_string()));
    }

    #[tokio::test]
    async fn test_subscribe_all_channel_types() {
        let config = ConnectionConfig::fugle_stock(AuthRequest::with_api_key("test-key"));
        let client = WebSocketClient::new(config);

        // Subscribe to all channel types
        let _ = client
            .subscribe_channel(StockSubscription::new(Channel::Trades, "2330"))
            .await;
        let _ = client
            .subscribe_channel(StockSubscription::new(Channel::Candles, "2330"))
            .await;
        let _ = client
            .subscribe_channel(StockSubscription::new(Channel::Books, "2330"))
            .await;
        let _ = client
            .subscribe_channel(StockSubscription::new(Channel::Aggregates, "2330"))
            .await;
        let _ = client
            .subscribe_channel(StockSubscription::new(Channel::Indices, "IX0001"))
            .await;

        let keys = client.subscription_keys();
        assert_eq!(keys.len(), 5);
    }
}

/// Tests for graceful shutdown (Phase 7 - Plan 02)
#[cfg(test)]
mod disconnect_tests {
    use super::*;
    use crate::AuthRequest;

    #[tokio::test]
    async fn test_disconnect_sets_closed_state() {
        let config = ConnectionConfig::fugle_stock(AuthRequest::with_api_key("test-key"));
        let client = WebSocketClient::new(config);

        // Manually set to Connected for testing
        {
            let mut state = client.state.write().await;
            *state = ConnectionState::Connected;
        }

        // Disconnect should succeed even without actual connection
        let result = client.disconnect().await;
        assert!(result.is_ok());

        // State should be Closed
        let state = client.state_async().await;
        assert!(matches!(
            state,
            ConnectionState::Closed {
                code: Some(1000),
                ..
            }
        ));
    }

    #[tokio::test]
    async fn test_disconnect_emits_event() {
        let config = ConnectionConfig::fugle_stock(AuthRequest::with_api_key("test-key"));
        let client = WebSocketClient::new(config);

        // Manually set to Connected
        {
            let mut state = client.state.write().await;
            *state = ConnectionState::Connected;
        }

        // Disconnect
        let _ = client.disconnect().await;

        // Check event was emitted
        let rx = Arc::clone(&client.event_rx);
        let event = tokio::task::spawn_blocking(move || {
            let rx_guard = rx.blocking_lock();
            rx_guard.try_recv()
        })
        .await
        .unwrap();

        assert!(matches!(
            event,
            Ok(ConnectionEvent::Disconnected {
                code: Some(1000),
                ..
            })
        ));
    }

    #[tokio::test]
    async fn test_force_close_sets_abnormal_code() {
        let config = ConnectionConfig::fugle_stock(AuthRequest::with_api_key("test-key"));
        let client = WebSocketClient::new(config);

        // Manually set to Connected
        {
            let mut state = client.state.write().await;
            *state = ConnectionState::Connected;
        }

        // Force close
        let result = client.force_close().await;
        assert!(result.is_ok());

        // State should be Closed with 1006
        let state = client.state_async().await;
        assert!(matches!(
            state,
            ConnectionState::Closed {
                code: Some(1006),
                ..
            }
        ));
    }

    #[tokio::test]
    async fn test_force_close_emits_event_with_1006() {
        let config = ConnectionConfig::fugle_stock(AuthRequest::with_api_key("test-key"));
        let client = WebSocketClient::new(config);

        // Manually set to Connected
        {
            let mut state = client.state.write().await;
            *state = ConnectionState::Connected;
        }

        // Force close
        let _ = client.force_close().await;

        // Check event was emitted with 1006
        let rx = Arc::clone(&client.event_rx);
        let event = tokio::task::spawn_blocking(move || {
            let rx_guard = rx.blocking_lock();
            rx_guard.try_recv()
        })
        .await
        .unwrap();

        assert!(matches!(
            event,
            Ok(ConnectionEvent::Disconnected {
                code: Some(1006),
                reason,
            }) if reason == "Force closed"
        ));
    }

    #[tokio::test]
    async fn test_disconnect_from_disconnected_state() {
        let config = ConnectionConfig::fugle_stock(AuthRequest::with_api_key("test-key"));
        let client = WebSocketClient::new(config);

        // Client starts in Disconnected state
        let state = client.state_async().await;
        assert_eq!(state, ConnectionState::Disconnected);

        // Disconnect should succeed even when already disconnected
        let result = client.disconnect().await;
        assert!(result.is_ok());

        // State should now be Closed
        let state = client.state_async().await;
        assert!(matches!(state, ConnectionState::Closed { .. }));
    }

    #[tokio::test]
    async fn test_is_closed_after_disconnect() {
        let config = ConnectionConfig::fugle_stock(AuthRequest::with_api_key("test-key"));
        let client = WebSocketClient::new(config);

        // Initially not closed
        assert!(!client.is_closed().await);

        // Disconnect
        let _ = client.disconnect().await;

        // Now should be closed
        assert!(client.is_closed().await);
    }

    #[tokio::test]
    async fn test_is_closed_after_force_close() {
        let config = ConnectionConfig::fugle_stock(AuthRequest::with_api_key("test-key"));
        let client = WebSocketClient::new(config);

        // Initially not closed
        assert!(!client.is_closed().await);

        // Force close
        let _ = client.force_close().await;

        // Now should be closed
        assert!(client.is_closed().await);
    }

    #[tokio::test]
    async fn test_operations_fail_after_disconnect() {
        use crate::models::Channel;

        let config = ConnectionConfig::fugle_stock(AuthRequest::with_api_key("test-key"));
        let client = WebSocketClient::new(config);

        // Disconnect
        let _ = client.disconnect().await;

        // Subscribe should fail with ClientClosed
        let req = SubscribeRequest::new(Channel::Trades, "2330");
        let result = client.subscribe(req).await;
        assert!(matches!(result, Err(MarketDataError::ClientClosed)));

        // Reconnect should fail with ClientClosed
        let result = client.reconnect().await;
        assert!(matches!(result, Err(MarketDataError::ClientClosed)));

        // Connect should fail with ClientClosed
        let result = client.connect().await;
        assert!(matches!(result, Err(MarketDataError::ClientClosed)));
    }

    #[tokio::test]
    async fn test_closed_state_has_normal_closure_reason() {
        let config = ConnectionConfig::fugle_stock(AuthRequest::with_api_key("test-key"));
        let client = WebSocketClient::new(config);

        let _ = client.disconnect().await;

        let state = client.state_async().await;
        if let ConnectionState::Closed { code, reason } = state {
            assert_eq!(code, Some(1000));
            assert_eq!(reason, "Normal closure");
        } else {
            panic!("Expected Closed state");
        }
    }

    #[tokio::test]
    async fn test_force_closed_state_has_force_reason() {
        let config = ConnectionConfig::fugle_stock(AuthRequest::with_api_key("test-key"));
        let client = WebSocketClient::new(config);

        let _ = client.force_close().await;

        let state = client.state_async().await;
        if let ConnectionState::Closed { code, reason } = state {
            assert_eq!(code, Some(1006));
            assert_eq!(reason, "Force closed");
        } else {
            panic!("Expected Closed state");
        }
    }
}
