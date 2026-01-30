//! REST client wrapper for JavaScript
//!
//! This module provides the JavaScript-facing RestClient that wraps
//! marketdata-core::RestClient for NAPI-RS bindings.

use crate::errors::to_napi_error;
use napi_derive::napi;
use serde_json::Value;

/// REST client for Fugle market data API (JavaScript wrapper)
///
/// # JavaScript Usage
///
/// ```javascript
/// const { RestClient } = require('@fubon/marketdata-js');
///
/// // Create client with API key
/// const client = new RestClient('your-api-key');
///
/// // Access stock market data
/// const quote = client.stock.intraday.quote('2330');
/// console.log(quote.lastPrice, quote.symbol);
///
/// // Access futures/options market data
/// const futoptQuote = client.futopt.intraday.quote('TXFC4');
/// console.log(futoptQuote.lastPrice, futoptQuote.symbol);
/// ```
#[napi]
pub struct RestClient {
    inner: marketdata_core::RestClient,
}

#[napi]
impl RestClient {
    /// Create a new REST client with API key authentication
    ///
    /// @param apiKey - Your Fugle API key
    #[napi(constructor)]
    pub fn new(api_key: String) -> Self {
        let auth = marketdata_core::rest::Auth::ApiKey(api_key);
        Self {
            inner: marketdata_core::RestClient::new(auth),
        }
    }

    /// Get the stock client for accessing stock market data
    #[napi(getter)]
    pub fn stock(&self) -> StockClient {
        StockClient {
            inner: self.inner.clone(),
        }
    }

    /// Get the FutOpt client for accessing futures/options market data
    #[napi(getter)]
    pub fn futopt(&self) -> FutOptClient {
        FutOptClient {
            inner: self.inner.clone(),
        }
    }
}

/// Stock market data client
#[napi]
pub struct StockClient {
    inner: marketdata_core::RestClient,
}

#[napi]
impl StockClient {
    /// Get intraday client for real-time stock data
    #[napi(getter)]
    pub fn intraday(&self) -> StockIntradayClient {
        StockIntradayClient {
            inner: self.inner.clone(),
        }
    }
}

/// Stock intraday data client
#[napi]
pub struct StockIntradayClient {
    inner: marketdata_core::RestClient,
}

#[napi]
impl StockIntradayClient {
    /// Get intraday quote for a stock symbol
    ///
    /// @param symbol - Stock symbol (e.g., "2330" for TSMC)
    /// @returns Promise resolving to Quote object with current price and volume data
    ///
    /// @example
    /// ```javascript
    /// const client = new RestClient('your-api-key');
    /// const quote = await client.stock.intraday.quote('2330');
    /// console.log(quote.lastPrice);  // 580.0
    /// console.log(quote.symbol);     // "2330"
    /// console.log(quote.bids);       // [{price: 579.0, size: 100}, ...]
    /// ```
    #[napi(ts_return_type = "Promise<QuoteResponse>")]
    pub async fn quote(&self, symbol: String) -> napi::Result<Value> {
        let inner = self.inner.clone();

        // Use spawn_blocking since core uses synchronous HTTP (ureq)
        let result = tokio::task::spawn_blocking(move || {
            inner.stock().intraday().quote().symbol(&symbol).send()
        })
        .await
        .map_err(|e| napi::Error::from_reason(format!("Task error: {}", e)))?;

        match result {
            Ok(quote) => serde_json::to_value(&quote)
                .map_err(|e| napi::Error::from_reason(e.to_string())),
            Err(e) => Err(to_napi_error(e)),
        }
    }

    /// Get intraday ticker for a stock symbol
    ///
    /// @param symbol - Stock symbol (e.g., "2330" for TSMC)
    /// @returns Promise resolving to Ticker object with last trade info
    #[napi(ts_return_type = "Promise<TickerResponse>")]
    pub async fn ticker(&self, symbol: String) -> napi::Result<Value> {
        let inner = self.inner.clone();

        let result = tokio::task::spawn_blocking(move || {
            inner.stock().intraday().ticker().symbol(&symbol).send()
        })
        .await
        .map_err(|e| napi::Error::from_reason(format!("Task error: {}", e)))?;

        match result {
            Ok(ticker) => serde_json::to_value(&ticker)
                .map_err(|e| napi::Error::from_reason(e.to_string())),
            Err(e) => Err(to_napi_error(e)),
        }
    }

    /// Get intraday candles for a stock symbol
    ///
    /// @param symbol - Stock symbol (e.g., "2330" for TSMC)
    /// @param timeframe - Candle timeframe: "1", "5", "10", "15", "30", "60" (minutes)
    /// @returns Promise resolving to Candles response with OHLCV data
    #[napi(ts_return_type = "Promise<CandlesResponse>")]
    pub async fn candles(&self, symbol: String, timeframe: String) -> napi::Result<Value> {
        let inner = self.inner.clone();

        let result = tokio::task::spawn_blocking(move || {
            inner
                .stock()
                .intraday()
                .candles()
                .symbol(&symbol)
                .timeframe(&timeframe)
                .send()
        })
        .await
        .map_err(|e| napi::Error::from_reason(format!("Task error: {}", e)))?;

        match result {
            Ok(candles) => serde_json::to_value(&candles)
                .map_err(|e| napi::Error::from_reason(e.to_string())),
            Err(e) => Err(to_napi_error(e)),
        }
    }

    /// Get intraday trades for a stock symbol
    ///
    /// @param symbol - Stock symbol (e.g., "2330" for TSMC)
    /// @returns Promise resolving to Trades response with recent trade history
    #[napi(ts_return_type = "Promise<TradesResponse>")]
    pub async fn trades(&self, symbol: String) -> napi::Result<Value> {
        let inner = self.inner.clone();

        let result = tokio::task::spawn_blocking(move || {
            inner.stock().intraday().trades().symbol(&symbol).send()
        })
        .await
        .map_err(|e| napi::Error::from_reason(format!("Task error: {}", e)))?;

        match result {
            Ok(trades) => serde_json::to_value(&trades)
                .map_err(|e| napi::Error::from_reason(e.to_string())),
            Err(e) => Err(to_napi_error(e)),
        }
    }

    /// Get intraday volumes for a stock symbol
    ///
    /// @param symbol - Stock symbol (e.g., "2330" for TSMC)
    /// @returns Promise resolving to Volumes response with volume at each price level
    #[napi(ts_return_type = "Promise<VolumesResponse>")]
    pub async fn volumes(&self, symbol: String) -> napi::Result<Value> {
        let inner = self.inner.clone();

        let result = tokio::task::spawn_blocking(move || {
            inner.stock().intraday().volumes().symbol(&symbol).send()
        })
        .await
        .map_err(|e| napi::Error::from_reason(format!("Task error: {}", e)))?;

        match result {
            Ok(volumes) => serde_json::to_value(&volumes)
                .map_err(|e| napi::Error::from_reason(e.to_string())),
            Err(e) => Err(to_napi_error(e)),
        }
    }
}

/// Futures and Options market data client
#[napi]
pub struct FutOptClient {
    inner: marketdata_core::RestClient,
}

#[napi]
impl FutOptClient {
    /// Get intraday client for real-time futures/options data
    #[napi(getter)]
    pub fn intraday(&self) -> FutOptIntradayClient {
        FutOptIntradayClient {
            inner: self.inner.clone(),
        }
    }
}

/// FutOpt intraday data client
#[napi]
pub struct FutOptIntradayClient {
    inner: marketdata_core::RestClient,
}

#[napi]
impl FutOptIntradayClient {
    /// Get intraday quote for a futures/options contract
    ///
    /// @param symbol - Contract symbol (e.g., "TXFC4" for TX futures, "TXO18000C4" for options)
    /// @returns Promise resolving to Quote object with current price and volume data
    ///
    /// @example
    /// ```javascript
    /// const client = new RestClient('your-api-key');
    /// const quote = await client.futopt.intraday.quote('TXFC4');
    /// console.log(quote.lastPrice);  // 17550.0
    /// console.log(quote.symbol);     // "TXFC4"
    /// ```
    #[napi(ts_return_type = "Promise<QuoteResponse>")]
    pub async fn quote(&self, symbol: String) -> napi::Result<Value> {
        let inner = self.inner.clone();

        let result = tokio::task::spawn_blocking(move || {
            inner.futopt().intraday().quote().symbol(&symbol).send()
        })
        .await
        .map_err(|e| napi::Error::from_reason(format!("Task error: {}", e)))?;

        match result {
            Ok(quote) => serde_json::to_value(&quote)
                .map_err(|e| napi::Error::from_reason(e.to_string())),
            Err(e) => Err(to_napi_error(e)),
        }
    }

    /// Get intraday ticker for a futures/options contract
    ///
    /// @param symbol - Contract symbol (e.g., "TXFC4")
    /// @returns Promise resolving to Ticker object with last trade info
    #[napi(ts_return_type = "Promise<TickerResponse>")]
    pub async fn ticker(&self, symbol: String) -> napi::Result<Value> {
        let inner = self.inner.clone();

        let result = tokio::task::spawn_blocking(move || {
            inner.futopt().intraday().ticker().symbol(&symbol).send()
        })
        .await
        .map_err(|e| napi::Error::from_reason(format!("Task error: {}", e)))?;

        match result {
            Ok(ticker) => serde_json::to_value(&ticker)
                .map_err(|e| napi::Error::from_reason(e.to_string())),
            Err(e) => Err(to_napi_error(e)),
        }
    }

    /// Get intraday candles for a futures/options contract
    ///
    /// @param symbol - Contract symbol (e.g., "TXFC4")
    /// @param timeframe - Candle timeframe: "1", "5", "10", "15", "30", "60" (minutes)
    /// @returns Promise resolving to Candles response with OHLCV data
    #[napi(ts_return_type = "Promise<CandlesResponse>")]
    pub async fn candles(&self, symbol: String, timeframe: String) -> napi::Result<Value> {
        let inner = self.inner.clone();

        let result = tokio::task::spawn_blocking(move || {
            inner
                .futopt()
                .intraday()
                .candles()
                .symbol(&symbol)
                .timeframe(&timeframe)
                .send()
        })
        .await
        .map_err(|e| napi::Error::from_reason(format!("Task error: {}", e)))?;

        match result {
            Ok(candles) => serde_json::to_value(&candles)
                .map_err(|e| napi::Error::from_reason(e.to_string())),
            Err(e) => Err(to_napi_error(e)),
        }
    }

    /// Get intraday trades for a futures/options contract
    ///
    /// @param symbol - Contract symbol (e.g., "TXFC4")
    /// @returns Promise resolving to Trades response with recent trade history
    #[napi(ts_return_type = "Promise<TradesResponse>")]
    pub async fn trades(&self, symbol: String) -> napi::Result<Value> {
        let inner = self.inner.clone();

        let result = tokio::task::spawn_blocking(move || {
            inner.futopt().intraday().trades().symbol(&symbol).send()
        })
        .await
        .map_err(|e| napi::Error::from_reason(format!("Task error: {}", e)))?;

        match result {
            Ok(trades) => serde_json::to_value(&trades)
                .map_err(|e| napi::Error::from_reason(e.to_string())),
            Err(e) => Err(to_napi_error(e)),
        }
    }

    /// Get intraday volumes for a futures/options contract
    ///
    /// @param symbol - Contract symbol (e.g., "TXFC4")
    /// @returns Promise resolving to Volumes response with volume at each price level
    #[napi(ts_return_type = "Promise<VolumesResponse>")]
    pub async fn volumes(&self, symbol: String) -> napi::Result<Value> {
        let inner = self.inner.clone();

        let result = tokio::task::spawn_blocking(move || {
            inner.futopt().intraday().volumes().symbol(&symbol).send()
        })
        .await
        .map_err(|e| napi::Error::from_reason(format!("Task error: {}", e)))?;

        match result {
            Ok(volumes) => serde_json::to_value(&volumes)
                .map_err(|e| napi::Error::from_reason(e.to_string())),
            Err(e) => Err(to_napi_error(e)),
        }
    }

    /// Get product list for futures/options
    ///
    /// @param typ - Type: "FUTURE" or "OPTION" (required)
    /// @param contractType - Contract type filter (optional): "I" (index), "R" (rate), "B" (bond), "C" (currency), "S" (stock), "E" (ETF)
    /// @returns Promise resolving to Products response with available contracts
    #[napi(ts_return_type = "Promise<ProductsResponse>", ts_args_type = "type: FutOptType, contractType?: ContractType")]
    pub async fn products(&self, typ: String, contract_type: Option<String>) -> napi::Result<Value> {
        use marketdata_core::models::futopt::{ContractType, FutOptType};

        // Parse typ parameter before spawn_blocking
        let fut_opt_type = match typ.to_uppercase().as_str() {
            "FUTURE" => FutOptType::Future,
            "OPTION" => FutOptType::Option,
            _ => {
                return Err(napi::Error::from_reason(format!(
                    "Invalid type '{}': must be 'FUTURE' or 'OPTION'",
                    typ
                )))
            }
        };

        // Parse optional contract type
        let ct_enum = if let Some(ct) = contract_type {
            Some(match ct.to_uppercase().as_str() {
                "I" | "INDEX" => ContractType::Index,
                "R" | "RATE" => ContractType::Rate,
                "B" | "BOND" => ContractType::Bond,
                "C" | "CURRENCY" => ContractType::Currency,
                "S" | "STOCK" => ContractType::Stock,
                "E" | "ETF" => ContractType::Etf,
                _ => {
                    return Err(napi::Error::from_reason(format!(
                        "Invalid contractType '{}': must be I/R/B/C/S/E",
                        ct
                    )))
                }
            })
        } else {
            None
        };

        let inner = self.inner.clone();

        let result = tokio::task::spawn_blocking(move || {
            let mut builder = inner.futopt().intraday().products().typ(fut_opt_type);
            if let Some(ct) = ct_enum {
                builder = builder.contract_type(ct);
            }
            builder.send()
        })
        .await
        .map_err(|e| napi::Error::from_reason(format!("Task error: {}", e)))?;

        match result {
            Ok(products) => serde_json::to_value(&products)
                .map_err(|e| napi::Error::from_reason(e.to_string())),
            Err(e) => Err(to_napi_error(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rest_client_creation() {
        let client = RestClient::new("test-api-key".to_string());
        // Verify client was created (compilation success is the test)
        let _ = client.stock();
        let _ = client.futopt();
    }

    #[test]
    fn test_stock_client_chain() {
        let client = RestClient::new("test-api-key".to_string());
        let stock = client.stock();
        let _intraday = stock.intraday();
    }

    #[test]
    fn test_futopt_client_chain() {
        let client = RestClient::new("test-api-key".to_string());
        let futopt = client.futopt();
        let _intraday = futopt.intraday();
    }
}
