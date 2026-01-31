---
phase: 07-complete-rest-api-coverage
plan: 01
subsystem: rest-api
tags: [historical, candles, stats, rust-core]
execution:
  duration: 6m
  completed: 2026-01-31
dependency-graph:
  requires: []
  provides: [HistoricalCandlesRequestBuilder, StatsRequestBuilder, StatsResponse]
  affects: [07-05 UniFFI exports, 07-06/07/08 language bindings]
tech-stack:
  added: []
  patterns: [builder-pattern, serde-deserialization]
key-files:
  created:
    - core/src/rest/stock/historical/mod.rs
    - core/src/rest/stock/historical/candles.rs
    - core/src/rest/stock/historical/stats.rs
    - core/src/models/historical.rs
  modified:
    - core/src/rest/stock/mod.rs
    - core/src/rest/client.rs
    - core/src/models/mod.rs
decisions:
  - id: D-07-01-1
    choice: "Reuse existing HistoricalCandle and HistoricalCandlesResponse from candle.rs"
    reason: "Existing types in candle.rs already match official SDK structure; no duplication needed"
  - id: D-07-01-2
    choice: "StatsResponse model with all serde renames for camelCase API fields"
    reason: "Official Node.js SDK shows 17 fields including week52High/Low with camelCase naming"
---

# Phase 7 Plan 1: Stock Historical REST Endpoints Summary

**One-liner:** Implemented historical candles and stats REST endpoints with typed request builders and StatsResponse model

## What Was Built

### Historical Candles Endpoint
- `HistoricalCandlesRequestBuilder` with 7 builder methods:
  - `symbol(str)` - Required stock symbol
  - `from(str)` - Start date (YYYY-MM-DD)
  - `to(str)` - End date (YYYY-MM-DD)
  - `timeframe(str)` - D/W/M/1/5/10/15/30/60
  - `fields(str)` - Field selection
  - `sort(str)` - asc/desc
  - `adjusted(bool)` - Split/dividend adjustment
- Endpoint: `GET /stock/historical/candles/{symbol}?{query_params}`

### Historical Stats Endpoint
- `StatsRequestBuilder` with symbol method
- Endpoint: `GET /stock/historical/stats/{symbol}`

### StatsResponse Model
17 fields matching official SDK:
- Basic: date, type, exchange, market, symbol, name
- Prices: openPrice, highPrice, lowPrice, closePrice, previousClose
- Changes: change, changePercent
- Volume: tradeVolume, tradeValue
- Range: week52High, week52Low

## Technical Details

### Response Model Attributes
```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
#[cfg_attr(feature = "js", napi_derive::napi(object))]
pub struct StatsResponse {
    #[serde(rename = "openPrice")]
    pub open_price: f64,
    // ... with all camelCase serde renames
}
```

### Client Access Pattern
```rust
let client = RestClient::new(Auth::SdkToken("token".to_string()));

// Historical candles
let candles = client.stock().historical().candles()
    .symbol("2330")
    .from("2024-01-01")
    .to("2024-01-31")
    .timeframe("D")
    .send()?;

// Historical stats
let stats = client.stock().historical().stats()
    .symbol("2330")
    .send()?;
```

## Deviations from Plan

### Auto-fixed Issues
None - plan executed as designed.

### Parallel Execution Note
This plan executed concurrently with plans 07-02 through 07-04. Changes were merged into a combined commit by the parallel execution infrastructure. The historical endpoints were committed as part of commit `cdbd2c1` alongside other plan implementations.

## Verification Results

| Check | Status | Details |
|-------|--------|---------|
| Directory structure | PASS | `core/src/rest/stock/historical/` with candles.rs, stats.rs, mod.rs |
| HistoricalCandlesRequestBuilder methods | PASS | All 7 builder methods implemented |
| StatsRequestBuilder methods | PASS | symbol() method implemented |
| serde renames | PASS | All camelCase API fields properly renamed |
| pyclass/napi attributes | PASS | Conditional compilation attributes present |
| cargo check | PASS | Compiles with all features |
| cargo test | PASS | 67 passed, 5 ignored |

## Next Steps

1. **07-05:** Export historical endpoints through UniFFI interface
2. **07-06/07/08:** Add historical methods to Python/Node.js/UniFFI language bindings
3. **Integration tests:** Add VCR cassettes for historical endpoint responses

## Commits

| Commit | Type | Message |
|--------|------|---------|
| cdbd2c1 | feat | Included in parallel execution commit with 07-02,03,04 changes |

Note: Due to parallel plan execution, historical endpoint changes were bundled with other endpoint implementations in a single commit.
