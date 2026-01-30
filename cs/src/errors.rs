//! Error handling for C# FFI boundary

use std::panic;
use libc::c_int;

// Error codes matching core::errors::MarketDataError variants
pub const SUCCESS: c_int = 0;
pub const ERROR_INVALID_ARG: c_int = -1;
pub const ERROR_AUTH_FAILED: c_int = -2;
pub const ERROR_RATE_LIMITED: c_int = -3;
pub const ERROR_API_ERROR: c_int = -4;
pub const ERROR_CONNECTION_FAILED: c_int = -5;
pub const ERROR_TIMEOUT: c_int = -6;
pub const ERROR_WEBSOCKET: c_int = -7;
pub const ERROR_INTERNAL: c_int = -999;

/// Convert core error to error code
pub fn error_to_code(e: &marketdata_core::MarketDataError) -> c_int {
    use marketdata_core::MarketDataError;
    match e {
        MarketDataError::AuthError { .. } => ERROR_AUTH_FAILED,
        MarketDataError::ApiError { status, .. } if *status == 429 => ERROR_RATE_LIMITED,
        MarketDataError::ApiError { .. } => ERROR_API_ERROR,
        MarketDataError::ConnectionError { .. } => ERROR_CONNECTION_FAILED,
        MarketDataError::TimeoutError { .. } => ERROR_TIMEOUT,
        MarketDataError::WebSocketError { .. } => ERROR_WEBSOCKET,
        MarketDataError::InvalidSymbol { .. } => ERROR_INVALID_ARG,
        MarketDataError::DeserializationError { .. } => ERROR_INVALID_ARG,
        MarketDataError::RuntimeError { .. } => ERROR_INTERNAL,
        MarketDataError::ConfigError(_) => ERROR_INVALID_ARG,
        MarketDataError::ClientClosed => ERROR_CONNECTION_FAILED,
        MarketDataError::Other(_) => ERROR_INTERNAL,
    }
}

/// Execute closure with panic recovery, return error code on panic
pub fn catch_panic<F, T>(f: F) -> Result<T, c_int>
where
    F: FnOnce() -> Result<T, c_int> + panic::UnwindSafe,
{
    match panic::catch_unwind(f) {
        Ok(result) => result,
        Err(_) => Err(ERROR_INTERNAL),
    }
}
