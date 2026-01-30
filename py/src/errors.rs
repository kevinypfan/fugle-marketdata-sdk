//! Error mapping from marketdata-core to Python exceptions
//!
//! Maps MarketDataError variants to Python exceptions with error_code attribute.

use pyo3::prelude::*;
use pyo3::exceptions::PyException;
use pyo3::create_exception;

// Create a custom Python exception hierarchy for market data errors

// Base exception for all market data errors
create_exception!(marketdata_py, MarketDataError, PyException);

// API-related errors
create_exception!(marketdata_py, ApiError, MarketDataError, "API request failed");
create_exception!(marketdata_py, RateLimitError, ApiError, "Rate limit exceeded");

// Authentication errors
create_exception!(marketdata_py, AuthError, MarketDataError, "Authentication failed");

// Connection errors
create_exception!(marketdata_py, ConnectionError, MarketDataError, "Connection failed");
create_exception!(marketdata_py, TimeoutError, MarketDataError, "Operation timed out");

// WebSocket errors
create_exception!(marketdata_py, WebSocketError, MarketDataError, "WebSocket operation failed");

/// Convert marketdata_core error to PyErr with specific exception types
///
/// Maps MarketDataError variants to specific Python exception types:
/// - AuthError → AuthError
/// - ApiError → ApiError (or RateLimitError for 429 status)
/// - TimeoutError → TimeoutError
/// - Connection/WebSocket errors → WebSocketError
/// - Other errors → MarketDataError (base exception)
///
/// The error_code is available as the second element of args:
/// ```python
/// try:
///     quote = client.stock.intraday.quote("INVALID")
/// except ApiError as e:
///     message = e.args[0]
///     error_code = e.args[1]  # error code is in args[1]
///     print(f"Error {error_code}: {message}")
/// ```
pub fn to_py_err(err: marketdata_core::MarketDataError) -> PyErr {
    use marketdata_core::MarketDataError as CoreError;

    let error_code = err.to_error_code();
    let message = err.to_string();

    // Map to specific exception types based on error variant
    match err {
        CoreError::AuthError { .. } => {
            AuthError::new_err((message, error_code))
        }
        CoreError::ApiError { status, .. } => {
            // Rate limit error for HTTP 429
            if status == 429 {
                RateLimitError::new_err((message, error_code))
            } else {
                ApiError::new_err((message, error_code))
            }
        }
        CoreError::TimeoutError { .. } => {
            TimeoutError::new_err((message, error_code))
        }
        CoreError::ConnectionError { .. }
        | CoreError::WebSocketError { .. }
        | CoreError::ClientClosed => {
            WebSocketError::new_err((message, error_code))
        }
        _ => {
            // All other errors use base MarketDataError
            MarketDataError::new_err((message, error_code))
        }
    }
}

/// Helper to get error_code from a MarketDataError
#[allow(dead_code)]
pub fn get_error_code(err: &marketdata_core::MarketDataError) -> i32 {
    err.to_error_code()
}

#[cfg(test)]
mod tests {
    use super::*;
    use marketdata_core::MarketDataError as CoreError;

    #[test]
    fn test_error_code_mapping() {
        let err = CoreError::InvalidSymbol {
            symbol: "TEST".to_string(),
        };
        assert_eq!(get_error_code(&err), 1001);

        let err = CoreError::AuthError {
            msg: "test".to_string(),
        };
        assert_eq!(get_error_code(&err), 2002);

        let err = CoreError::ApiError {
            status: 404,
            message: "not found".to_string(),
        };
        assert_eq!(get_error_code(&err), 2003);

        let err = CoreError::TimeoutError {
            operation: "test".to_string(),
        };
        assert_eq!(get_error_code(&err), 3001);
    }

    #[test]
    fn test_error_message() {
        let err = CoreError::InvalidSymbol {
            symbol: "BAD".to_string(),
        };
        assert_eq!(err.to_string(), "Invalid symbol: BAD");
    }

    #[test]
    fn test_client_closed_error_code() {
        let err = CoreError::ClientClosed;
        assert_eq!(get_error_code(&err), 2010);
        assert_eq!(err.to_string(), "Client already closed");
    }
}
