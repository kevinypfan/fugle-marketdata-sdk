# Phase 7 Plan 06: Propagate to UniFFI Bindings Summary

## Overview

**One-liner:** UniFFI client types and regenerated language bindings for C#, Java, Go exposing new REST endpoints

**Duration:** ~15 minutes (session continuation after context reset)
**Completed:** 2026-01-31

## What Was Built

### UniFFI Client Types (uniffi/src/client.rs)

Added 5 new client structs with async (get_*) and sync (*_sync) method variants:

**StockHistoricalClient:**
- `get_candles(symbol, from?, to?, timeframe?)` / `candles_sync(...)`
- `get_stats(symbol, type_?)` / `stats_sync(...)`

**StockSnapshotClient:**
- `get_quotes(market, type_filter?)` / `quotes_sync(...)`
- `get_movers(market, direction?, change?)` / `movers_sync(...)`
- `get_actives(market, trade?)` / `actives_sync(...)`

**StockTechnicalClient:**
- `get_sma(symbol, period?)` / `sma_sync(...)`
- `get_rsi(symbol, period?)` / `rsi_sync(...)`
- `get_kdj(symbol, period?)` / `kdj_sync(...)`
- `get_macd(symbol, period?)` / `macd_sync(...)`
- `get_bb(symbol, period?)` / `bb_sync(...)`

**StockCorporateActionsClient:**
- `get_capital_changes(symbol, start_date?, end_date?)` / `capital_changes_sync(...)`
- `get_dividends(symbol, start_date?, end_date?)` / `dividends_sync(...)`
- `get_listing_applicants(symbol)` / `listing_applicants_sync(...)`

**FutOptHistoricalClient:**
- `get_candles(symbol, from?, to?, timeframe?, after_hours?)` / `candles_sync(...)`
- `get_daily(symbol, from?, to?, sort?, after_hours?)` / `daily_sync(...)`

### UniFFI Model Types (uniffi/src/models.rs)

Added 50+ new record types with `uniffi::Record` derive and `From<CoreType>` implementations:
- Historical: StatsResponse, HistoricalCandlesResponse
- Snapshot: SnapshotQuotesResponse, SnapshotQuote, MoversResponse, Mover, ActivesResponse, Active
- Technical: SmaResponse, SmaDataPoint, RsiResponse, RsiDataPoint, KdjResponse, KdjDataPoint, MacdResponse, MacdDataPoint, BbResponse, BbDataPoint
- Corporate: CapitalChangesResponse, CapitalChange, DividendsResponse, Dividend, ListingApplicantsResponse, ListingApplicant
- FutOpt: FutOptHistoricalCandlesResponse, FutOptHistoricalCandle, FutOptDailyResponse, FutOptDailyData

### Regenerated Language Bindings

**C# (bindings/csharp/MarketdataUniffi/marketdata_uniffi.cs):**
- Updated with all new types as public records
- Fixed internal → public accessibility for model types
- Build verified: `dotnet build` passes
- Tests verified: 45 tests pass

**Go (bindings/go/marketdata/):**
- Updated marketdata_uniffi.go with all new client types
- Updated marketdata_uniffi.h header
- Build verified: `go build` passes

**Java (bindings/java/src/main/java/.../generated/):**
- Added 80+ new generated files
- Requires JDK 21 (uniffi-bindgen-java generates pattern matching in switch)
- All model types and client interfaces generated

## Files Created/Modified

### Created
- 80+ new Java generated files in bindings/java/src/main/java/.../generated/

### Modified
- `uniffi/src/client.rs` - Added 5 new client types with 20+ methods
- `uniffi/src/models.rs` - Added 50+ new model types
- `uniffi/src/errors.rs` - Added InvalidParameter error handling
- `bindings/csharp/MarketdataUniffi/marketdata_uniffi.cs` - Regenerated
- `bindings/go/marketdata/marketdata_uniffi.go` - Regenerated
- `bindings/go/marketdata/marketdata_uniffi.h` - Regenerated
- `bindings/java/src/main/java/.../generated/*.java` - 10 modified files

## Verification Results

- `cargo build -p marketdata-uniffi --release` - Passed
- `cargo test -p marketdata-uniffi` - 14/14 tests passed
- `dotnet build` (C#) - Passed
- `go build` (Go) - Passed
- Java - Requires JDK 21 (pattern matching syntax)

## Commits

- `2b77f69` - feat(07-06): add new REST endpoint client types to UniFFI
- `00cfc17` - feat(07-06): regenerate language bindings with new REST endpoints

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Borrow checker issues with builder pattern**
- **Found during:** Task 1
- **Issue:** Core library uses reference-based builders with lifetimes that caused borrow checker errors
- **Fix:** Created helper functions with intermediate variable bindings to extend lifetimes
- **Files modified:** `uniffi/src/client.rs`

**2. [Rule 3 - Blocking] InvalidParameter error variant not handled**
- **Found during:** Task 2
- **Issue:** Core library added InvalidParameter error in Phase 7, not handled in UniFFI error mapping
- **Fix:** Added match arm for InvalidParameter → ApiError conversion
- **Files modified:** `uniffi/src/errors.rs`

**3. [Rule 1 - Bug] C# generated types marked as internal**
- **Found during:** Task 3
- **Issue:** uniffi-bindgen-cs generates record types as `internal` by default
- **Fix:** Post-processed generated file to change `internal record` to `public record`
- **Files modified:** `bindings/csharp/MarketdataUniffi/marketdata_uniffi.cs`

## Next Phase Readiness

**UniFFI bindings complete and ready for:**
- Python binding wrappers (07-07)
- Node.js binding wrappers (07-08)
- Language-specific wrapper classes can now access all new endpoints

**Dependencies satisfied:**
- All new REST endpoint methods exported via UniFFI
- All response models have proper type conversions
- Error handling covers all core error variants

**Known limitations:**
- Java bindings require JDK 21 (uniffi-bindgen-java uses pattern matching)
- Wrapper files (FugleRestClient.cs, FugleRestClient.java) not updated with convenience methods
