//! Type conversions between marketdata-core types and Python dicts
//!
//! All marketdata-core types are converted to Python dictionaries to allow
//! users to access fields directly without needing separate Python classes
//! for each data model.

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

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

// Note: PyO3 tests require Python linking and are tested via maturin develop + pytest
// Unit tests for dict conversion logic will be validated through Python integration tests
