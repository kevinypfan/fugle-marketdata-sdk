//! Python WebSocket client wrapper
//!
//! Provides Python-friendly interface to marketdata-core WebSocket streaming.
//! Supports callback-based event handling and iterator-based message consumption.
//!
//! # Example (Python)
//!
//! ```python
//! from marketdata_py import WebSocketClient
//!
//! # Create client with API key
//! ws = WebSocketClient("your-api-key")
//!
//! # Callback mode: ws.stock.on("message", handler)
//! def on_message(msg):
//!     print(f"Received: {msg}")
//!
//! ws.stock.on("message", on_message)
//! ws.stock.connect()
//! ws.stock.subscribe("trades", "2330")
//!
//! # Or iterator mode:
//! for msg in ws.stock.messages():
//!     print(msg)
//! ```

use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3_async_runtimes::tokio::future_into_py;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::callback::CallbackRegistry;
use crate::errors;

/// Auto-reconnect configuration
///
/// Controls automatic reconnection behavior when connection is lost.
///
/// # Example (Python)
///
/// ```python
/// from marketdata_py import ReconnectConfig
///
/// config = ReconnectConfig(
///     enabled=True,
///     max_attempts=5,
///     initial_delay_ms=1000,
///     max_delay_ms=30000
/// )
/// ```
#[pyclass]
#[derive(Clone)]
pub struct ReconnectConfig {
    /// Whether auto-reconnect is enabled
    #[pyo3(get)]
    pub enabled: bool,
    /// Maximum number of reconnection attempts
    #[pyo3(get)]
    pub max_attempts: u32,
    /// Initial delay in milliseconds for exponential backoff
    #[pyo3(get)]
    pub initial_delay_ms: u64,
    /// Maximum delay in milliseconds (caps exponential backoff)
    #[pyo3(get)]
    pub max_delay_ms: u64,
}

#[pymethods]
impl ReconnectConfig {
    /// Create a new reconnect configuration
    ///
    /// Args:
    ///     enabled: Whether auto-reconnect is enabled (default: True)
    ///     max_attempts: Maximum reconnection attempts (default: 5, min: 1)
    ///     initial_delay_ms: Initial delay for exponential backoff (default: 1000ms, min: 100ms)
    ///     max_delay_ms: Maximum delay cap (default: 60000ms = 60s)
    ///
    /// Raises:
    ///     ValueError: If validation fails
    #[new]
    #[pyo3(signature = (*, enabled=true, max_attempts=5, initial_delay_ms=1000, max_delay_ms=60000))]
    pub fn new(
        enabled: bool,
        max_attempts: u32,
        initial_delay_ms: u64,
        max_delay_ms: u64,
    ) -> PyResult<Self> {
        // Validate using core's validation logic (fail fast)
        let _ = marketdata_core::ReconnectionConfig::new(
            max_attempts,
            Duration::from_millis(initial_delay_ms),
            Duration::from_millis(max_delay_ms),
        )
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

        Ok(Self {
            enabled,
            max_attempts,
            initial_delay_ms,
            max_delay_ms,
        })
    }

    /// Create a default reconnect configuration (enabled with 5 attempts)
    #[staticmethod]
    pub fn default_config() -> Self {
        Self {
            enabled: true,
            max_attempts: 5,
            initial_delay_ms: 1000,
            max_delay_ms: 60000,
        }
    }

    /// Create a disabled reconnect configuration
    #[staticmethod]
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            max_attempts: 5,
            initial_delay_ms: 1000,
            max_delay_ms: 60000,
        }
    }
}

impl ReconnectConfig {
    /// Convert to core ReconnectionConfig
    ///
    /// This should not fail since validation already happened in __new__
    pub fn to_core(&self) -> marketdata_core::ReconnectionConfig {
        marketdata_core::ReconnectionConfig::new(
            self.max_attempts,
            Duration::from_millis(self.initial_delay_ms),
            Duration::from_millis(self.max_delay_ms),
        )
        .expect("Config already validated in constructor")
    }
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_attempts: 5,
            initial_delay_ms: 1000,
            max_delay_ms: 60000,
        }
    }
}

/// Health check configuration for WebSocket connections
///
/// Configures ping/pong based connection monitoring.
///
/// # Example (Python)
///
/// ```python
/// from marketdata_py import HealthCheckConfig, WebSocketClient
///
/// # Custom health check
/// health_check = HealthCheckConfig(
///     enabled=True,
///     ping_interval=15000,  # 15 seconds
///     max_missed_pongs=3
/// )
///
/// ws = WebSocketClient(
///     api_key="your-key",
///     health_check=health_check
/// )
/// ```
#[pyclass]
#[derive(Clone)]
pub struct HealthCheckConfig {
    /// Whether health check is enabled
    #[pyo3(get)]
    pub enabled: bool,
    /// Interval between ping messages in milliseconds (named to match the old `fugle-marketdata` SDK)
    #[pyo3(get)]
    pub ping_interval: u64,
    /// Maximum missed pongs before disconnect
    #[pyo3(get)]
    pub max_missed_pongs: u64,
}

#[pymethods]
impl HealthCheckConfig {
    /// Create a new health check configuration
    ///
    /// Args:
    ///     enabled: Whether health check is enabled (default: False)
    ///     ping_interval: Interval between pings in milliseconds (default: 30000, min: 5000)
    ///     max_missed_pongs: Max missed pongs before disconnect (default: 2, min: 1)
    ///
    /// Raises:
    ///     ValueError: If validation fails (interval < 5000ms or max_missed_pongs == 0)
    ///
    /// Example:
    ///     ```python
    ///     # Default config (disabled)
    ///     config = HealthCheckConfig()
    ///
    ///     # Enabled with custom settings
    ///     config = HealthCheckConfig(enabled=True, ping_interval=15000, max_missed_pongs=3)
    ///     ```
    #[new]
    #[pyo3(signature = (*, enabled=false, ping_interval=30000, max_missed_pongs=2))]
    pub fn new(enabled: bool, ping_interval: u64, max_missed_pongs: u64) -> PyResult<Self> {
        // Validate using core's validation logic (fail fast)
        let duration = Duration::from_millis(ping_interval);
        let _ = marketdata_core::HealthCheckConfig::new(enabled, duration, max_missed_pongs)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

        Ok(Self {
            enabled,
            ping_interval,
            max_missed_pongs,
        })
    }
}

impl HealthCheckConfig {
    /// Convert to core HealthCheckConfig
    ///
    /// This should not fail since validation already happened in __new__
    pub fn to_core(&self) -> marketdata_core::HealthCheckConfig {
        marketdata_core::HealthCheckConfig::new(
            self.enabled,
            Duration::from_millis(self.ping_interval),
            self.max_missed_pongs,
        )
        .expect("Config already validated in constructor")
    }
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            ping_interval: 30000,
            max_missed_pongs: 2,
        }
    }
}

/// Python WebSocket client for Fugle market data streaming
///
/// # Example (Python)
///
/// ```python
/// from marketdata_py import WebSocketClient, ReconnectConfig, HealthCheckConfig
///
/// # Create client with API key
/// ws = WebSocketClient(api_key="your-api-key")
///
/// # With custom reconnect config
/// rc = ReconnectConfig(max_attempts=10)
/// ws = WebSocketClient(api_key="your-key", reconnect=rc)
///
/// # Access stock streaming
/// ws.stock.connect()
/// ws.stock.subscribe("trades", "2330")
/// ```
#[pyclass]
pub struct WebSocketClient {
    api_key: String,
    base_url: Option<String>,
    reconnect_config: ReconnectConfig,
    health_check_config: HealthCheckConfig,
}

#[pymethods]
impl WebSocketClient {
    /// Create a new WebSocket client with authentication
    ///
    /// Provide exactly one authentication method:
    ///   - api_key: Your Fugle API key
    ///   - bearer_token: Bearer token for authentication
    ///   - sdk_token: SDK token for authentication
    ///
    /// Optional configuration:
    ///   - base_url: Custom base URL for WebSocket endpoint
    ///   - reconnect: ReconnectConfig for auto-reconnection behavior
    ///   - health_check: HealthCheckConfig for connection monitoring
    ///
    /// Returns:
    ///     A new WebSocketClient instance
    ///
    /// Raises:
    ///     ValueError: If zero or multiple auth methods provided
    ///
    /// Example:
    ///     ```python
    ///     # Simple usage
    ///     ws = WebSocketClient(api_key="your-key")
    ///
    ///     # With custom reconnect
    ///     rc = ReconnectConfig(max_attempts=10)
    ///     ws = WebSocketClient(api_key="key", reconnect=rc)
    ///
    ///     # With health check
    ///     hc = HealthCheckConfig(enabled=True, interval_ms=15000)
    ///     ws = WebSocketClient(api_key="key", health_check=hc)
    ///     ```
    #[new]
    #[pyo3(signature = (*, api_key=None, bearer_token=None, sdk_token=None, base_url=None, reconnect=None, health_check=None))]
    pub fn new(
        api_key: Option<String>,
        bearer_token: Option<String>,
        sdk_token: Option<String>,
        base_url: Option<String>,
        reconnect: Option<&Bound<'_, ReconnectConfig>>,
        health_check: Option<&Bound<'_, HealthCheckConfig>>,
    ) -> PyResult<Self> {
        // Validate exactly one auth method (fail fast)
        let auth_count = [&api_key, &bearer_token, &sdk_token]
            .iter()
            .filter(|opt| opt.is_some())
            .count();

        if auth_count != 1 {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "Provide exactly one of: api_key, bearer_token, sdk_token"
            ));
        }

        // Extract the auth key (for now, WebSocket uses string key internally)
        let auth_key = api_key.or(bearer_token).or(sdk_token).unwrap();

        // Extract configs with defaults (clone from Bound to avoid lifetime issues)
        let reconnect_config = if let Some(cfg) = reconnect {
            cfg.borrow().clone()
        } else {
            ReconnectConfig::default()
        };

        let health_check_config = if let Some(cfg) = health_check {
            cfg.borrow().clone()
        } else {
            HealthCheckConfig::default()
        };

        Ok(Self {
            api_key: auth_key,
            base_url,
            reconnect_config,
            health_check_config,
        })
    }

    /// Access stock market data WebSocket streaming
    ///
    /// Returns:
    ///     StockWebSocketClient for stock streaming with inherited config
    #[getter]
    pub fn stock(&self) -> StockWebSocketClient {
        StockWebSocketClient::new(
            self.api_key.clone(),
            self.reconnect_config.clone(),
            self.health_check_config.clone(),
        )
    }

    /// Access futures and options WebSocket streaming
    ///
    /// Returns:
    ///     FutOptWebSocketClient for FutOpt streaming with inherited config
    #[getter]
    pub fn futopt(&self) -> FutOptWebSocketClient {
        FutOptWebSocketClient::new(
            self.api_key.clone(),
            self.reconnect_config.clone(),
            self.health_check_config.clone(),
        )
    }
}

/// Internal WebSocket state (not Send/Sync safe, managed via Mutex)
///
/// The `inner` is wrapped in Arc to allow cloning the reference out of
/// the Mutex before async operations (avoiding holding MutexGuard across await).
struct WebSocketState {
    inner: Arc<marketdata_core::WebSocketClient>,
    receiver: Arc<marketdata_core::MessageReceiver>,
}

/// Stock market WebSocket client
///
/// Access via `ws.stock`
///
/// Supports both iterator-based and callback-based message consumption.
/// When message callbacks are registered before connect(), a background
/// thread automatically dispatches messages to the callbacks.
#[pyclass]
pub struct StockWebSocketClient {
    api_key: String,
    reconnect_config: ReconnectConfig,
    health_check_config: HealthCheckConfig,
    callbacks: Arc<CallbackRegistry>,
    // State is wrapped in Mutex<Option<>> for thread-safety
    state: Arc<Mutex<Option<WebSocketState>>>,
    runtime: Arc<Mutex<Option<tokio::runtime::Runtime>>>,
    // Background thread control
    message_thread_stop: Arc<AtomicBool>,
    message_thread_handle: Arc<Mutex<Option<std::thread::JoinHandle<()>>>>,
}

impl StockWebSocketClient {
    fn new(
        api_key: String,
        reconnect_config: ReconnectConfig,
        health_check_config: HealthCheckConfig,
    ) -> Self {
        Self {
            api_key,
            reconnect_config,
            health_check_config,
            callbacks: Arc::new(CallbackRegistry::new()),
            state: Arc::new(Mutex::new(None)),
            runtime: Arc::new(Mutex::new(None)),
            message_thread_stop: Arc::new(AtomicBool::new(false)),
            message_thread_handle: Arc::new(Mutex::new(None)),
        }
    }

    /// Get or create the tokio runtime
    fn ensure_runtime(&self) -> Result<(), String> {
        let mut runtime_guard = self.runtime.lock().map_err(|e| e.to_string())?;
        if runtime_guard.is_none() {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .map_err(|e| format!("Failed to create tokio runtime: {}", e))?;
            *runtime_guard = Some(rt);
        }
        Ok(())
    }

    /// Check if message callbacks are registered
    fn has_message_callbacks(&self) -> bool {
        self.callbacks.count(crate::callback::EventType::Message) > 0
    }

    /// Start background message dispatch thread
    fn start_message_thread(&self, receiver: Arc<marketdata_core::MessageReceiver>) {
        let callbacks = Arc::clone(&self.callbacks);
        let stop_flag = Arc::clone(&self.message_thread_stop);

        // Reset stop flag
        stop_flag.store(false, Ordering::SeqCst);

        #[allow(deprecated)]  // with_gil is deprecated in PyO3 0.27, use attach instead
        let handle = std::thread::spawn(move || {
            while !stop_flag.load(Ordering::SeqCst) {
                match receiver.receive_timeout(Duration::from_millis(100)) {
                    Ok(Some(msg)) => {
                        // Acquire GIL and invoke callback
                        Python::attach(|py| {
                            if let Ok(dict) = message_to_dict(py, &msg) {
                                let args = pyo3::types::PyTuple::new(py, [dict.into_any()]).expect("Failed to create tuple");
                                callbacks.invoke(
                                    py,
                                    crate::callback::EventType::Message,
                                    &args,
                                );
                            }
                        });
                    }
                    Ok(None) => {
                        // Timeout, continue loop
                    }
                    Err(_) => {
                        // Channel closed, exit loop
                        break;
                    }
                }
            }
        });

        // Store thread handle
        if let Ok(mut guard) = self.message_thread_handle.lock() {
            *guard = Some(handle);
        }
    }

    /// Stop background message dispatch thread
    fn stop_message_thread(&self) {
        // Signal thread to stop
        self.message_thread_stop.store(true, Ordering::SeqCst);

        // Wait for thread to finish
        if let Ok(mut guard) = self.message_thread_handle.lock() {
            if let Some(handle) = guard.take() {
                let _ = handle.join();
            }
        }
    }
}

#[pymethods]
impl StockWebSocketClient {
    /// Register a callback for an event type
    ///
    /// Supported events:
    ///   - "message" / "data": Called with message dict when data received
    ///   - "connect" / "connected": Called when connection established
    ///   - "disconnect" / "disconnected" / "close": Called when connection closed
    ///   - "reconnect" / "reconnecting": Called when reconnecting
    ///   - "error": Called with (message, code) when error occurs
    ///
    /// Args:
    ///     event: Event type string
    ///     callback: Python callable to invoke
    ///
    /// Example:
    ///     ```python
    ///     def on_message(msg):
    ///         print(f"Symbol: {msg.get('symbol')}, Price: {msg.get('price')}")
    ///
    ///     ws.stock.on("message", on_message)
    ///     ```
    #[pyo3(signature = (event, callback))]
    pub fn on(&self, event: &str, callback: &Bound<'_, PyAny>) -> PyResult<()> {
        self.callbacks.register(event, callback)
    }

    /// Remove all callbacks for an event type
    #[pyo3(signature = (event))]
    pub fn off(&self, event: &str) -> PyResult<()> {
        self.callbacks.unregister(event)
    }

    /// Connect to WebSocket server
    ///
    /// If message callbacks are registered before connect(), a background
    /// thread will automatically dispatch incoming messages to the callbacks.
    ///
    /// Raises:
    ///     MarketDataError: If connection fails
    pub fn connect(&self, py: Python<'_>) -> PyResult<()> {
        // Ensure runtime exists
        self.ensure_runtime().map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(e)
        })?;

        // Create WebSocket client with full config
        let auth = marketdata_core::AuthRequest::with_api_key(&self.api_key);
        let config = marketdata_core::ConnectionConfig::fugle_stock(auth);
        let ws_client = marketdata_core::WebSocketClient::with_full_config(
            config,
            self.reconnect_config.to_core(),
            self.health_check_config.to_core(),
        );

        // Get message receiver before connect
        let receiver = ws_client.messages();

        // Get event channel reference before ws_client is moved into Arc
        let events = Arc::clone(ws_client.state_events());

        // Clone callbacks for event dispatch
        let callbacks = Arc::clone(&self.callbacks);

        // Connect with GIL released
        // We need to do this in a scope where we have the runtime
        let runtime_guard = self.runtime.lock().map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(format!("Lock error: {}", e))
        })?;
        let runtime = runtime_guard.as_ref().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Runtime not initialized")
        })?;

        // Connect synchronously (blocking the current thread)
        let result = runtime.block_on(async {
            ws_client.connect().await
        });

        match result {
            Ok(()) => {
                // Clone receiver for potential background thread
                let receiver_for_thread = Arc::clone(&receiver);

                // Store state
                let state = WebSocketState {
                    inner: Arc::new(ws_client),
                    receiver,
                };

                let mut state_guard = self.state.lock().map_err(|e| {
                    pyo3::exceptions::PyRuntimeError::new_err(format!("Lock error: {}", e))
                })?;
                *state_guard = Some(state);

                // Start background message thread if message callbacks are registered
                if self.has_message_callbacks() {
                    self.start_message_thread(receiver_for_thread);
                }

                // Spawn event listener thread for connection events
                let callbacks_for_events = Arc::clone(&self.callbacks);
                std::thread::Builder::new()
                    .name("stock_ws_events".to_string())
                    .spawn(move || {
                        loop {
                            let event = {
                                let rx = events.blocking_lock();
                                rx.recv()
                            };
                            match event {
                                Ok(event) => {
                                    Python::attach(|py| {
                                        use marketdata_core::websocket::ConnectionEvent;
                                        match event {
                                            ConnectionEvent::Reconnecting { attempt } => {
                                                callbacks_for_events.invoke_reconnect(py, attempt);
                                            }
                                            ConnectionEvent::Error { message, code } => {
                                                callbacks_for_events.invoke_error(py, &message, code);
                                            }
                                            ConnectionEvent::Disconnected { code, reason } => {
                                                callbacks_for_events.invoke_disconnect(py, code, &reason);
                                            }
                                            ConnectionEvent::ReconnectFailed { attempts } => {
                                                callbacks_for_events.invoke_error(
                                                    py,
                                                    &format!("Reconnection failed after {} attempts", attempts),
                                                    -1,
                                                );
                                            }
                                            ConnectionEvent::PongMissed => {
                                                callbacks_for_events.invoke_error(
                                                    py,
                                                    "Health check: pong missed",
                                                    -2,
                                                );
                                            }
                                            ConnectionEvent::Authenticated => {
                                                callbacks_for_events.invoke_authenticated(py);
                                            }
                                            ConnectionEvent::Unauthenticated { message } => {
                                                callbacks_for_events.invoke_unauthenticated(py, &message);
                                            }
                                            _ => {} // Connecting, Connected handled elsewhere
                                        }
                                    });
                                }
                                Err(_) => break, // Channel closed
                            }
                        }
                    })
                    .ok();

                // Invoke connect callbacks
                callbacks.invoke_connect(py);
                Ok(())
            }
            Err(e) => Err(errors::to_py_err(e)),
        }
    }

    /// Disconnect from WebSocket server
    #[pyo3(signature = ())]
    pub fn disconnect(&self, py: Python<'_>) -> PyResult<()> {
        // Stop background message thread first
        self.stop_message_thread();

        let mut state_guard = self.state.lock().map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(format!("Lock error: {}", e))
        })?;

        if let Some(state) = state_guard.take() {
            let runtime_guard = self.runtime.lock().map_err(|e| {
                pyo3::exceptions::PyRuntimeError::new_err(format!("Lock error: {}", e))
            })?;

            if let Some(runtime) = runtime_guard.as_ref() {
                runtime.block_on(async {
                    let _ = state.inner.disconnect().await;
                });
            }

            // Invoke disconnect callbacks
            self.callbacks.invoke_disconnect(py, Some(1000), "Normal closure");
        }

        Ok(())
    }

    /// Check if currently connected
    #[pyo3(signature = ())]
    pub fn is_connected(&self) -> bool {
        let state_guard = match self.state.lock() {
            Ok(g) => g,
            Err(_) => return false,
        };

        if state_guard.is_none() {
            return false;
        }

        let runtime_guard = match self.runtime.lock() {
            Ok(g) => g,
            Err(_) => return false,
        };

        if let (Some(state), Some(runtime)) = (state_guard.as_ref(), runtime_guard.as_ref()) {
            runtime.block_on(async {
                state.inner.is_connected().await
            })
        } else {
            false
        }
    }

    /// Check if client has been closed
    ///
    /// Returns true if disconnect() has been called and client is closed.
    /// Once closed, the client cannot be reused - create a new instance.
    #[pyo3(signature = ())]
    pub fn is_closed(&self) -> bool {
        let state_guard = match self.state.lock() {
            Ok(g) => g,
            Err(_) => return false,
        };

        // If state is None (never connected), not closed
        let state = match state_guard.as_ref() {
            Some(s) => s,
            None => return false,
        };

        let runtime_guard = match self.runtime.lock() {
            Ok(g) => g,
            Err(_) => return false,
        };

        if let Some(runtime) = runtime_guard.as_ref() {
            runtime.block_on(async {
                state.inner.is_closed().await
            })
        } else {
            // If no runtime, use sync version
            state.inner.is_closed_sync()
        }
    }

    /// Subscribe to a channel for a symbol
    ///
    /// Args:
    ///     channel: Channel name (trades, candles, books, aggregates, indices)
    ///     symbol: Stock symbol (e.g., "2330")
    ///     odd_lot: Whether to subscribe to odd lot data (default: False)
    ///
    /// Example:
    ///     ```python
    ///     ws.stock.subscribe("trades", "2330")
    ///     ws.stock.subscribe("candles", "2330", odd_lot=True)
    ///     ```
    #[pyo3(signature = (channel, symbol=None, *, symbols=None, odd_lot=false))]
    pub fn subscribe(
        &self,
        channel: &str,
        symbol: Option<&str>,
        symbols: Option<Vec<String>>,
        odd_lot: bool,
    ) -> PyResult<()> {
        // Old SDK accepts either `symbol` (single) or `symbols` (batch). Exactly one
        // must be provided. We loop internally to keep core's wire protocol simple
        // (one subscribe message per symbol).
        let target_symbols: Vec<String> = match (symbol, symbols) {
            (Some(s), None) => vec![s.to_string()],
            (None, Some(list)) if !list.is_empty() => list,
            (None, Some(_)) => {
                return Err(pyo3::exceptions::PyValueError::new_err(
                    "subscribe(symbols=[]) is empty - provide at least one symbol",
                ));
            }
            (Some(_), Some(_)) => {
                return Err(pyo3::exceptions::PyValueError::new_err(
                    "subscribe() accepts either `symbol` or `symbols`, not both",
                ));
            }
            (None, None) => {
                return Err(pyo3::exceptions::PyValueError::new_err(
                    "subscribe() requires either `symbol` or `symbols`",
                ));
            }
        };

        let state_guard = self.state.lock().map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(format!("Lock error: {}", e))
        })?;

        let state = state_guard.as_ref().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Not connected. Call connect() first.")
        })?;

        // Parse channel
        let ch = match channel.to_lowercase().as_str() {
            "trades" => marketdata_core::Channel::Trades,
            "candles" => marketdata_core::Channel::Candles,
            "books" => marketdata_core::Channel::Books,
            "aggregates" => marketdata_core::Channel::Aggregates,
            "indices" => marketdata_core::Channel::Indices,
            _ => {
                return Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "Invalid channel: '{}'. Valid channels: trades, candles, books, aggregates, indices",
                    channel
                )));
            }
        };

        let runtime_guard = self.runtime.lock().map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(format!("Lock error: {}", e))
        })?;

        let runtime = runtime_guard.as_ref().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Runtime not initialized")
        })?;

        for sym in target_symbols {
            let sub = marketdata_core::StockSubscription::new(ch, &sym).with_odd_lot(odd_lot);
            let result = runtime.block_on(async { state.inner.subscribe_channel(sub).await });
            result.map_err(errors::to_py_err)?;
        }

        Ok(())
    }

    /// Unsubscribe from a channel
    ///
    /// Accepts either `subscription_id` (single) or `ids` (batch list) to match the
    /// old fugle-marketdata Node SDK shape. Exactly one must be provided.
    ///
    /// Args:
    ///     subscription_id: The subscription ID returned from subscribe (single)
    ///     ids: A list of subscription IDs to unsubscribe (batch)
    #[pyo3(signature = (subscription_id=None, *, ids=None))]
    pub fn unsubscribe(
        &self,
        subscription_id: Option<&str>,
        ids: Option<Vec<String>>,
    ) -> PyResult<()> {
        let target_ids: Vec<String> = match (subscription_id, ids) {
            (Some(id), None) => vec![id.to_string()],
            (None, Some(list)) if !list.is_empty() => list,
            (None, Some(_)) => {
                return Err(pyo3::exceptions::PyValueError::new_err(
                    "unsubscribe(ids=[]) is empty - provide at least one id",
                ));
            }
            (Some(_), Some(_)) => {
                return Err(pyo3::exceptions::PyValueError::new_err(
                    "unsubscribe() accepts either `subscription_id` or `ids`, not both",
                ));
            }
            (None, None) => {
                return Err(pyo3::exceptions::PyValueError::new_err(
                    "unsubscribe() requires either `subscription_id` or `ids`",
                ));
            }
        };

        let state_guard = self.state.lock().map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(format!("Lock error: {}", e))
        })?;

        let state = state_guard.as_ref().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Not connected. Call connect() first.")
        })?;

        let runtime_guard = self.runtime.lock().map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(format!("Lock error: {}", e))
        })?;

        let runtime = runtime_guard.as_ref().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Runtime not initialized")
        })?;

        for id in target_ids {
            let result = runtime.block_on(async { state.inner.unsubscribe_by_id(&id).await });
            result.map_err(errors::to_py_err)?;
        }

        Ok(())
    }

    /// Get message iterator for consuming streaming data
    ///
    /// Returns:
    ///     MessageIterator for iterating over messages
    ///
    /// Example:
    ///     ```python
    ///     for msg in ws.stock.messages():
    ///         print(msg)
    ///     ```
    ///
    /// Note: The iterator blocks waiting for messages. Use timeout parameter
    /// to control blocking behavior.
    #[pyo3(signature = (timeout_ms=None))]
    pub fn messages(&self, timeout_ms: Option<u64>) -> PyResult<crate::iterator::MessageIterator> {
        let state_guard = self.state.lock().map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(format!("Lock error: {}", e))
        })?;

        let state = state_guard.as_ref().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Not connected. Call connect() first.")
        })?;

        let receiver = Arc::clone(&state.receiver);
        let timeout = timeout_ms.map(Duration::from_millis);

        Ok(crate::iterator::MessageIterator::new(receiver, timeout))
    }

    /// Get the locally cached list of active subscription keys.
    ///
    /// Note: this is the *local* cache maintained by core's SubscriptionManager.
    /// To request the authoritative list from the server (matches the old
    /// fugle-marketdata SDK), call `subscriptions()` instead — the server's
    /// response will arrive via the registered `message` callback.
    #[pyo3(signature = ())]
    pub fn local_subscriptions(&self) -> Vec<String> {
        let state_guard = match self.state.lock() {
            Ok(g) => g,
            Err(_) => return vec![],
        };

        state_guard
            .as_ref()
            .map(|s| s.inner.subscription_keys())
            .unwrap_or_default()
    }

    /// Ask the server for its current subscription list.
    ///
    /// Sends `{"event": "subscriptions"}` to the server. The server replies
    /// asynchronously and the response is delivered via the `message` callback,
    /// matching the old `fugle-marketdata` SDK's `subscriptions()` semantics.
    ///
    /// Raises:
    ///     RuntimeError: If not connected
    #[pyo3(signature = ())]
    pub fn subscriptions(&self) -> PyResult<()> {
        let state_guard = self.state.lock().map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(format!("Lock error: {}", e))
        })?;
        let state = state_guard.as_ref().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Not connected. Call connect() first.")
        })?;
        let runtime_guard = self.runtime.lock().map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(format!("Lock error: {}", e))
        })?;
        let runtime = runtime_guard.as_ref().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Runtime not initialized")
        })?;

        let request = marketdata_core::WebSocketRequest::subscriptions();
        runtime
            .block_on(async { state.inner.send(request).await })
            .map_err(errors::to_py_err)
    }

    /// Send a `ping` frame to the server (matches the old fugle-marketdata SDK).
    ///
    /// Args:
    ///     state: Optional state string echoed back in the server's `pong` reply
    ///
    /// Raises:
    ///     RuntimeError: If not connected
    #[pyo3(signature = (state=None))]
    pub fn ping(&self, state: Option<String>) -> PyResult<()> {
        let state_guard = self.state.lock().map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(format!("Lock error: {}", e))
        })?;
        let st = state_guard.as_ref().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Not connected. Call connect() first.")
        })?;
        let runtime_guard = self.runtime.lock().map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(format!("Lock error: {}", e))
        })?;
        let runtime = runtime_guard.as_ref().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Runtime not initialized")
        })?;

        let request = marketdata_core::WebSocketRequest::ping(state);
        runtime
            .block_on(async { st.inner.send(request).await })
            .map_err(errors::to_py_err)
    }

    /// Connect to WebSocket server (async version)
    ///
    /// Returns an awaitable that completes when connection is established.
    /// Releases GIL during connection, enabling concurrent Python tasks.
    ///
    /// Raises:
    ///     MarketDataError: If connection fails
    ///
    /// Example:
    ///     ```python
    ///     await ws.stock.connect_async()
    ///     ```
    pub fn connect_async<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        // Ensure runtime exists
        self.ensure_runtime().map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(e)
        })?;

        let api_key = self.api_key.clone();
        let reconnect_config = self.reconnect_config.to_core();
        let health_check_config = self.health_check_config.to_core();
        let callbacks = Arc::clone(&self.callbacks);
        let state_arc = Arc::clone(&self.state);
        let has_message_callbacks = self.has_message_callbacks();
        let message_thread_stop = Arc::clone(&self.message_thread_stop);
        let message_thread_handle = Arc::clone(&self.message_thread_handle);

        future_into_py(py, async move {
            // Create WebSocket client with full config
            let auth = marketdata_core::AuthRequest::with_api_key(&api_key);
            let config = marketdata_core::ConnectionConfig::fugle_stock(auth);
            let ws_client = marketdata_core::WebSocketClient::with_full_config(
                config,
                reconnect_config,
                health_check_config,
            );

            // Get message receiver before connect
            let receiver = ws_client.messages();

            // Connect without holding GIL
            ws_client.connect().await
                .map_err(|e| crate::errors::to_py_err(e))?;

            // Clone receiver for potential background thread
            let receiver_for_thread = Arc::clone(&receiver);

            // Store state
            let state = WebSocketState {
                inner: Arc::new(ws_client),
                receiver,
            };

            let mut state_guard = state_arc.lock()
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("Lock error: {}", e)))?;
            *state_guard = Some(state);

            // Start background message thread if callbacks registered
            if has_message_callbacks {
                message_thread_stop.store(false, Ordering::SeqCst);
                let callbacks_clone = Arc::clone(&callbacks);
                let stop_flag = Arc::clone(&message_thread_stop);

                #[allow(deprecated)]
                let handle = std::thread::spawn(move || {
                    while !stop_flag.load(Ordering::SeqCst) {
                        match receiver_for_thread.receive_timeout(Duration::from_millis(100)) {
                            Ok(Some(msg)) => {
                                Python::attach(|py| {
                                    if let Ok(dict) = message_to_dict(py, &msg) {
                                        let args = pyo3::types::PyTuple::new(py, [dict.into_any()])
                                            .expect("Failed to create tuple");
                                        callbacks_clone.invoke(py, crate::callback::EventType::Message, &args);
                                    }
                                });
                            }
                            Ok(None) => {}
                            Err(_) => break,
                        }
                    }
                });

                if let Ok(mut guard) = message_thread_handle.lock() {
                    *guard = Some(handle);
                }
            }

            // Invoke connect callbacks with GIL
            Python::attach(|py| {
                callbacks.invoke_connect(py);
            });

            Ok(())
        })
    }

    /// Disconnect from WebSocket server (async version)
    ///
    /// Returns an awaitable that completes when disconnection finishes.
    ///
    /// Example:
    ///     ```python
    ///     await ws.stock.disconnect_async()
    ///     ```
    pub fn disconnect_async<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        // Stop background message thread first
        self.stop_message_thread();

        let state_arc = Arc::clone(&self.state);
        let callbacks = Arc::clone(&self.callbacks);

        future_into_py(py, async move {
            // Extract state from mutex without holding guard across await
            let state_opt = {
                let mut state_guard = state_arc.lock()
                    .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("Lock error: {}", e)))?;
                state_guard.take()
            };
            // Guard is dropped here

            if let Some(state) = state_opt {
                let _ = state.inner.disconnect().await;

                // Invoke disconnect callbacks with GIL
                Python::attach(|py| {
                    callbacks.invoke_disconnect(py, Some(1000), "Normal closure");
                });
            }

            Ok(())
        })
    }

    /// Subscribe to a channel (async version)
    ///
    /// Args:
    ///     channel: Channel name (trades, candles, books, aggregates, indices)
    ///     symbol: Stock symbol (e.g., "2330")
    ///     odd_lot: Whether to subscribe to odd lot data (default: False)
    ///
    /// Returns:
    ///     Awaitable that completes when subscription is confirmed
    ///
    /// Example:
    ///     ```python
    ///     await ws.stock.subscribe_async("trades", "2330")
    ///     ```
    #[pyo3(signature = (channel, symbol, odd_lot=false))]
    pub fn subscribe_async<'py>(
        &self,
        py: Python<'py>,
        channel: String,
        symbol: String,
        odd_lot: bool,
    ) -> PyResult<Bound<'py, PyAny>> {
        let state_arc = Arc::clone(&self.state);

        future_into_py(py, async move {
            // Parse channel first (doesn't need lock)
            let ch = match channel.to_lowercase().as_str() {
                "trades" => marketdata_core::Channel::Trades,
                "candles" => marketdata_core::Channel::Candles,
                "books" => marketdata_core::Channel::Books,
                "aggregates" => marketdata_core::Channel::Aggregates,
                "indices" => marketdata_core::Channel::Indices,
                _ => {
                    return Err(pyo3::exceptions::PyValueError::new_err(format!(
                        "Invalid channel: '{}'. Valid channels: trades, candles, books, aggregates, indices",
                        channel
                    )));
                }
            };

            // Create subscription
            let sub = marketdata_core::StockSubscription::new(ch, &symbol).with_odd_lot(odd_lot);

            // Clone the Arc<WebSocketClient> out of mutex to avoid holding guard across await
            let ws_client = {
                let state_guard = state_arc.lock()
                    .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("Lock error: {}", e)))?;
                let state = state_guard.as_ref()
                    .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("Not connected. Call connect() first."))?;
                Arc::clone(&state.inner)
            };
            // Guard is dropped here

            // Subscribe without holding mutex
            ws_client.subscribe_channel(sub).await
                .map_err(crate::errors::to_py_err)?;

            Ok(())
        })
    }

    /// Async context manager support: enter
    ///
    /// Example:
    ///     ```python
    ///     async with ws.stock as client:
    ///         await client.subscribe("trades", "2330")
    ///     ```
    fn __aenter__<'py>(slf: PyRef<'py, Self>, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        slf.connect_async(py)
    }

    /// Async context manager support: exit
    ///
    /// Automatically disconnects when exiting async with block.
    fn __aexit__<'py>(
        &self,
        py: Python<'py>,
        _exc_type: &Bound<'py, PyAny>,
        _exc_value: &Bound<'py, PyAny>,
        _traceback: &Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        self.disconnect_async(py)
    }
}

/// FutOpt (futures and options) WebSocket client
///
/// Access via `ws.futopt`
///
/// Note: `unsendable` is required because the underlying WebSocket state contains
/// `std::sync::mpsc::Receiver` which is not `Sync`. This means the client
/// should only be used from the thread that created it.
#[pyclass(unsendable)]
pub struct FutOptWebSocketClient {
    api_key: String,
    reconnect_config: ReconnectConfig,
    health_check_config: HealthCheckConfig,
    callbacks: Arc<CallbackRegistry>,
    state: Arc<Mutex<Option<WebSocketState>>>,
    runtime: Arc<Mutex<Option<tokio::runtime::Runtime>>>,
}

impl FutOptWebSocketClient {
    fn new(
        api_key: String,
        reconnect_config: ReconnectConfig,
        health_check_config: HealthCheckConfig,
    ) -> Self {
        Self {
            api_key,
            reconnect_config,
            health_check_config,
            callbacks: Arc::new(CallbackRegistry::new()),
            state: Arc::new(Mutex::new(None)),
            runtime: Arc::new(Mutex::new(None)),
        }
    }

    /// Get or create the tokio runtime
    fn ensure_runtime(&self) -> Result<(), String> {
        let mut runtime_guard = self.runtime.lock().map_err(|e| e.to_string())?;
        if runtime_guard.is_none() {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .map_err(|e| format!("Failed to create tokio runtime: {}", e))?;
            *runtime_guard = Some(rt);
        }
        Ok(())
    }
}

#[pymethods]
impl FutOptWebSocketClient {
    /// Register a callback for an event type
    ///
    /// Supported events:
    ///   - "message" / "data": Called with message dict when data received
    ///   - "connect" / "connected": Called when connection established
    ///   - "disconnect" / "disconnected" / "close": Called when connection closed
    ///   - "reconnect" / "reconnecting": Called when reconnecting
    ///   - "error": Called with (message, code) when error occurs
    ///
    /// Args:
    ///     event: Event type string
    ///     callback: Python callable to invoke
    #[pyo3(signature = (event, callback))]
    pub fn on(&self, event: &str, callback: &Bound<'_, PyAny>) -> PyResult<()> {
        self.callbacks.register(event, callback)
    }

    /// Remove all callbacks for an event type
    #[pyo3(signature = (event))]
    pub fn off(&self, event: &str) -> PyResult<()> {
        self.callbacks.unregister(event)
    }

    /// Connect to WebSocket server
    ///
    /// Raises:
    ///     MarketDataError: If connection fails
    #[pyo3(signature = ())]
    pub fn connect(&self, py: Python<'_>) -> PyResult<()> {
        // Ensure runtime exists
        self.ensure_runtime().map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(e)
        })?;

        // Create WebSocket client for FutOpt endpoint with full config
        let auth = marketdata_core::AuthRequest::with_api_key(&self.api_key);
        let config = marketdata_core::ConnectionConfig::fugle_futopt(auth);
        let ws_client = marketdata_core::WebSocketClient::with_full_config(
            config,
            self.reconnect_config.to_core(),
            self.health_check_config.to_core(),
        );

        // Get message receiver before connect
        let receiver = ws_client.messages();

        // Get event channel reference before ws_client is moved into Arc
        let events = Arc::clone(ws_client.state_events());

        // Clone callbacks for event dispatch
        let callbacks = Arc::clone(&self.callbacks);

        let runtime_guard = self.runtime.lock().map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(format!("Lock error: {}", e))
        })?;
        let runtime = runtime_guard.as_ref().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Runtime not initialized")
        })?;

        let result = runtime.block_on(async {
            ws_client.connect().await
        });

        match result {
            Ok(()) => {
                // Store state
                let state = WebSocketState {
                    inner: Arc::new(ws_client),
                    receiver,
                };

                let mut state_guard = self.state.lock().map_err(|e| {
                    pyo3::exceptions::PyRuntimeError::new_err(format!("Lock error: {}", e))
                })?;
                *state_guard = Some(state);

                // Spawn event listener thread for connection events
                let callbacks_for_events = Arc::clone(&self.callbacks);
                std::thread::Builder::new()
                    .name("futopt_ws_events".to_string())
                    .spawn(move || {
                        loop {
                            let event = {
                                let rx = events.blocking_lock();
                                rx.recv()
                            };
                            match event {
                                Ok(event) => {
                                    Python::attach(|py| {
                                        use marketdata_core::websocket::ConnectionEvent;
                                        match event {
                                            ConnectionEvent::Reconnecting { attempt } => {
                                                callbacks_for_events.invoke_reconnect(py, attempt);
                                            }
                                            ConnectionEvent::Error { message, code } => {
                                                callbacks_for_events.invoke_error(py, &message, code);
                                            }
                                            ConnectionEvent::Disconnected { code, reason } => {
                                                callbacks_for_events.invoke_disconnect(py, code, &reason);
                                            }
                                            ConnectionEvent::ReconnectFailed { attempts } => {
                                                callbacks_for_events.invoke_error(
                                                    py,
                                                    &format!("Reconnection failed after {} attempts", attempts),
                                                    -1,
                                                );
                                            }
                                            ConnectionEvent::PongMissed => {
                                                callbacks_for_events.invoke_error(
                                                    py,
                                                    "Health check: pong missed",
                                                    -2,
                                                );
                                            }
                                            ConnectionEvent::Authenticated => {
                                                callbacks_for_events.invoke_authenticated(py);
                                            }
                                            ConnectionEvent::Unauthenticated { message } => {
                                                callbacks_for_events.invoke_unauthenticated(py, &message);
                                            }
                                            _ => {} // Connecting, Connected handled elsewhere
                                        }
                                    });
                                }
                                Err(_) => break, // Channel closed
                            }
                        }
                    })
                    .ok();

                // Invoke connect callbacks
                callbacks.invoke_connect(py);
                Ok(())
            }
            Err(e) => Err(errors::to_py_err(e)),
        }
    }

    /// Disconnect from WebSocket server
    #[pyo3(signature = ())]
    pub fn disconnect(&self, py: Python<'_>) -> PyResult<()> {
        let mut state_guard = self.state.lock().map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(format!("Lock error: {}", e))
        })?;

        if let Some(state) = state_guard.take() {
            let runtime_guard = self.runtime.lock().map_err(|e| {
                pyo3::exceptions::PyRuntimeError::new_err(format!("Lock error: {}", e))
            })?;

            if let Some(runtime) = runtime_guard.as_ref() {
                runtime.block_on(async {
                    let _ = state.inner.disconnect().await;
                });
            }

            // Invoke disconnect callbacks
            self.callbacks.invoke_disconnect(py, Some(1000), "Normal closure");
        }

        Ok(())
    }

    /// Check if currently connected
    #[pyo3(signature = ())]
    pub fn is_connected(&self) -> bool {
        let state_guard = match self.state.lock() {
            Ok(g) => g,
            Err(_) => return false,
        };

        if state_guard.is_none() {
            return false;
        }

        let runtime_guard = match self.runtime.lock() {
            Ok(g) => g,
            Err(_) => return false,
        };

        if let (Some(state), Some(runtime)) = (state_guard.as_ref(), runtime_guard.as_ref()) {
            runtime.block_on(async {
                state.inner.is_connected().await
            })
        } else {
            false
        }
    }

    /// Check if client has been closed
    ///
    /// Returns true if disconnect() has been called and client is closed.
    /// Once closed, the client cannot be reused - create a new instance.
    #[pyo3(signature = ())]
    pub fn is_closed(&self) -> bool {
        let state_guard = match self.state.lock() {
            Ok(g) => g,
            Err(_) => return false,
        };

        // If state is None (never connected), not closed
        let state = match state_guard.as_ref() {
            Some(s) => s,
            None => return false,
        };

        let runtime_guard = match self.runtime.lock() {
            Ok(g) => g,
            Err(_) => return false,
        };

        if let Some(runtime) = runtime_guard.as_ref() {
            runtime.block_on(async {
                state.inner.is_closed().await
            })
        } else {
            // If no runtime, use sync version
            state.inner.is_closed_sync()
        }
    }

    /// Subscribe to a channel for a FutOpt symbol
    ///
    /// Args:
    ///     channel: Channel name (trades, candles, books, aggregates)
    ///     symbol: FutOpt contract symbol (e.g., "TXFC4", "TXF202502")
    ///     after_hours: Whether to subscribe to after-hours session (default: False)
    ///
    /// Example:
    ///     ```python
    ///     ws.futopt.subscribe("trades", "TXFC4")
    ///     ws.futopt.subscribe("books", "MXFB4", after_hours=True)
    ///     ```
    #[pyo3(signature = (channel, symbol=None, *, symbols=None, after_hours=false))]
    pub fn subscribe(
        &self,
        channel: &str,
        symbol: Option<&str>,
        symbols: Option<Vec<String>>,
        after_hours: bool,
    ) -> PyResult<()> {
        // Mirror the stock client: accept either single `symbol` or batch `symbols`,
        // loop internally so the wire protocol stays one-message-per-symbol.
        let target_symbols: Vec<String> = match (symbol, symbols) {
            (Some(s), None) => vec![s.to_string()],
            (None, Some(list)) if !list.is_empty() => list,
            (None, Some(_)) => {
                return Err(pyo3::exceptions::PyValueError::new_err(
                    "subscribe(symbols=[]) is empty - provide at least one symbol",
                ));
            }
            (Some(_), Some(_)) => {
                return Err(pyo3::exceptions::PyValueError::new_err(
                    "subscribe() accepts either `symbol` or `symbols`, not both",
                ));
            }
            (None, None) => {
                return Err(pyo3::exceptions::PyValueError::new_err(
                    "subscribe() requires either `symbol` or `symbols`",
                ));
            }
        };

        let state_guard = self.state.lock().map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(format!("Lock error: {}", e))
        })?;

        let state = state_guard.as_ref().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Not connected. Call connect() first.")
        })?;

        // Parse channel (FutOpt doesn't have indices channel)
        let ch = match channel.to_lowercase().as_str() {
            "trades" => marketdata_core::FutOptChannel::Trades,
            "candles" => marketdata_core::FutOptChannel::Candles,
            "books" => marketdata_core::FutOptChannel::Books,
            "aggregates" => marketdata_core::FutOptChannel::Aggregates,
            _ => {
                return Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "Invalid channel: '{}'. Valid channels: trades, candles, books, aggregates",
                    channel
                )));
            }
        };

        let runtime_guard = self.runtime.lock().map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(format!("Lock error: {}", e))
        })?;

        let runtime = runtime_guard.as_ref().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Runtime not initialized")
        })?;

        for sym in target_symbols {
            // Construct via FutOptSubscription so the after_hours flag is honored
            let _sub = marketdata_core::FutOptSubscription::new(ch, &sym)
                .with_after_hours(after_hours);

            // TODO: Add afterHours support to SubscribeRequest in marketdata-core.
            // For now, send a plain SubscribeRequest (after_hours flag is constructed
            // above for parity with the stock side and future wiring).
            let request = marketdata_core::WebSocketRequest::subscribe(
                marketdata_core::SubscribeRequest {
                    channel: ch.as_str().to_string(),
                    symbol: Some(sym.clone()),
                },
            );

            let result = runtime.block_on(async { state.inner.send(request).await });
            result.map_err(errors::to_py_err)?;
        }

        Ok(())
    }

    /// Unsubscribe from a channel
    ///
    /// Accepts either `subscription_id` (single) or `ids` (batch). Mirrors the
    /// old fugle-marketdata Node SDK shape.
    #[pyo3(signature = (subscription_id=None, *, ids=None))]
    pub fn unsubscribe(
        &self,
        subscription_id: Option<&str>,
        ids: Option<Vec<String>>,
    ) -> PyResult<()> {
        let target_ids: Vec<String> = match (subscription_id, ids) {
            (Some(id), None) => vec![id.to_string()],
            (None, Some(list)) if !list.is_empty() => list,
            (None, Some(_)) => {
                return Err(pyo3::exceptions::PyValueError::new_err(
                    "unsubscribe(ids=[]) is empty - provide at least one id",
                ));
            }
            (Some(_), Some(_)) => {
                return Err(pyo3::exceptions::PyValueError::new_err(
                    "unsubscribe() accepts either `subscription_id` or `ids`, not both",
                ));
            }
            (None, None) => {
                return Err(pyo3::exceptions::PyValueError::new_err(
                    "unsubscribe() requires either `subscription_id` or `ids`",
                ));
            }
        };

        let state_guard = self.state.lock().map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(format!("Lock error: {}", e))
        })?;

        let state = state_guard.as_ref().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Not connected. Call connect() first.")
        })?;

        let runtime_guard = self.runtime.lock().map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(format!("Lock error: {}", e))
        })?;

        let runtime = runtime_guard.as_ref().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Runtime not initialized")
        })?;

        for id in target_ids {
            let result = runtime.block_on(async { state.inner.unsubscribe_by_id(&id).await });
            result.map_err(errors::to_py_err)?;
        }

        Ok(())
    }

    /// Get message iterator for consuming streaming data
    ///
    /// Returns:
    ///     MessageIterator for iterating over messages
    ///
    /// Example:
    ///     ```python
    ///     for msg in ws.futopt.messages():
    ///         print(msg)
    ///     ```
    #[pyo3(signature = (timeout_ms=None))]
    pub fn messages(&self, timeout_ms: Option<u64>) -> PyResult<crate::iterator::MessageIterator> {
        let state_guard = self.state.lock().map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(format!("Lock error: {}", e))
        })?;

        let state = state_guard.as_ref().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Not connected. Call connect() first.")
        })?;

        let receiver = Arc::clone(&state.receiver);
        let timeout = timeout_ms.map(Duration::from_millis);

        Ok(crate::iterator::MessageIterator::new(receiver, timeout))
    }

    /// Get the locally cached list of active subscription keys.
    ///
    /// Note: this is the *local* cache maintained by core's SubscriptionManager.
    /// To request the authoritative list from the server (matches the old
    /// fugle-marketdata SDK), call `subscriptions()` instead — the server's
    /// response will arrive via the registered `message` callback.
    #[pyo3(signature = ())]
    pub fn local_subscriptions(&self) -> Vec<String> {
        let state_guard = match self.state.lock() {
            Ok(g) => g,
            Err(_) => return vec![],
        };

        state_guard
            .as_ref()
            .map(|s| s.inner.subscription_keys())
            .unwrap_or_default()
    }

    /// Ask the server for its current subscription list.
    ///
    /// Sends `{"event": "subscriptions"}` to the server. The server replies
    /// asynchronously and the response is delivered via the `message` callback,
    /// matching the old `fugle-marketdata` SDK's `subscriptions()` semantics.
    ///
    /// Raises:
    ///     RuntimeError: If not connected
    #[pyo3(signature = ())]
    pub fn subscriptions(&self) -> PyResult<()> {
        let state_guard = self.state.lock().map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(format!("Lock error: {}", e))
        })?;
        let state = state_guard.as_ref().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Not connected. Call connect() first.")
        })?;
        let runtime_guard = self.runtime.lock().map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(format!("Lock error: {}", e))
        })?;
        let runtime = runtime_guard.as_ref().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Runtime not initialized")
        })?;

        let request = marketdata_core::WebSocketRequest::subscriptions();
        runtime
            .block_on(async { state.inner.send(request).await })
            .map_err(errors::to_py_err)
    }

    /// Send a `ping` frame to the server (matches the old fugle-marketdata SDK).
    ///
    /// Args:
    ///     state: Optional state string echoed back in the server's `pong` reply
    ///
    /// Raises:
    ///     RuntimeError: If not connected
    #[pyo3(signature = (state=None))]
    pub fn ping(&self, state: Option<String>) -> PyResult<()> {
        let state_guard = self.state.lock().map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(format!("Lock error: {}", e))
        })?;
        let st = state_guard.as_ref().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Not connected. Call connect() first.")
        })?;
        let runtime_guard = self.runtime.lock().map_err(|e| {
            pyo3::exceptions::PyRuntimeError::new_err(format!("Lock error: {}", e))
        })?;
        let runtime = runtime_guard.as_ref().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err("Runtime not initialized")
        })?;

        let request = marketdata_core::WebSocketRequest::ping(state);
        runtime
            .block_on(async { st.inner.send(request).await })
            .map_err(errors::to_py_err)
    }
}

/// Convert WebSocketMessage to Python dict
pub fn message_to_dict(py: Python<'_>, msg: &marketdata_core::WebSocketMessage) -> PyResult<Py<PyDict>> {
    let dict = PyDict::new(py);

    dict.set_item("event", &msg.event)?;

    if let Some(ref channel) = msg.channel {
        dict.set_item("channel", channel)?;
    }

    if let Some(ref symbol) = msg.symbol {
        dict.set_item("symbol", symbol)?;
    }

    if let Some(ref id) = msg.id {
        dict.set_item("id", id)?;
    }

    // Convert data to Python dict if present
    if let Some(ref data) = msg.data {
        // data is serde_json::Value, convert to Python
        let py_data = json_value_to_py(py, data)?;
        dict.set_item("data", py_data)?;
    }

    Ok(dict.unbind())
}

/// Convert serde_json::Value to Py<PyAny>
fn json_value_to_py(py: Python<'_>, value: &serde_json::Value) -> PyResult<Py<PyAny>> {
    use pyo3::IntoPyObject;

    match value {
        serde_json::Value::Null => Ok(py.None()),
        serde_json::Value::Bool(b) => {
            Ok(b.into_pyobject(py)?.to_owned().unbind().into_any())
        }
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(i.into_pyobject(py)?.to_owned().unbind().into_any())
            } else if let Some(u) = n.as_u64() {
                Ok(u.into_pyobject(py)?.to_owned().unbind().into_any())
            } else if let Some(f) = n.as_f64() {
                Ok(f.into_pyobject(py)?.to_owned().unbind().into_any())
            } else {
                Ok(py.None())
            }
        }
        serde_json::Value::String(s) => {
            Ok(s.into_pyobject(py)?.to_owned().unbind().into_any())
        }
        serde_json::Value::Array(arr) => {
            let list = pyo3::types::PyList::empty(py);
            for item in arr {
                list.append(json_value_to_py(py, item)?)?;
            }
            Ok(list.unbind().into_any())
        }
        serde_json::Value::Object(obj) => {
            let dict = PyDict::new(py);
            for (k, v) in obj {
                dict.set_item(k, json_value_to_py(py, v)?)?;
            }
            Ok(dict.unbind().into_any())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_websocket_client_creation_with_api_key() {
        // WebSocketClient::new requires Python bindings, test the internal child client instead
        let client = StockWebSocketClient::new(
            "test-key".to_string(),
            ReconnectConfig::default(),
            HealthCheckConfig::default(),
        );
        let state = client.state.lock().unwrap();
        assert!(state.is_none());
    }

    #[test]
    fn test_stock_websocket_client_creation() {
        let client = StockWebSocketClient::new(
            "test-key".to_string(),
            ReconnectConfig::default(),
            HealthCheckConfig::default(),
        );
        let state = client.state.lock().unwrap();
        assert!(state.is_none());
    }

    #[test]
    fn test_futopt_websocket_client_creation() {
        let client = FutOptWebSocketClient::new(
            "test-key".to_string(),
            ReconnectConfig::default(),
            HealthCheckConfig::default(),
        );
        let state = client.state.lock().unwrap();
        assert!(state.is_none());
    }
}
