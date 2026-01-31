---
phase: 07
plan: 05
subsystem: rest-api
tags: [futopt, historical, candles, daily, rust-core]

dependency-graph:
  requires: [07-RESEARCH]
  provides: [FutOptHistoricalClient, FutOptHistoricalCandlesRequestBuilder, FutOptDailyRequestBuilder]
  affects: [07-06 through 07-09 (binding layer plans)]

tech-stack:
  added: []
  patterns: [builder-pattern, serde-rename, fluent-api]

key-files:
  created:
    - core/src/rest/futopt/historical/mod.rs
    - core/src/rest/futopt/historical/candles.rs
    - core/src/rest/futopt/historical/daily.rs
    - core/src/models/futopt/historical.rs
  modified:
    - core/src/rest/futopt/mod.rs
    - core/src/models/futopt/mod.rs

decisions:
  - id: 07-05-01
    choice: "Typed builder pattern with Option<> fields for optional parameters"
    rationale: "Follows established pattern from futopt/intraday and stock/historical endpoints"
  - id: 07-05-02
    choice: "Include FutOpt-specific fields (open_interest, settlement_price) in response models"
    rationale: "Futures/options have unique fields not present in stock historical data"

metrics:
  duration: 4 min
  completed: 2026-01-31
---

# Phase 7 Plan 5: FutOpt Historical REST Endpoints Summary

FutOpt Historical candles and daily endpoints implemented in Rust core with typed request builders and response models.

## One-liner

FutOpt Historical REST endpoints (candles/daily) with typed builders and FutOpt-specific fields (open_interest, settlement_price).

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Create FutOpt Historical module with candles endpoint | 39c17d1 | historical/mod.rs, candles.rs |
| 2 | Create FutOpt daily endpoint and response models | 39c17d1 | daily.rs, historical.rs, models/mod.rs |

## Artifacts Produced

### REST Endpoints
- `GET /futopt/historical/candles/{symbol}` - Historical OHLC candles
  - Parameters: from, to, timeframe, afterHours
- `GET /futopt/historical/daily/{symbol}` - Daily historical data
  - Parameters: from, to, afterHours

### Request Builders
- `FutOptHistoricalCandlesRequestBuilder` - symbol/from/to/timeframe/after_hours methods
- `FutOptDailyRequestBuilder` - symbol/from/to/after_hours methods

### Response Models
- `FutOptHistoricalCandlesResponse` - symbol, data_type, exchange, timeframe, candles
- `FutOptHistoricalCandle` - date, open/high/low/close, volume, open_interest, change, change_percent
- `FutOptDailyResponse` - symbol, data_type, exchange, data
- `FutOptDailyData` - date, open/high/low/close, volume, open_interest, settlement_price

### Client Integration
- `FutOptClient::historical()` - Returns `FutOptHistoricalClient`
- `FutOptHistoricalClient::candles()` - Returns builder
- `FutOptHistoricalClient::daily()` - Returns builder

## Decisions Made

1. **07-05-01: Builder pattern with optional fields**
   - Same pattern as existing intraday endpoints
   - `symbol` required (error if not provided)
   - `from`, `to`, `timeframe`, `after_hours` optional

2. **07-05-02: FutOpt-specific response fields**
   - `open_interest: Option<u64>` - Total outstanding contracts
   - `settlement_price: Option<f64>` - Official closing price for margin calculation
   - These distinguish FutOpt from Stock historical data

## Deviations from Plan

None - plan executed exactly as written.

## Verification Results

```
Success Criteria:
1. core/src/rest/futopt/historical/ exists with candles.rs and daily.rs
2. FutOptHistoricalCandlesRequestBuilder has all required methods
3. FutOptDailyRequestBuilder has all required methods
4. Response models include open_interest and settlement_price
5. cargo check -p marketdata-core passes

All 5 criteria verified.
```

## Test Coverage

- 4 unit tests in candles.rs (builder validation, params, timeframes, chaining)
- 3 unit tests in daily.rs (builder validation, params, chaining)
- 6 unit tests in historical.rs (response deserialization, helper methods, minimal cases)

## Dependencies

- No new crate dependencies added
- Uses existing: serde, serde_json, ureq

## Next Phase Readiness

Ready for:
- 07-06: Python binding propagation via PyO3
- 07-07: Node.js binding propagation via napi-rs
- 07-08: UniFFI binding propagation for C#/Java/Go
