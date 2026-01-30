//! Python REST client wrapper
//!
//! Provides Python-friendly interface to marketdata-core RestClient.
//! Mirrors the official SDK's API: `client.stock.intraday.quote()` pattern.

use pyo3::prelude::*;
use pyo3_async_runtimes::tokio::future_into_py;

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
    ///     Awaitable[dict]: Quote data including prices, order book, and trading info
    ///
    /// Raises:
    ///     MarketDataError: If the request fails
    ///
    /// Example:
    ///     ```python
    ///     quote = await client.stock.intraday.quote("2330")
    ///     print(f"Last price: {quote['lastPrice']}")
    ///     print(f"Change: {quote['change']}")
    ///     ```
    #[pyo3(signature = (symbol, odd_lot=false))]
    pub fn quote<'py>(&self, py: Python<'py>, symbol: String, odd_lot: bool) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        future_into_py(py, async move {
            let result = tokio::task::spawn_blocking(move || {
                let stock = client.stock();
                let intraday = stock.intraday();
                let mut builder = intraday.quote().symbol(&symbol);
                if odd_lot {
                    builder = builder.odd_lot(true);
                }
                builder.send()
            }).await
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("Task join error: {}", e)))?;

            match result {
                Ok(quote) => Python::attach(|py| types::quote_to_dict(py, &quote)),
                Err(e) => Err(errors::to_py_err(e)),
            }
        })
    }

    /// Get ticker information for a stock symbol
    ///
    /// Args:
    ///     symbol: Stock symbol (e.g., "2330" for TSMC)
    ///
    /// Returns:
    ///     Awaitable[dict]: Ticker data
    ///
    /// Raises:
    ///     MarketDataError: If the request fails
    ///
    /// Example:
    ///     ```python
    ///     ticker = await client.stock.intraday.ticker("2330")
    ///     ```
    pub fn ticker<'py>(&self, py: Python<'py>, symbol: String) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        future_into_py(py, async move {
            let result = tokio::task::spawn_blocking(move || {
                let stock = client.stock();
                let intraday = stock.intraday();
                intraday.ticker().symbol(&symbol).send()
            }).await
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("Task join error: {}", e)))?;

            match result {
                Ok(ticker) => Python::attach(|py| types::ticker_to_dict(py, &ticker)),
                Err(e) => Err(errors::to_py_err(e)),
            }
        })
    }

    /// Get candlestick chart data
    ///
    /// Args:
    ///     symbol: Stock symbol (e.g., "2330" for TSMC)
    ///     timeframe: Timeframe in minutes (default: "1")
    ///
    /// Returns:
    ///     Awaitable[dict]: Candlestick data
    ///
    /// Raises:
    ///     MarketDataError: If the request fails
    ///
    /// Example:
    ///     ```python
    ///     candles = await client.stock.intraday.candles("2330", "5")
    ///     ```
    #[pyo3(signature = (symbol, timeframe="1".to_string()))]
    pub fn candles<'py>(&self, py: Python<'py>, symbol: String, timeframe: String) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        future_into_py(py, async move {
            let result = tokio::task::spawn_blocking(move || {
                let stock = client.stock();
                let intraday = stock.intraday();
                intraday.candles().symbol(&symbol).timeframe(&timeframe).send()
            }).await
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("Task join error: {}", e)))?;

            match result {
                Ok(candles) => Python::attach(|py| types::candles_to_dict(py, &candles)),
                Err(e) => Err(errors::to_py_err(e)),
            }
        })
    }

    /// Get trade ticks data
    ///
    /// Args:
    ///     symbol: Stock symbol (e.g., "2330" for TSMC)
    ///
    /// Returns:
    ///     Awaitable[dict]: Trade ticks data
    ///
    /// Raises:
    ///     MarketDataError: If the request fails
    ///
    /// Example:
    ///     ```python
    ///     trades = await client.stock.intraday.trades("2330")
    ///     ```
    pub fn trades<'py>(&self, py: Python<'py>, symbol: String) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        future_into_py(py, async move {
            let result = tokio::task::spawn_blocking(move || {
                let stock = client.stock();
                let intraday = stock.intraday();
                intraday.trades().symbol(&symbol).send()
            }).await
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("Task join error: {}", e)))?;

            match result {
                Ok(trades) => Python::attach(|py| types::trades_to_dict(py, &trades)),
                Err(e) => Err(errors::to_py_err(e)),
            }
        })
    }

    /// Get volume data
    ///
    /// Args:
    ///     symbol: Stock symbol (e.g., "2330" for TSMC)
    ///
    /// Returns:
    ///     Awaitable[dict]: Volume data
    ///
    /// Raises:
    ///     MarketDataError: If the request fails
    ///
    /// Example:
    ///     ```python
    ///     volumes = await client.stock.intraday.volumes("2330")
    ///     ```
    pub fn volumes<'py>(&self, py: Python<'py>, symbol: String) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        future_into_py(py, async move {
            let result = tokio::task::spawn_blocking(move || {
                let stock = client.stock();
                let intraday = stock.intraday();
                intraday.volumes().symbol(&symbol).send()
            }).await
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("Task join error: {}", e)))?;

            match result {
                Ok(volumes) => Python::attach(|py| types::volumes_to_dict(py, &volumes)),
                Err(e) => Err(errors::to_py_err(e)),
            }
        })
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
    ///     Awaitable[dict]: Quote data including prices, order book, and trading info
    ///
    /// Raises:
    ///     MarketDataError: If the request fails
    ///
    /// Example:
    ///     ```python
    ///     # Regular session
    ///     quote = await client.futopt.intraday.quote("TXFC4")
    ///
    ///     # After-hours session
    ///     ah_quote = await client.futopt.intraday.quote("TXFC4", after_hours=True)
    ///     ```
    #[pyo3(signature = (symbol, after_hours=false))]
    pub fn quote<'py>(&self, py: Python<'py>, symbol: String, after_hours: bool) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        future_into_py(py, async move {
            let result = tokio::task::spawn_blocking(move || {
                let futopt = client.futopt();
                let intraday = futopt.intraday();
                let mut builder = intraday.quote().symbol(&symbol);
                if after_hours {
                    builder = builder.after_hours();
                }
                builder.send()
            }).await
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("Task join error: {}", e)))?;

            match result {
                Ok(quote) => Python::attach(|py| types::futopt_quote_to_dict(py, &quote)),
                Err(e) => Err(errors::to_py_err(e)),
            }
        })
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
