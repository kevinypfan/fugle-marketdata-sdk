//! WebSocket client FFI exports for C# binding

use std::ffi::{c_char, c_int};
use std::panic::AssertUnwindSafe;
use std::ptr;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{self, Receiver, TryRecvError};

use marketdata_core::{
    websocket::{WebSocketClient as CoreWebSocketClient, ConnectionConfig},
    models::AuthRequest,
};

use crate::errors::{catch_panic, error_to_code, SUCCESS, ERROR_INVALID_ARG, ERROR_INTERNAL, ERROR_CONNECTION_FAILED};
use crate::types::{cstr_to_string, string_to_cstring, RUNTIME};

/// Message status codes for polling
pub const MESSAGE_AVAILABLE: c_int = 1;
pub const NO_MESSAGE: c_int = 0;

/// Connection state codes
pub const STATE_DISCONNECTED: c_int = 0;
pub const STATE_CONNECTING: c_int = 1;
pub const STATE_CONNECTED: c_int = 2;
pub const STATE_RECONNECTING: c_int = 3;

/// Opaque handle for WebSocket client
pub struct WebSocketHandle {
    client: Arc<Mutex<Option<CoreWebSocketClient>>>,
    message_rx: Arc<Mutex<Option<Receiver<String>>>>,
    state: Arc<Mutex<c_int>>,
}

/// Create a new WebSocket client with API key
#[no_mangle]
pub extern "C" fn fugle_ws_client_new(_api_key: *const c_char) -> *mut WebSocketHandle {
    catch_panic(AssertUnwindSafe(|| {
        // API key validation handled in connect(), just create empty handle
        let handle = Box::new(WebSocketHandle {
            client: Arc::new(Mutex::new(None)),
            message_rx: Arc::new(Mutex::new(None)),
            state: Arc::new(Mutex::new(STATE_DISCONNECTED)),
        });

        Ok(Box::into_raw(handle))
    }))
    .unwrap_or(ptr::null_mut())
}

/// Connect WebSocket client (blocking during handshake)
/// endpoint_type: 0 = stock, 1 = futopt
/// Returns: SUCCESS on success, error code on failure
#[no_mangle]
pub extern "C" fn fugle_ws_connect(
    handle: *mut WebSocketHandle,
    api_key: *const c_char,
    endpoint_type: c_int,
) -> c_int {
    catch_panic(AssertUnwindSafe(|| {
        if handle.is_null() {
            return Err(ERROR_INVALID_ARG);
        }

        let api_key = unsafe { cstr_to_string(api_key) }
            .ok_or(ERROR_INVALID_ARG)?;

        let handle = unsafe { &mut *handle };

        // Update state to connecting
        *handle.state.lock().unwrap() = STATE_CONNECTING;

        // Create message channel
        let (tx, rx) = mpsc::channel::<String>();
        *handle.message_rx.lock().unwrap() = Some(rx);

        // Create and connect client
        let auth = AuthRequest::with_api_key(api_key);
        let config = if endpoint_type == 0 {
            ConnectionConfig::fugle_stock(auth)
        } else {
            ConnectionConfig::fugle_futopt(auth)
        };

        let state_clone = Arc::clone(&handle.state);
        let tx_clone = tx.clone();

        let connect_result = RUNTIME.block_on(async {
            let client = CoreWebSocketClient::new(config);

            // Get message receiver from core client
            let message_receiver = client.messages();

            // Spawn a task to forward messages to the channel
            let tx_forward = tx_clone.clone();
            tokio::spawn(async move {
                loop {
                    // Poll for messages from core client (non-blocking)
                    if let Some(msg_result) = (*message_receiver).try_receive() {
                        // Forward message as JSON string
                        if let Ok(json) = serde_json::to_string(&msg_result) {
                            let _ = tx_forward.send(json);
                        }
                    } else {
                        // No message available, sleep briefly
                        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                    }
                }
            });

            // Connect
            match client.connect().await {
                Ok(_) => {
                    *state_clone.lock().unwrap() = STATE_CONNECTED;
                    Ok(client)
                }
                Err(e) => {
                    *state_clone.lock().unwrap() = STATE_DISCONNECTED;
                    Err(error_to_code(&e))
                }
            }
        });

        match connect_result {
            Ok(client) => {
                *handle.client.lock().unwrap() = Some(client);
                Ok(SUCCESS)
            }
            Err(code) => Err(code),
        }
    }))
    .unwrap_or(ERROR_INTERNAL)
}

/// Disconnect WebSocket client
#[no_mangle]
pub extern "C" fn fugle_ws_disconnect(handle: *mut WebSocketHandle) -> c_int {
    catch_panic(AssertUnwindSafe(|| {
        if handle.is_null() {
            return Err(ERROR_INVALID_ARG);
        }

        let handle = unsafe { &mut *handle };

        if let Some(client) = handle.client.lock().unwrap().take() {
            let _ = RUNTIME.block_on(async {
                client.disconnect().await
            });
        }

        *handle.state.lock().unwrap() = STATE_DISCONNECTED;
        *handle.message_rx.lock().unwrap() = None;

        Ok(SUCCESS)
    }))
    .unwrap_or(ERROR_INTERNAL)
}

/// Get current connection state
#[no_mangle]
pub extern "C" fn fugle_ws_get_state(handle: *const WebSocketHandle) -> c_int {
    catch_panic(AssertUnwindSafe(|| {
        if handle.is_null() {
            return Ok(STATE_DISCONNECTED);
        }

        let handle = unsafe { &*handle };
        Ok(*handle.state.lock().unwrap())
    }))
    .unwrap_or(STATE_DISCONNECTED)
}

/// Poll for next message (non-blocking)
/// Returns: MESSAGE_AVAILABLE if message retrieved, NO_MESSAGE if none, error code on failure
/// On MESSAGE_AVAILABLE, message_out is set to JSON string (caller must free with fugle_free_string)
#[no_mangle]
pub extern "C" fn fugle_ws_poll_message(
    handle: *const WebSocketHandle,
    message_out: *mut *mut c_char,
) -> c_int {
    catch_panic(AssertUnwindSafe(|| {
        if handle.is_null() || message_out.is_null() {
            return Err(ERROR_INVALID_ARG);
        }

        let handle = unsafe { &*handle };

        let rx_guard = handle.message_rx.lock().unwrap();
        if let Some(rx) = rx_guard.as_ref() {
            match rx.try_recv() {
                Ok(msg) => {
                    let c_msg = string_to_cstring(&msg);
                    unsafe { *message_out = c_msg; }
                    Ok(MESSAGE_AVAILABLE)
                }
                Err(TryRecvError::Empty) => {
                    unsafe { *message_out = ptr::null_mut(); }
                    Ok(NO_MESSAGE)
                }
                Err(TryRecvError::Disconnected) => {
                    unsafe { *message_out = ptr::null_mut(); }
                    Err(ERROR_CONNECTION_FAILED)
                }
            }
        } else {
            unsafe { *message_out = ptr::null_mut(); }
            Ok(NO_MESSAGE)
        }
    }))
    .unwrap_or(ERROR_INTERNAL)
}

/// Subscribe to a channel
/// channel: "trades", "books", "candles", etc.
/// symbol: stock/futopt symbol like "2330" or "TXFF4"
#[no_mangle]
pub extern "C" fn fugle_ws_subscribe(
    handle: *mut WebSocketHandle,
    channel: *const c_char,
    symbol: *const c_char,
) -> c_int {
    catch_panic(AssertUnwindSafe(|| {
        if handle.is_null() {
            return Err(ERROR_INVALID_ARG);
        }

        let channel = unsafe { cstr_to_string(channel) }
            .ok_or(ERROR_INVALID_ARG)?;
        let symbol = unsafe { cstr_to_string(symbol) }
            .ok_or(ERROR_INVALID_ARG)?;

        let handle = unsafe { &mut *handle };

        let mut client_guard = handle.client.lock().unwrap();
        if let Some(client) = client_guard.as_mut() {
            let result = RUNTIME.block_on(async {
                // Use generic subscribe method - works for both stock and futopt
                let channel_enum = match channel.as_str() {
                    "trades" => marketdata_core::models::Channel::Trades,
                    "candles" => marketdata_core::models::Channel::Candles,
                    "books" => marketdata_core::models::Channel::Books,
                    "aggregates" => marketdata_core::models::Channel::Aggregates,
                    "indices" => marketdata_core::models::Channel::Indices,
                    _ => return Err(marketdata_core::MarketDataError::InvalidSymbol {
                        symbol: format!("Invalid channel: {}", channel)
                    }),
                };

                let req = marketdata_core::models::SubscribeRequest::new(channel_enum, symbol);
                client.subscribe(req).await
            });

            match result {
                Ok(_) => Ok(SUCCESS),
                Err(e) => Err(error_to_code(&e)),
            }
        } else {
            Err(ERROR_CONNECTION_FAILED)
        }
    }))
    .unwrap_or(ERROR_INTERNAL)
}

/// Unsubscribe from a channel
/// key: subscription key in format "channel:symbol" (e.g., "trades:2330")
#[no_mangle]
pub extern "C" fn fugle_ws_unsubscribe(
    handle: *mut WebSocketHandle,
    key: *const c_char,
) -> c_int {
    catch_panic(AssertUnwindSafe(|| {
        if handle.is_null() {
            return Err(ERROR_INVALID_ARG);
        }

        let key = unsafe { cstr_to_string(key) }
            .ok_or(ERROR_INVALID_ARG)?;

        let handle = unsafe { &mut *handle };

        let mut client_guard = handle.client.lock().unwrap();
        if let Some(client) = client_guard.as_mut() {
            let result = RUNTIME.block_on(async {
                client.unsubscribe(&key).await
            });

            match result {
                Ok(_) => Ok(SUCCESS),
                Err(e) => Err(error_to_code(&e)),
            }
        } else {
            Err(ERROR_CONNECTION_FAILED)
        }
    }))
    .unwrap_or(ERROR_INTERNAL)
}

/// Destroy WebSocket client handle
#[no_mangle]
pub extern "C" fn fugle_ws_client_free(handle: *mut WebSocketHandle) {
    if !handle.is_null() {
        let _ = catch_panic(AssertUnwindSafe(|| {
            // Disconnect first if connected
            fugle_ws_disconnect(handle);

            unsafe {
                let _ = Box::from_raw(handle);
            }
            Ok(())
        }));
    }
}
