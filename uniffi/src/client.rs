//! REST client wrapper types for UniFFI bindings
//!
//! This module provides Arc-wrapped client types that can be safely passed across FFI boundaries.
//! All methods return typed models (Quote, Ticker, etc.) via async methods.
//! Sync variants are provided for simple use cases.

use std::sync::Arc;
use marketdata_core::{Auth, RestClient as CoreRestClient};
use crate::errors::MarketDataError;
use crate::models::{
    Quote, Ticker, TradesResponse, IntradayCandlesResponse, VolumesResponse,
    FutOptQuote, FutOptTicker, ProductsResponse,
};

// ============================================================================
// RestClient - Main entry point
// ============================================================================

/// REST client for UniFFI bindings
///
/// Wraps the core RestClient and provides Arc-wrapped sub-clients for FFI safety.
#[derive(uniffi::Object)]
pub struct RestClient {
    inner: CoreRestClient,
}

impl RestClient {
    /// Create a new REST client with the given authentication
    pub fn new(auth: Auth) -> Self {
        Self {
            inner: CoreRestClient::new(auth),
        }
    }
}

#[uniffi::export]
impl RestClient {
    /// Access stock-related endpoints
    pub fn stock(&self) -> Arc<StockClient> {
        Arc::new(StockClient::new(self.inner.clone()))
    }

    /// Access FutOpt (futures and options) endpoints
    pub fn futopt(&self) -> Arc<FutOptClient> {
        Arc::new(FutOptClient::new(self.inner.clone()))
    }
}

// ============================================================================
// Stock Client Hierarchy
// ============================================================================

/// Stock market data client
#[derive(uniffi::Object)]
pub struct StockClient {
    inner: CoreRestClient,
}

impl StockClient {
    pub fn new(client: CoreRestClient) -> Self {
        Self { inner: client }
    }
}

#[uniffi::export]
impl StockClient {
    /// Access intraday (real-time) endpoints
    pub fn intraday(&self) -> Arc<StockIntradayClient> {
        Arc::new(StockIntradayClient::new(self.inner.clone()))
    }
}

/// Stock intraday endpoints with typed model returns
///
/// All methods have both async (get_*) and sync (*_sync) variants:
/// - Async methods are preferred for best performance (non-blocking)
/// - Sync methods block the calling thread (simpler API for scripting)
#[derive(uniffi::Object)]
pub struct StockIntradayClient {
    inner: CoreRestClient,
}

impl StockIntradayClient {
    pub fn new(client: CoreRestClient) -> Self {
        Self { inner: client }
    }
}

#[uniffi::export(async_runtime = "tokio")]
impl StockIntradayClient {
    // ========== Async Methods (Primary) ==========

    /// Get quote for a symbol (async)
    ///
    /// Returns typed Quote model with all fields directly accessible.
    pub async fn get_quote(&self, symbol: String) -> Result<Quote, MarketDataError> {
        let inner = self.inner.clone();
        let result = tokio::task::spawn_blocking(move || {
            inner.stock().intraday().quote().symbol(&symbol).send()
        })
        .await
        .map_err(|e| MarketDataError::Other { msg: e.to_string() })??;

        Ok(result.into())
    }

    /// Get ticker info for a symbol (async)
    ///
    /// Returns typed Ticker model with stock metadata.
    pub async fn get_ticker(&self, symbol: String) -> Result<Ticker, MarketDataError> {
        let inner = self.inner.clone();
        let result = tokio::task::spawn_blocking(move || {
            inner.stock().intraday().ticker().symbol(&symbol).send()
        })
        .await
        .map_err(|e| MarketDataError::Other { msg: e.to_string() })??;

        Ok(result.into())
    }

    /// Get trade history for a symbol (async)
    ///
    /// Returns typed TradesResponse with list of trades.
    pub async fn get_trades(&self, symbol: String) -> Result<TradesResponse, MarketDataError> {
        let inner = self.inner.clone();
        let result = tokio::task::spawn_blocking(move || {
            inner.stock().intraday().trades().symbol(&symbol).send()
        })
        .await
        .map_err(|e| MarketDataError::Other { msg: e.to_string() })??;

        Ok(result.into())
    }

    /// Get candlestick data for a symbol (async)
    ///
    /// timeframe: "1", "5", "10", "15", "30", "60" (minutes)
    /// Returns typed IntradayCandlesResponse with OHLCV data.
    pub async fn get_candles(&self, symbol: String, timeframe: String) -> Result<IntradayCandlesResponse, MarketDataError> {
        let inner = self.inner.clone();
        let result = tokio::task::spawn_blocking(move || {
            inner.stock().intraday().candles()
                .symbol(&symbol)
                .timeframe(&timeframe)
                .send()
        })
        .await
        .map_err(|e| MarketDataError::Other { msg: e.to_string() })??;

        Ok(result.into())
    }

    /// Get volume breakdown for a symbol (async)
    ///
    /// Returns typed VolumesResponse with volume at price data.
    pub async fn get_volumes(&self, symbol: String) -> Result<VolumesResponse, MarketDataError> {
        let inner = self.inner.clone();
        let result = tokio::task::spawn_blocking(move || {
            inner.stock().intraday().volumes().symbol(&symbol).send()
        })
        .await
        .map_err(|e| MarketDataError::Other { msg: e.to_string() })??;

        Ok(result.into())
    }

    // ========== Sync Methods (Blocking) ==========

    /// Get quote for a symbol (sync/blocking)
    pub fn quote_sync(&self, symbol: String) -> Result<Quote, MarketDataError> {
        let result = self.inner.stock().intraday().quote().symbol(&symbol).send()?;
        Ok(result.into())
    }

    /// Get ticker info for a symbol (sync/blocking)
    pub fn ticker_sync(&self, symbol: String) -> Result<Ticker, MarketDataError> {
        let result = self.inner.stock().intraday().ticker().symbol(&symbol).send()?;
        Ok(result.into())
    }

    /// Get trade history for a symbol (sync/blocking)
    pub fn trades_sync(&self, symbol: String) -> Result<TradesResponse, MarketDataError> {
        let result = self.inner.stock().intraday().trades().symbol(&symbol).send()?;
        Ok(result.into())
    }

    /// Get candlestick data for a symbol (sync/blocking)
    pub fn candles_sync(&self, symbol: String, timeframe: String) -> Result<IntradayCandlesResponse, MarketDataError> {
        let result = self.inner.stock().intraday().candles()
            .symbol(&symbol)
            .timeframe(&timeframe)
            .send()?;
        Ok(result.into())
    }

    /// Get volume breakdown for a symbol (sync/blocking)
    pub fn volumes_sync(&self, symbol: String) -> Result<VolumesResponse, MarketDataError> {
        let result = self.inner.stock().intraday().volumes().symbol(&symbol).send()?;
        Ok(result.into())
    }
}

// ============================================================================
// FutOpt Client Hierarchy
// ============================================================================

/// FutOpt market data client
#[derive(uniffi::Object)]
pub struct FutOptClient {
    inner: CoreRestClient,
}

impl FutOptClient {
    pub fn new(client: CoreRestClient) -> Self {
        Self { inner: client }
    }
}

#[uniffi::export]
impl FutOptClient {
    /// Access intraday (real-time) endpoints
    pub fn intraday(&self) -> Arc<FutOptIntradayClient> {
        Arc::new(FutOptIntradayClient::new(self.inner.clone()))
    }
}

/// FutOpt intraday endpoints with typed model returns
#[derive(uniffi::Object)]
pub struct FutOptIntradayClient {
    inner: CoreRestClient,
}

// Non-exported impl block for constructors
impl FutOptIntradayClient {
    pub fn new(client: CoreRestClient) -> Self {
        Self { inner: client }
    }
}

#[uniffi::export(async_runtime = "tokio")]
impl FutOptIntradayClient {
    // ========== Async Methods (Primary) ==========

    /// Get quote for a futures/options contract (async)
    ///
    /// after_hours: true for after-hours session
    pub async fn get_quote(&self, symbol: String, after_hours: bool) -> Result<FutOptQuote, MarketDataError> {
        let inner = self.inner.clone();
        let result = tokio::task::spawn_blocking(move || {
            let mut builder = inner.futopt().intraday().quote().symbol(&symbol);
            if after_hours {
                builder = builder.after_hours();
            }
            builder.send()
        })
        .await
        .map_err(|e| MarketDataError::Other { msg: e.to_string() })??;

        Ok(result.into())
    }

    /// Get ticker info for a contract (async)
    pub async fn get_ticker(&self, symbol: String, after_hours: bool) -> Result<FutOptTicker, MarketDataError> {
        let inner = self.inner.clone();
        let result = tokio::task::spawn_blocking(move || {
            let mut builder = inner.futopt().intraday().ticker().symbol(&symbol);
            if after_hours {
                builder = builder.after_hours();
            }
            builder.send()
        })
        .await
        .map_err(|e| MarketDataError::Other { msg: e.to_string() })??;

        Ok(result.into())
    }

    /// Get available products list (async)
    ///
    /// typ: "F" for futures, "O" for options
    pub async fn get_products(&self, typ: String) -> Result<ProductsResponse, MarketDataError> {
        let futopt_type = parse_futopt_type(&typ)?;
        let inner = self.inner.clone();
        let result = tokio::task::spawn_blocking(move || {
            inner.futopt().intraday().products().typ(futopt_type).send()
        })
        .await
        .map_err(|e| MarketDataError::Other { msg: e.to_string() })??;

        Ok(result.into())
    }

    // ========== Sync Methods (Blocking) ==========

    /// Get quote for a futures/options contract (sync/blocking)
    pub fn quote_sync(&self, symbol: String, after_hours: bool) -> Result<FutOptQuote, MarketDataError> {
        let mut builder = self.inner.futopt().intraday().quote().symbol(&symbol);
        if after_hours {
            builder = builder.after_hours();
        }
        let result = builder.send()?;
        Ok(result.into())
    }

    /// Get ticker info for a contract (sync/blocking)
    pub fn ticker_sync(&self, symbol: String, after_hours: bool) -> Result<FutOptTicker, MarketDataError> {
        let mut builder = self.inner.futopt().intraday().ticker().symbol(&symbol);
        if after_hours {
            builder = builder.after_hours();
        }
        let result = builder.send()?;
        Ok(result.into())
    }

    /// Get available products list (sync/blocking)
    pub fn products_sync(&self, typ: String) -> Result<ProductsResponse, MarketDataError> {
        let futopt_type = parse_futopt_type(&typ)?;
        let result = self.inner.futopt().intraday().products().typ(futopt_type).send()?;
        Ok(result.into())
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Parse FutOpt type from string
/// "F" or "futures" -> FutOptType::Future
/// "O" or "options" -> FutOptType::Option
fn parse_futopt_type(typ: &str) -> Result<marketdata_core::FutOptType, MarketDataError> {
    use marketdata_core::FutOptType;
    match typ.to_uppercase().as_str() {
        "F" | "FUTURE" | "FUTURES" => Ok(FutOptType::Future),
        "O" | "OPTION" | "OPTIONS" => Ok(FutOptType::Option),
        _ => Err(MarketDataError::ConfigError {
            msg: format!("Invalid FutOpt type: '{}'. Use 'F' for futures or 'O' for options.", typ)
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rest_client_creation() {
        let client = RestClient::new(Auth::SdkToken("test-token".to_string()));
        let _ = client.stock();
        let _ = client.futopt();
    }

    #[test]
    fn test_stock_client_chain() {
        let client = RestClient::new(Auth::SdkToken("test-token".to_string()));
        let stock = client.stock();
        let _intraday = stock.intraday();
    }

    #[test]
    fn test_futopt_client_chain() {
        let client = RestClient::new(Auth::SdkToken("test-token".to_string()));
        let futopt = client.futopt();
        let _intraday = futopt.intraday();
    }

    #[test]
    fn test_parse_futopt_type() {
        assert!(matches!(parse_futopt_type("F"), Ok(marketdata_core::FutOptType::Future)));
        assert!(matches!(parse_futopt_type("O"), Ok(marketdata_core::FutOptType::Option)));
        assert!(matches!(parse_futopt_type("future"), Ok(marketdata_core::FutOptType::Future)));
        assert!(matches!(parse_futopt_type("options"), Ok(marketdata_core::FutOptType::Option)));
        assert!(parse_futopt_type("invalid").is_err());
    }
}
