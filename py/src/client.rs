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

    /// Access historical stock data endpoints
    ///
    /// Returns:
    ///     StockHistoricalClient for accessing historical endpoints
    #[getter]
    pub fn historical(&self) -> StockHistoricalClient {
        StockHistoricalClient {
            inner: self.inner.clone(),
        }
    }

    /// Access snapshot endpoints for market-wide data
    ///
    /// Returns:
    ///     StockSnapshotClient for accessing snapshot endpoints
    #[getter]
    pub fn snapshot(&self) -> StockSnapshotClient {
        StockSnapshotClient {
            inner: self.inner.clone(),
        }
    }

    /// Access technical indicator endpoints
    ///
    /// Returns:
    ///     StockTechnicalClient for accessing technical endpoints
    #[getter]
    pub fn technical(&self) -> StockTechnicalClient {
        StockTechnicalClient {
            inner: self.inner.clone(),
        }
    }

    /// Access corporate actions endpoints
    ///
    /// Returns:
    ///     StockCorporateActionsClient for accessing corporate actions endpoints
    #[getter]
    pub fn corporate_actions(&self) -> StockCorporateActionsClient {
        StockCorporateActionsClient {
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
    #[pyo3(signature = (symbol))]
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
    #[pyo3(signature = (symbol))]
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
    #[pyo3(signature = (symbol))]
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

/// Stock historical data endpoints client
///
/// Access via `client.stock.historical`
#[pyclass]
pub struct StockHistoricalClient {
    inner: marketdata_core::RestClient,
}

#[pymethods]
impl StockHistoricalClient {
    /// Get historical candles for a stock symbol
    ///
    /// Args:
    ///     symbol: Stock symbol (e.g., "2330" for TSMC)
    ///     from_date: Start date (YYYY-MM-DD)
    ///     to_date: End date (YYYY-MM-DD)
    ///     timeframe: Timeframe ("D", "W", "M", "1", "5", "10", "15", "30", "60")
    ///     fields: Optional field selection
    ///     sort: Sort order ("asc" or "desc")
    ///     adjusted: Whether to adjust for splits/dividends
    ///
    /// Returns:
    ///     Awaitable[dict]: Historical candles data
    ///
    /// Example:
    ///     ```python
    ///     candles = await client.stock.historical.candles(
    ///         "2330",
    ///         from_date="2024-01-01",
    ///         to_date="2024-01-31",
    ///         timeframe="D"
    ///     )
    ///     ```
    #[pyo3(signature = (symbol, from_date=None, to_date=None, timeframe=None, fields=None, sort=None, adjusted=None))]
    pub fn candles<'py>(
        &self,
        py: Python<'py>,
        symbol: String,
        from_date: Option<String>,
        to_date: Option<String>,
        timeframe: Option<String>,
        fields: Option<String>,
        sort: Option<String>,
        adjusted: Option<bool>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        future_into_py(py, async move {
            let result = tokio::task::spawn_blocking(move || {
                let stock = client.stock();
                let historical = stock.historical();
                let mut builder = historical.candles().symbol(&symbol);
                if let Some(f) = from_date {
                    builder = builder.from(&f);
                }
                if let Some(t) = to_date {
                    builder = builder.to(&t);
                }
                if let Some(tf) = timeframe {
                    builder = builder.timeframe(&tf);
                }
                if let Some(fld) = fields {
                    builder = builder.fields(&fld);
                }
                if let Some(s) = sort {
                    builder = builder.sort(&s);
                }
                if let Some(adj) = adjusted {
                    builder = builder.adjusted(adj);
                }
                builder.send()
            })
            .await
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("Task join error: {}", e)))?;

            match result {
                Ok(candles) => Python::attach(|py| types::historical_candles_to_dict(py, &candles)),
                Err(e) => Err(errors::to_py_err(e)),
            }
        })
    }

    /// Get historical stats for a stock symbol
    ///
    /// Args:
    ///     symbol: Stock symbol (e.g., "2330" for TSMC)
    ///
    /// Returns:
    ///     Awaitable[dict]: Historical stats data including 52-week high/low
    ///
    /// Example:
    ///     ```python
    ///     stats = await client.stock.historical.stats("2330")
    ///     print(f"52-week high: {stats['week52High']}")
    ///     ```
    #[pyo3(signature = (symbol))]
    pub fn stats<'py>(&self, py: Python<'py>, symbol: String) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        future_into_py(py, async move {
            let result = tokio::task::spawn_blocking(move || {
                let stock = client.stock();
                let historical = stock.historical();
                historical.stats().symbol(&symbol).send()
            })
            .await
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("Task join error: {}", e)))?;

            match result {
                Ok(stats) => Python::attach(|py| types::stats_to_dict(py, &stats)),
                Err(e) => Err(errors::to_py_err(e)),
            }
        })
    }
}

/// Stock snapshot endpoints client
///
/// Access via `client.stock.snapshot`
#[pyclass]
pub struct StockSnapshotClient {
    inner: marketdata_core::RestClient,
}

#[pymethods]
impl StockSnapshotClient {
    /// Get snapshot quotes for a market
    ///
    /// Args:
    ///     market: Market code ("TSE", "OTC", "ESB", "TIB", "PSB")
    ///     type_filter: Type filter ("ALL", "ALLBUT0999", "COMMONSTOCK")
    ///
    /// Returns:
    ///     Awaitable[dict]: Market-wide quotes snapshot
    ///
    /// Example:
    ///     ```python
    ///     quotes = await client.stock.snapshot.quotes("TSE", type_filter="COMMONSTOCK")
    ///     ```
    #[pyo3(signature = (market, type_filter=None))]
    pub fn quotes<'py>(
        &self,
        py: Python<'py>,
        market: String,
        type_filter: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        future_into_py(py, async move {
            let result = tokio::task::spawn_blocking(move || {
                let stock = client.stock();
                let snapshot = stock.snapshot();
                let mut builder = snapshot.quotes().market(&market);
                if let Some(tf) = type_filter {
                    builder = builder.type_filter(&tf);
                }
                builder.send()
            })
            .await
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("Task join error: {}", e)))?;

            match result {
                Ok(quotes) => Python::attach(|py| types::snapshot_quotes_to_dict(py, &quotes)),
                Err(e) => Err(errors::to_py_err(e)),
            }
        })
    }

    /// Get top movers for a market
    ///
    /// Args:
    ///     market: Market code ("TSE", "OTC", "ESB", "TIB", "PSB")
    ///     direction: Direction filter ("up" for gainers, "down" for losers)
    ///     change: Change type ("percent" or "value")
    ///
    /// Returns:
    ///     Awaitable[dict]: Top movers data
    ///
    /// Example:
    ///     ```python
    ///     movers = await client.stock.snapshot.movers("TSE", direction="up", change="percent")
    ///     ```
    #[pyo3(signature = (market, direction=None, change=None))]
    pub fn movers<'py>(
        &self,
        py: Python<'py>,
        market: String,
        direction: Option<String>,
        change: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        future_into_py(py, async move {
            let result = tokio::task::spawn_blocking(move || {
                let stock = client.stock();
                let snapshot = stock.snapshot();
                let mut builder = snapshot.movers().market(&market);
                if let Some(d) = direction {
                    builder = builder.direction(&d);
                }
                if let Some(c) = change {
                    builder = builder.change(&c);
                }
                builder.send()
            })
            .await
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("Task join error: {}", e)))?;

            match result {
                Ok(movers) => Python::attach(|py| types::movers_to_dict(py, &movers)),
                Err(e) => Err(errors::to_py_err(e)),
            }
        })
    }

    /// Get most active stocks for a market
    ///
    /// Args:
    ///     market: Market code ("TSE", "OTC", "ESB", "TIB", "PSB")
    ///     trade: Trade type ("volume" or "value")
    ///
    /// Returns:
    ///     Awaitable[dict]: Most active stocks data
    ///
    /// Example:
    ///     ```python
    ///     actives = await client.stock.snapshot.actives("TSE", trade="volume")
    ///     ```
    #[pyo3(signature = (market, trade=None))]
    pub fn actives<'py>(
        &self,
        py: Python<'py>,
        market: String,
        trade: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        future_into_py(py, async move {
            let result = tokio::task::spawn_blocking(move || {
                let stock = client.stock();
                let snapshot = stock.snapshot();
                let mut builder = snapshot.actives().market(&market);
                if let Some(t) = trade {
                    builder = builder.trade(&t);
                }
                builder.send()
            })
            .await
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("Task join error: {}", e)))?;

            match result {
                Ok(actives) => Python::attach(|py| types::actives_to_dict(py, &actives)),
                Err(e) => Err(errors::to_py_err(e)),
            }
        })
    }
}

/// Stock technical indicator endpoints client
///
/// Access via `client.stock.technical`
#[pyclass]
pub struct StockTechnicalClient {
    inner: marketdata_core::RestClient,
}

#[pymethods]
impl StockTechnicalClient {
    /// Get Simple Moving Average (SMA) data
    ///
    /// Args:
    ///     symbol: Stock symbol (e.g., "2330" for TSMC)
    ///     from_date: Start date (YYYY-MM-DD)
    ///     to_date: End date (YYYY-MM-DD)
    ///     timeframe: Timeframe ("D", "W", "M", "1", "5", etc.)
    ///     period: Moving average period
    ///
    /// Returns:
    ///     Awaitable[dict]: SMA indicator data
    #[pyo3(signature = (symbol, from_date=None, to_date=None, timeframe=None, period=None))]
    pub fn sma<'py>(
        &self,
        py: Python<'py>,
        symbol: String,
        from_date: Option<String>,
        to_date: Option<String>,
        timeframe: Option<String>,
        period: Option<u32>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        future_into_py(py, async move {
            let result = tokio::task::spawn_blocking(move || {
                let stock = client.stock();
                let technical = stock.technical();
                let mut builder = technical.sma().symbol(&symbol);
                if let Some(f) = from_date {
                    builder = builder.from(&f);
                }
                if let Some(t) = to_date {
                    builder = builder.to(&t);
                }
                if let Some(tf) = timeframe {
                    builder = builder.timeframe(&tf);
                }
                if let Some(p) = period {
                    builder = builder.period(p);
                }
                builder.send()
            })
            .await
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("Task join error: {}", e)))?;

            match result {
                Ok(sma) => Python::attach(|py| types::technical_to_dict(py, &sma)),
                Err(e) => Err(errors::to_py_err(e)),
            }
        })
    }

    /// Get Relative Strength Index (RSI) data
    ///
    /// Args:
    ///     symbol: Stock symbol (e.g., "2330" for TSMC)
    ///     from_date: Start date (YYYY-MM-DD)
    ///     to_date: End date (YYYY-MM-DD)
    ///     timeframe: Timeframe ("D", "W", "M", "1", "5", etc.)
    ///     period: RSI period (default 14)
    ///
    /// Returns:
    ///     Awaitable[dict]: RSI indicator data
    #[pyo3(signature = (symbol, from_date=None, to_date=None, timeframe=None, period=None))]
    pub fn rsi<'py>(
        &self,
        py: Python<'py>,
        symbol: String,
        from_date: Option<String>,
        to_date: Option<String>,
        timeframe: Option<String>,
        period: Option<u32>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        future_into_py(py, async move {
            let result = tokio::task::spawn_blocking(move || {
                let stock = client.stock();
                let technical = stock.technical();
                let mut builder = technical.rsi().symbol(&symbol);
                if let Some(f) = from_date {
                    builder = builder.from(&f);
                }
                if let Some(t) = to_date {
                    builder = builder.to(&t);
                }
                if let Some(tf) = timeframe {
                    builder = builder.timeframe(&tf);
                }
                if let Some(p) = period {
                    builder = builder.period(p);
                }
                builder.send()
            })
            .await
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("Task join error: {}", e)))?;

            match result {
                Ok(rsi) => Python::attach(|py| types::technical_to_dict(py, &rsi)),
                Err(e) => Err(errors::to_py_err(e)),
            }
        })
    }

    /// Get KDJ (Stochastic Oscillator) data
    ///
    /// Args:
    ///     symbol: Stock symbol (e.g., "2330" for TSMC)
    ///     from_date: Start date (YYYY-MM-DD)
    ///     to_date: End date (YYYY-MM-DD)
    ///     timeframe: Timeframe ("D", "W", "M", "1", "5", etc.)
    ///     period: KDJ period
    ///
    /// Returns:
    ///     Awaitable[dict]: KDJ indicator data with K, D, J values
    #[pyo3(signature = (symbol, from_date=None, to_date=None, timeframe=None, period=None))]
    pub fn kdj<'py>(
        &self,
        py: Python<'py>,
        symbol: String,
        from_date: Option<String>,
        to_date: Option<String>,
        timeframe: Option<String>,
        period: Option<u32>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        future_into_py(py, async move {
            let result = tokio::task::spawn_blocking(move || {
                let stock = client.stock();
                let technical = stock.technical();
                let mut builder = technical.kdj().symbol(&symbol);
                if let Some(f) = from_date {
                    builder = builder.from(&f);
                }
                if let Some(t) = to_date {
                    builder = builder.to(&t);
                }
                if let Some(tf) = timeframe {
                    builder = builder.timeframe(&tf);
                }
                if let Some(p) = period {
                    builder = builder.period(p);
                }
                builder.send()
            })
            .await
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("Task join error: {}", e)))?;

            match result {
                Ok(kdj) => Python::attach(|py| types::technical_to_dict(py, &kdj)),
                Err(e) => Err(errors::to_py_err(e)),
            }
        })
    }

    /// Get MACD (Moving Average Convergence Divergence) data
    ///
    /// Args:
    ///     symbol: Stock symbol (e.g., "2330" for TSMC)
    ///     from_date: Start date (YYYY-MM-DD)
    ///     to_date: End date (YYYY-MM-DD)
    ///     timeframe: Timeframe ("D", "W", "M", "1", "5", etc.)
    ///     fast: Fast EMA period (default 12)
    ///     slow: Slow EMA period (default 26)
    ///     signal: Signal line period (default 9)
    ///
    /// Returns:
    ///     Awaitable[dict]: MACD indicator data with MACD, signal, histogram
    #[pyo3(signature = (symbol, from_date=None, to_date=None, timeframe=None, fast=None, slow=None, signal=None))]
    pub fn macd<'py>(
        &self,
        py: Python<'py>,
        symbol: String,
        from_date: Option<String>,
        to_date: Option<String>,
        timeframe: Option<String>,
        fast: Option<u32>,
        slow: Option<u32>,
        signal: Option<u32>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        future_into_py(py, async move {
            let result = tokio::task::spawn_blocking(move || {
                let stock = client.stock();
                let technical = stock.technical();
                let mut builder = technical.macd().symbol(&symbol);
                if let Some(f) = from_date {
                    builder = builder.from(&f);
                }
                if let Some(t) = to_date {
                    builder = builder.to(&t);
                }
                if let Some(tf) = timeframe {
                    builder = builder.timeframe(&tf);
                }
                if let Some(fst) = fast {
                    builder = builder.fast(fst);
                }
                if let Some(slw) = slow {
                    builder = builder.slow(slw);
                }
                if let Some(sig) = signal {
                    builder = builder.signal(sig);
                }
                builder.send()
            })
            .await
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("Task join error: {}", e)))?;

            match result {
                Ok(macd) => Python::attach(|py| types::technical_to_dict(py, &macd)),
                Err(e) => Err(errors::to_py_err(e)),
            }
        })
    }

    /// Get Bollinger Bands (BB) data
    ///
    /// Args:
    ///     symbol: Stock symbol (e.g., "2330" for TSMC)
    ///     from_date: Start date (YYYY-MM-DD)
    ///     to_date: End date (YYYY-MM-DD)
    ///     timeframe: Timeframe ("D", "W", "M", "1", "5", etc.)
    ///     period: Moving average period (default 20)
    ///     stddev: Standard deviation multiplier (default 2.0)
    ///
    /// Returns:
    ///     Awaitable[dict]: Bollinger Bands data with upper, middle, lower bands
    #[pyo3(signature = (symbol, from_date=None, to_date=None, timeframe=None, period=None, stddev=None))]
    pub fn bb<'py>(
        &self,
        py: Python<'py>,
        symbol: String,
        from_date: Option<String>,
        to_date: Option<String>,
        timeframe: Option<String>,
        period: Option<u32>,
        stddev: Option<f64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        future_into_py(py, async move {
            let result = tokio::task::spawn_blocking(move || {
                let stock = client.stock();
                let technical = stock.technical();
                let mut builder = technical.bb().symbol(&symbol);
                if let Some(f) = from_date {
                    builder = builder.from(&f);
                }
                if let Some(t) = to_date {
                    builder = builder.to(&t);
                }
                if let Some(tf) = timeframe {
                    builder = builder.timeframe(&tf);
                }
                if let Some(p) = period {
                    builder = builder.period(p);
                }
                if let Some(sd) = stddev {
                    builder = builder.stddev(sd);
                }
                builder.send()
            })
            .await
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("Task join error: {}", e)))?;

            match result {
                Ok(bb) => Python::attach(|py| types::technical_to_dict(py, &bb)),
                Err(e) => Err(errors::to_py_err(e)),
            }
        })
    }
}

/// Stock corporate actions endpoints client
///
/// Access via `client.stock.corporate_actions`
#[pyclass]
pub struct StockCorporateActionsClient {
    inner: marketdata_core::RestClient,
}

#[pymethods]
impl StockCorporateActionsClient {
    /// Get capital changes (stock splits, rights issues, etc.)
    ///
    /// Args:
    ///     date: Specific date (YYYY-MM-DD)
    ///     start_date: Start date for range query (YYYY-MM-DD)
    ///     end_date: End date for range query (YYYY-MM-DD)
    ///
    /// Returns:
    ///     Awaitable[dict]: Capital changes data
    ///
    /// Example:
    ///     ```python
    ///     changes = await client.stock.corporate_actions.capital_changes(
    ///         start_date="2024-01-01",
    ///         end_date="2024-01-31"
    ///     )
    ///     ```
    #[pyo3(signature = (date=None, start_date=None, end_date=None))]
    pub fn capital_changes<'py>(
        &self,
        py: Python<'py>,
        date: Option<String>,
        start_date: Option<String>,
        end_date: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        future_into_py(py, async move {
            let result = tokio::task::spawn_blocking(move || {
                let stock = client.stock();
                let corp = stock.corporate_actions();
                let mut builder = corp.capital_changes();
                if let Some(d) = date {
                    builder = builder.date(&d);
                }
                if let Some(sd) = start_date {
                    builder = builder.start_date(&sd);
                }
                if let Some(ed) = end_date {
                    builder = builder.end_date(&ed);
                }
                builder.send()
            })
            .await
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("Task join error: {}", e)))?;

            match result {
                Ok(changes) => Python::attach(|py| types::corporate_action_to_dict(py, &changes)),
                Err(e) => Err(errors::to_py_err(e)),
            }
        })
    }

    /// Get dividend announcements
    ///
    /// Args:
    ///     date: Specific date (YYYY-MM-DD)
    ///     start_date: Start date for range query (YYYY-MM-DD)
    ///     end_date: End date for range query (YYYY-MM-DD)
    ///
    /// Returns:
    ///     Awaitable[dict]: Dividend data
    ///
    /// Example:
    ///     ```python
    ///     dividends = await client.stock.corporate_actions.dividends(
    ///         start_date="2024-01-01",
    ///         end_date="2024-12-31"
    ///     )
    ///     ```
    #[pyo3(signature = (date=None, start_date=None, end_date=None))]
    pub fn dividends<'py>(
        &self,
        py: Python<'py>,
        date: Option<String>,
        start_date: Option<String>,
        end_date: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        future_into_py(py, async move {
            let result = tokio::task::spawn_blocking(move || {
                let stock = client.stock();
                let corp = stock.corporate_actions();
                let mut builder = corp.dividends();
                if let Some(d) = date {
                    builder = builder.date(&d);
                }
                if let Some(sd) = start_date {
                    builder = builder.start_date(&sd);
                }
                if let Some(ed) = end_date {
                    builder = builder.end_date(&ed);
                }
                builder.send()
            })
            .await
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("Task join error: {}", e)))?;

            match result {
                Ok(dividends) => Python::attach(|py| types::corporate_action_to_dict(py, &dividends)),
                Err(e) => Err(errors::to_py_err(e)),
            }
        })
    }

    /// Get IPO listing applicants
    ///
    /// Args:
    ///     date: Specific date (YYYY-MM-DD)
    ///     start_date: Start date for range query (YYYY-MM-DD)
    ///     end_date: End date for range query (YYYY-MM-DD)
    ///
    /// Returns:
    ///     Awaitable[dict]: Listing applicants data
    ///
    /// Example:
    ///     ```python
    ///     applicants = await client.stock.corporate_actions.listing_applicants()
    ///     ```
    #[pyo3(signature = (date=None, start_date=None, end_date=None))]
    pub fn listing_applicants<'py>(
        &self,
        py: Python<'py>,
        date: Option<String>,
        start_date: Option<String>,
        end_date: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        future_into_py(py, async move {
            let result = tokio::task::spawn_blocking(move || {
                let stock = client.stock();
                let corp = stock.corporate_actions();
                let mut builder = corp.listing_applicants();
                if let Some(d) = date {
                    builder = builder.date(&d);
                }
                if let Some(sd) = start_date {
                    builder = builder.start_date(&sd);
                }
                if let Some(ed) = end_date {
                    builder = builder.end_date(&ed);
                }
                builder.send()
            })
            .await
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("Task join error: {}", e)))?;

            match result {
                Ok(applicants) => Python::attach(|py| types::corporate_action_to_dict(py, &applicants)),
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

    /// Access historical FutOpt data endpoints
    ///
    /// Returns:
    ///     FutOptHistoricalClient for accessing historical endpoints
    #[getter]
    pub fn historical(&self) -> FutOptHistoricalClient {
        FutOptHistoricalClient {
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

/// FutOpt historical data endpoints client
///
/// Access via `client.futopt.historical`
#[pyclass]
pub struct FutOptHistoricalClient {
    inner: marketdata_core::RestClient,
}

#[pymethods]
impl FutOptHistoricalClient {
    /// Get historical candles for a FutOpt contract
    ///
    /// Args:
    ///     symbol: Contract symbol (e.g., "TXFC4" for TAIEX futures)
    ///     from_date: Start date (YYYY-MM-DD)
    ///     to_date: End date (YYYY-MM-DD)
    ///     timeframe: Timeframe ("D", "W", "M", "1", "5", "10", "15", "30", "60")
    ///     after_hours: Whether to include after-hours session data (default: False)
    ///
    /// Returns:
    ///     Awaitable[dict]: Historical candles data
    ///
    /// Example:
    ///     ```python
    ///     candles = await client.futopt.historical.candles(
    ///         "TXFC4",
    ///         from_date="2024-01-01",
    ///         to_date="2024-01-31",
    ///         timeframe="D"
    ///     )
    ///     ```
    #[pyo3(signature = (symbol, from_date=None, to_date=None, timeframe=None, after_hours=false))]
    pub fn candles<'py>(
        &self,
        py: Python<'py>,
        symbol: String,
        from_date: Option<String>,
        to_date: Option<String>,
        timeframe: Option<String>,
        after_hours: bool,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        future_into_py(py, async move {
            let result = tokio::task::spawn_blocking(move || {
                let futopt = client.futopt();
                let historical = futopt.historical();
                let mut builder = historical.candles().symbol(&symbol);
                if let Some(f) = from_date {
                    builder = builder.from(&f);
                }
                if let Some(t) = to_date {
                    builder = builder.to(&t);
                }
                if let Some(tf) = timeframe {
                    builder = builder.timeframe(&tf);
                }
                if after_hours {
                    builder = builder.after_hours(true);
                }
                builder.send()
            })
            .await
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("Task join error: {}", e)))?;

            match result {
                Ok(candles) => Python::attach(|py| types::futopt_historical_candles_to_dict(py, &candles)),
                Err(e) => Err(errors::to_py_err(e)),
            }
        })
    }

    /// Get daily historical data for a FutOpt contract
    ///
    /// Args:
    ///     symbol: Contract symbol (e.g., "TXFC4" for TAIEX futures)
    ///     from_date: Start date (YYYY-MM-DD)
    ///     to_date: End date (YYYY-MM-DD)
    ///     after_hours: Whether to include after-hours session data (default: False)
    ///
    /// Returns:
    ///     Awaitable[dict]: Daily historical data with settlement prices
    ///
    /// Example:
    ///     ```python
    ///     daily = await client.futopt.historical.daily(
    ///         "TXFC4",
    ///         from_date="2024-01-01",
    ///         to_date="2024-01-31"
    ///     )
    ///     ```
    #[pyo3(signature = (symbol, from_date=None, to_date=None, after_hours=false))]
    pub fn daily<'py>(
        &self,
        py: Python<'py>,
        symbol: String,
        from_date: Option<String>,
        to_date: Option<String>,
        after_hours: bool,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        future_into_py(py, async move {
            let result = tokio::task::spawn_blocking(move || {
                let futopt = client.futopt();
                let historical = futopt.historical();
                let mut builder = historical.daily().symbol(&symbol);
                if let Some(f) = from_date {
                    builder = builder.from(&f);
                }
                if let Some(t) = to_date {
                    builder = builder.to(&t);
                }
                if after_hours {
                    builder = builder.after_hours(true);
                }
                builder.send()
            })
            .await
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("Task join error: {}", e)))?;

            match result {
                Ok(daily) => Python::attach(|py| types::futopt_daily_to_dict(py, &daily)),
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
