//! Python REST client wrapper
//!
//! Provides Python-friendly interface to marketdata-core RestClient.
//! Mirrors the official SDK's API: `client.stock.intraday.quote()` pattern.

use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::errors;
use crate::types;

/// Python REST client for Fugle market data API
///
/// # Example (Python)
///
/// ```python
/// from marketdata_py import RestClient
///
/// # Create client with API key
/// client = RestClient("your-api-key")
///
/// # Get stock quote
/// quote = client.stock.intraday.quote("2330")
/// print(quote["lastPrice"])
///
/// # Get futures quote
/// futopt_quote = client.futopt.intraday.quote("TXFC4")
/// ```
#[pyclass]
pub struct RestClient {
    inner: marketdata_core::RestClient,
}

#[pymethods]
impl RestClient {
    /// Create a new REST client with API key authentication
    ///
    /// Args:
    ///     api_key: Your Fugle API key
    ///
    /// Returns:
    ///     A new RestClient instance
    #[new]
    pub fn new(api_key: String) -> Self {
        let inner = marketdata_core::RestClient::new(marketdata_core::Auth::ApiKey(api_key));
        Self { inner }
    }

    /// Create a REST client with bearer token authentication
    ///
    /// Args:
    ///     token: Bearer token for authentication
    ///
    /// Returns:
    ///     A new RestClient instance
    #[staticmethod]
    pub fn with_bearer_token(token: String) -> Self {
        let inner = marketdata_core::RestClient::new(marketdata_core::Auth::BearerToken(token));
        Self { inner }
    }

    /// Create a REST client with SDK token authentication
    ///
    /// Args:
    ///     sdk_token: SDK token for authentication
    ///
    /// Returns:
    ///     A new RestClient instance
    #[staticmethod]
    pub fn with_sdk_token(sdk_token: String) -> Self {
        let inner = marketdata_core::RestClient::new(marketdata_core::Auth::SdkToken(sdk_token));
        Self { inner }
    }

    /// Access stock market data endpoints
    ///
    /// Returns:
    ///     StockClient for accessing stock endpoints
    #[getter]
    pub fn stock(&self) -> StockClient {
        StockClient {
            inner: self.inner.clone(),
        }
    }

    /// Access futures and options market data endpoints
    ///
    /// Returns:
    ///     FutOptClient for accessing FutOpt endpoints
    #[getter]
    pub fn futopt(&self) -> FutOptClient {
        FutOptClient {
            inner: self.inner.clone(),
        }
    }
}

/// Stock market data client
///
/// Access via `client.stock`
#[pyclass]
pub struct StockClient {
    inner: marketdata_core::RestClient,
}

#[pymethods]
impl StockClient {
    /// Access intraday (real-time) stock endpoints
    ///
    /// Returns:
    ///     StockIntradayClient for accessing intraday endpoints
    #[getter]
    pub fn intraday(&self) -> StockIntradayClient {
        StockIntradayClient {
            inner: self.inner.clone(),
        }
    }
}

/// Stock intraday (real-time) endpoints client
///
/// Access via `client.stock.intraday`
#[pyclass]
pub struct StockIntradayClient {
    inner: marketdata_core::RestClient,
}

#[pymethods]
impl StockIntradayClient {
    /// Get intraday quote for a stock symbol
    ///
    /// Args:
    ///     symbol: Stock symbol (e.g., "2330" for TSMC)
    ///     odd_lot: Whether to query odd lot data (default: False)
    ///
    /// Returns:
    ///     dict: Quote data including prices, order book, and trading info
    ///
    /// Raises:
    ///     MarketDataError: If the request fails
    ///
    /// Example:
    ///     ```python
    ///     quote = client.stock.intraday.quote("2330")
    ///     print(f"Last price: {quote['lastPrice']}")
    ///     print(f"Change: {quote['change']}")
    ///     ```
    #[pyo3(signature = (symbol, odd_lot=false))]
    #[allow(deprecated)]  // allow_threads is deprecated in PyO3 0.27 but replacement not yet available
    pub fn quote(&self, py: Python<'_>, symbol: &str, odd_lot: bool) -> PyResult<Py<PyDict>> {
        // Clone the necessary data before releasing GIL
        let client = self.inner.clone();
        let symbol_owned = symbol.to_string();

        // Release GIL during blocking HTTP call
        let result = py.allow_threads(move || {
            let stock = client.stock();
            let intraday = stock.intraday();
            let mut builder = intraday.quote().symbol(&symbol_owned);
            if odd_lot {
                builder = builder.odd_lot(true);
            }
            builder.send()
        });

        match result {
            Ok(quote) => types::quote_to_dict(py, &quote),
            Err(e) => Err(errors::to_py_err(e)),
        }
    }
}

/// Futures and options market data client
///
/// Access via `client.futopt`
#[pyclass]
pub struct FutOptClient {
    inner: marketdata_core::RestClient,
}

#[pymethods]
impl FutOptClient {
    /// Access intraday (real-time) FutOpt endpoints
    ///
    /// Returns:
    ///     FutOptIntradayClient for accessing intraday endpoints
    #[getter]
    pub fn intraday(&self) -> FutOptIntradayClient {
        FutOptIntradayClient {
            inner: self.inner.clone(),
        }
    }
}

/// FutOpt intraday (real-time) endpoints client
///
/// Access via `client.futopt.intraday`
#[pyclass]
pub struct FutOptIntradayClient {
    inner: marketdata_core::RestClient,
}

#[pymethods]
impl FutOptIntradayClient {
    /// Get intraday quote for a futures/options contract
    ///
    /// Args:
    ///     symbol: Contract symbol (e.g., "TXFC4" for TAIEX futures)
    ///     after_hours: Whether to query after-hours session data (default: False)
    ///
    /// Returns:
    ///     dict: Quote data including prices, order book, and trading info
    ///
    /// Raises:
    ///     MarketDataError: If the request fails
    ///
    /// Example:
    ///     ```python
    ///     # Regular session
    ///     quote = client.futopt.intraday.quote("TXFC4")
    ///
    ///     # After-hours session
    ///     ah_quote = client.futopt.intraday.quote("TXFC4", after_hours=True)
    ///     ```
    #[pyo3(signature = (symbol, after_hours=false))]
    #[allow(deprecated)]  // allow_threads is deprecated in PyO3 0.27 but replacement not yet available
    pub fn quote(&self, py: Python<'_>, symbol: &str, after_hours: bool) -> PyResult<Py<PyDict>> {
        // Clone the necessary data before releasing GIL
        let client = self.inner.clone();
        let symbol_owned = symbol.to_string();

        // Release GIL during blocking HTTP call
        let result = py.allow_threads(move || {
            let futopt = client.futopt();
            let intraday = futopt.intraday();
            let mut builder = intraday.quote().symbol(&symbol_owned);
            if after_hours {
                builder = builder.after_hours();
            }
            builder.send()
        });

        match result {
            Ok(quote) => types::futopt_quote_to_dict(py, &quote),
            Err(e) => Err(errors::to_py_err(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rest_client_creation() {
        let _client = RestClient::new("test-key".to_string());
        // Client should be created without error
    }

    #[test]
    fn test_rest_client_with_bearer_token() {
        let _client = RestClient::with_bearer_token("test-token".to_string());
        // Client should be created without error
    }

    #[test]
    fn test_rest_client_with_sdk_token() {
        let _client = RestClient::with_sdk_token("test-sdk-token".to_string());
        // Client should be created without error
    }
}
