//! Type conversions between marketdata-core types and Python dicts
//!
//! All marketdata-core types are converted to Python dictionaries to allow
//! users to access fields directly without needing separate Python classes
//! for each data model.

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use pyo3::IntoPyObjectExt;
use serde_json::Value;

/// Convert a Quote to a Python dict
pub fn quote_to_dict(py: Python<'_>, quote: &marketdata_core::Quote) -> PyResult<Py<PyDict>> {
    let dict = PyDict::new(py);

    dict.set_item("date", &quote.date)?;
    dict.set_item("symbol", &quote.symbol)?;
    dict.set_item("type", &quote.data_type)?;
    dict.set_item("exchange", &quote.exchange)?;
    dict.set_item("market", &quote.market)?;
    dict.set_item("name", &quote.name)?;

    // OHLC prices
    dict.set_item("openPrice", quote.open_price)?;
    dict.set_item("openTime", quote.open_time)?;
    dict.set_item("highPrice", quote.high_price)?;
    dict.set_item("highTime", quote.high_time)?;
    dict.set_item("lowPrice", quote.low_price)?;
    dict.set_item("lowTime", quote.low_time)?;
    dict.set_item("closePrice", quote.close_price)?;
    dict.set_item("closeTime", quote.close_time)?;

    // Current trading info
    dict.set_item("lastPrice", quote.last_price)?;
    dict.set_item("lastSize", quote.last_size)?;
    dict.set_item("avgPrice", quote.avg_price)?;
    dict.set_item("change", quote.change)?;
    dict.set_item("changePercent", quote.change_percent)?;
    dict.set_item("amplitude", quote.amplitude)?;

    // Order book
    let bids = PyList::empty(py);
    for level in &quote.bids {
        let level_dict = PyDict::new(py);
        level_dict.set_item("price", level.price)?;
        level_dict.set_item("size", level.size)?;
        bids.append(level_dict)?;
    }
    dict.set_item("bids", bids)?;

    let asks = PyList::empty(py);
    for level in &quote.asks {
        let level_dict = PyDict::new(py);
        level_dict.set_item("price", level.price)?;
        level_dict.set_item("size", level.size)?;
        asks.append(level_dict)?;
    }
    dict.set_item("asks", asks)?;

    // Total stats
    if let Some(ref total) = quote.total {
        let total_dict = PyDict::new(py);
        total_dict.set_item("tradeValue", total.trade_value)?;
        total_dict.set_item("tradeVolume", total.trade_volume)?;
        total_dict.set_item("transaction", total.transaction)?;
        dict.set_item("total", total_dict)?;
    } else {
        dict.set_item("total", py.None())?;
    }

    // Flags
    dict.set_item("isLimitUpPrice", quote.is_limit_up_price)?;
    dict.set_item("isLimitDownPrice", quote.is_limit_down_price)?;
    dict.set_item("isTrial", quote.is_trial)?;
    dict.set_item("isOpen", quote.is_open)?;
    dict.set_item("isClose", quote.is_close)?;
    dict.set_item("lastUpdated", quote.last_updated)?;

    Ok(dict.into())
}

/// Convert a FutOptQuote to a Python dict
pub fn futopt_quote_to_dict(
    py: Python<'_>,
    quote: &marketdata_core::FutOptQuote,
) -> PyResult<Py<PyDict>> {
    let dict = PyDict::new(py);

    dict.set_item("date", &quote.date)?;
    dict.set_item("symbol", &quote.symbol)?;
    dict.set_item("type", &quote.contract_type)?;
    dict.set_item("exchange", &quote.exchange)?;
    dict.set_item("name", &quote.name)?;
    dict.set_item("previousClose", quote.previous_close)?;

    // OHLC prices
    dict.set_item("openPrice", quote.open_price)?;
    dict.set_item("openTime", quote.open_time)?;
    dict.set_item("highPrice", quote.high_price)?;
    dict.set_item("highTime", quote.high_time)?;
    dict.set_item("lowPrice", quote.low_price)?;
    dict.set_item("lowTime", quote.low_time)?;
    dict.set_item("closePrice", quote.close_price)?;
    dict.set_item("closeTime", quote.close_time)?;

    // Current trading info
    dict.set_item("lastPrice", quote.last_price)?;
    dict.set_item("lastSize", quote.last_size)?;
    dict.set_item("avgPrice", quote.avg_price)?;
    dict.set_item("change", quote.change)?;
    dict.set_item("changePercent", quote.change_percent)?;
    dict.set_item("amplitude", quote.amplitude)?;

    // Order book
    let bids = PyList::empty(py);
    for level in &quote.bids {
        let level_dict = PyDict::new(py);
        level_dict.set_item("price", level.price)?;
        level_dict.set_item("size", level.size)?;
        bids.append(level_dict)?;
    }
    dict.set_item("bids", bids)?;

    let asks = PyList::empty(py);
    for level in &quote.asks {
        let level_dict = PyDict::new(py);
        level_dict.set_item("price", level.price)?;
        level_dict.set_item("size", level.size)?;
        asks.append(level_dict)?;
    }
    dict.set_item("asks", asks)?;

    // Total stats
    if let Some(ref total) = quote.total {
        let total_dict = PyDict::new(py);
        total_dict.set_item("tradeVolume", total.trade_volume)?;
        total_dict.set_item("totalBidMatch", total.total_bid_match)?;
        total_dict.set_item("totalAskMatch", total.total_ask_match)?;
        dict.set_item("total", total_dict)?;
    } else {
        dict.set_item("total", py.None())?;
    }

    // Last trade
    if let Some(ref last_trade) = quote.last_trade {
        let trade_dict = PyDict::new(py);
        trade_dict.set_item("price", last_trade.price)?;
        trade_dict.set_item("size", last_trade.size)?;
        trade_dict.set_item("time", last_trade.time)?;
        dict.set_item("lastTrade", trade_dict)?;
    } else {
        dict.set_item("lastTrade", py.None())?;
    }

    dict.set_item("lastUpdated", quote.last_updated)?;

    Ok(dict.into())
}

/// Convert a serde_json::Value to a Python object
///
/// Used for generic API responses that return JSON data
fn json_value_to_py<'py>(py: Python<'py>, value: &Value) -> PyResult<Bound<'py, PyAny>> {
    match value {
        Value::Null => Ok(py.None().into_bound(py)),
        Value::Bool(b) => Ok(b.into_bound_py_any(py)?),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(i.into_bound_py_any(py)?)
            } else if let Some(f) = n.as_f64() {
                Ok(f.into_bound_py_any(py)?)
            } else {
                Ok(n.to_string().into_bound_py_any(py)?)
            }
        }
        Value::String(s) => Ok(s.into_bound_py_any(py)?),
        Value::Array(arr) => {
            let list = PyList::empty(py);
            for item in arr {
                list.append(json_value_to_py(py, item)?)?;
            }
            Ok(list.into_any())
        }
        Value::Object(obj) => {
            let dict = PyDict::new(py);
            for (key, val) in obj {
                dict.set_item(key, json_value_to_py(py, val)?)?;
            }
            Ok(dict.into_any())
        }
    }
}

/// Convert a Ticker to a Python dict
pub fn ticker_to_dict(py: Python<'_>, ticker: &marketdata_core::Ticker) -> PyResult<Py<PyDict>> {
    let json_val = serde_json::to_value(ticker)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Serialization error: {}", e)))?;
    let bound_any = json_value_to_py(py, &json_val)?;
    Ok(bound_any.unbind().cast::<PyDict>(py)?)
}

/// Convert IntradayCandlesResponse to a Python dict
pub fn candles_to_dict(py: Python<'_>, candles: &marketdata_core::IntradayCandlesResponse) -> PyResult<Py<PyDict>> {
    let json_val = serde_json::to_value(candles)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Serialization error: {}", e)))?;
    let bound_any = json_value_to_py(py, &json_val)?;
    Ok(bound_any.unbind().cast::<PyDict>(py)?)
}

/// Convert TradesResponse to a Python dict
pub fn trades_to_dict(py: Python<'_>, trades: &marketdata_core::TradesResponse) -> PyResult<Py<PyDict>> {
    let json_val = serde_json::to_value(trades)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Serialization error: {}", e)))?;
    let bound_any = json_value_to_py(py, &json_val)?;
    Ok(bound_any.unbind().cast::<PyDict>(py)?)
}

/// Convert VolumesResponse to a Python dict
pub fn volumes_to_dict(py: Python<'_>, volumes: &marketdata_core::VolumesResponse) -> PyResult<Py<PyDict>> {
    let json_val = serde_json::to_value(volumes)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Serialization error: {}", e)))?;
    let bound_any = json_value_to_py(py, &json_val)?;
    Ok(bound_any.unbind().cast::<PyDict>(py)?)
}

// Note: PyO3 tests require Python linking and are tested via maturin develop + pytest
// Unit tests for dict conversion logic will be validated through Python integration tests
