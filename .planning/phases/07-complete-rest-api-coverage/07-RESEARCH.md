# Phase 7: Complete REST API Coverage - Research

**Researched:** 2026-01-31
**Domain:** REST API implementation and multi-language FFI bindings
**Confidence:** HIGH

## Summary

Phase 7 completes the SDK's REST API coverage by implementing 17 missing endpoints across Stock and FutOpt categories. The research verified exact endpoint paths, response structures, and parameter signatures from official Python and Node.js SDKs.

The standard approach follows the established pattern: implement typed endpoints in Rust core (`core/src/rest/`), create typed models (`core/src/models/`), then propagate through all 5 language bindings (Python via PyO3, Node.js via napi-rs, C#/Java/Go via UniFFI).

**Primary recommendation:** Implement endpoints in logical groups (Historical → Snapshot → Technical → Corporate Actions), with each group completing the full stack (Rust core → models → all bindings) before moving to the next category. This ensures incremental testability and maintains API consistency.

## Standard Stack

The established libraries and patterns for this domain:

### Core (Rust)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| ureq | 2.x | HTTP client | Blocking HTTP for REST (already established) |
| serde | 1.0 | Serialization | JSON request/response handling |
| serde_json | 1.0 | JSON parsing | Type-safe deserialization to models |
| tokio | 1.x | Async runtime | spawn_blocking for non-blocking FFI calls |

### Bindings Layer
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| PyO3 | 0.27 | Python FFI | Python bindings (spawn_blocking pattern) |
| napi-rs | 3.4 | Node.js FFI | Node.js bindings (Promise-returning async) |
| UniFFI | 0.28 | Multi-language FFI | C#, Java, Go bindings (proc-macro typed interface) |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| ureq (blocking) | reqwest (async) | reqwest requires tokio in core; ureq simpler for FFI boundaries |
| UniFFI proc-macro | UDL file | Proc-macro avoids duplicate type generation |
| Separate endpoints | Generic endpoint | Typed builders provide better IDE support and compile-time safety |

**Installation:**
```bash
# Already in workspace, no new dependencies required
cargo build --workspace
```

## Architecture Patterns

### Recommended Endpoint Structure
```
core/src/
├── rest/
│   └── stock/
│       ├── intraday/       # ✓ Complete (quote, ticker, candles, trades, volumes)
│       ├── historical/     # → ADD (candles, stats)
│       ├── snapshot/       # → ADD (quotes, movers, actives)
│       ├── technical/      # → ADD (sma, rsi, kdj, macd, bb)
│       └── corporate_actions/ # → ADD (capital_changes, dividends, listing_applicants)
├── models/
│   ├── quote.rs           # ✓ Existing model
│   ├── candle.rs          # → ADD HistoricalCandle, Stats
│   ├── snapshot.rs        # → ADD SnapshotQuote, Movers, Actives
│   ├── technical.rs       # → ADD SmaData, RsiData, KdjData, MacdData, BbData
│   └── corporate.rs       # → ADD CapitalChange, Dividend, ListingApplicant
```

### Pattern 1: Request Builder Pattern (Existing)
**What:** Fluent builder API for constructing REST requests
**When to use:** All REST endpoints with optional parameters
**Example:**
```rust
// Source: core/src/rest/stock/intraday/quote.rs (verified existing pattern)
pub struct QuoteRequestBuilder<'a> {
    client: &'a RestClient,
    symbol: Option<String>,
    odd_lot: Option<bool>,
}

impl<'a> QuoteRequestBuilder<'a> {
    pub fn symbol(mut self, symbol: &str) -> Self {
        self.symbol = Some(symbol.to_string());
        self
    }

    pub fn send(self) -> Result<Quote, MarketDataError> {
        // Build URL, make HTTP call, deserialize response
    }
}
```

### Pattern 2: Typed Model Deserialization
**What:** serde-powered JSON to strongly-typed Rust structs
**When to use:** All response types for compile-time safety
**Example:**
```rust
// Source: core/src/models/quote.rs (verified existing pattern)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
#[cfg_attr(feature = "js", napi_derive::napi(object))]
pub struct Quote {
    pub date: String,
    pub symbol: String,
    #[serde(rename = "lastPrice")]
    pub last_price: Option<f64>,
    // ... 40+ fields with proper serde attributes
}
```

### Pattern 3: FFI Async Bridging via spawn_blocking
**What:** Wrap blocking ureq calls in tokio::spawn_blocking for non-blocking FFI
**When to use:** All UniFFI async exports and PyO3/napi-rs async methods
**Example:**
```rust
// Source: uniffi/src/client.rs (verified existing pattern)
#[uniffi::export(async_runtime = "tokio")]
impl StockIntradayClient {
    pub async fn get_quote(&self, symbol: String) -> Result<Quote, MarketDataError> {
        let inner = self.inner.clone();
        let result = tokio::task::spawn_blocking(move || {
            inner.stock().intraday().quote().symbol(&symbol).send()
        })
        .await??;
        Ok(result.into())
    }
}
```

### Pattern 4: Language Binding Propagation
**What:** Expose Rust core through language-specific FFI layers
**When to use:** After implementing endpoints in core, propagate to all bindings
**Layers:**
1. **Rust core:** Implement endpoint + model
2. **PyO3 (Python):** Add async method with spawn_blocking wrapper
3. **napi-rs (Node.js):** Add async method returning Promise
4. **UniFFI:** Add to typed interface for C#/Java/Go generation
5. **Language wrappers:** Create idiomatic wrappers (FugleRestClient, etc.)

### Anti-Patterns to Avoid
- **JSON string returns:** UniFFI supports typed models; never return JSON strings
- **Sync-only APIs:** All bindings now expect async/await; provide sync wrappers via `.await`
- **Missing serde renames:** API uses camelCase; Rust uses snake_case; always use `#[serde(rename)]`
- **Direct HTTP in bindings:** Always call core; never duplicate HTTP logic in binding layers

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| HTTP client | Custom reqwest wrapper | ureq (already used) | Blocking HTTP simpler for FFI; spawn_blocking bridges to async |
| JSON parsing | Manual string parsing | serde_json + typed models | Type safety, error handling, maintainability |
| Parameter validation | Ad-hoc checks | Builder pattern validation | Consistent API, compile-time guarantees |
| Async FFI bridging | Manual thread pools | tokio::spawn_blocking | Tokio runtime already required; proven pattern |
| Type conversion | Manual JSON construction | `From` trait + serde | Type-safe, automatic conversion |

**Key insight:** The SDK already has proven patterns for REST endpoints (intraday quote/ticker). New endpoints are variations on the same theme—don't reinvent, replicate the pattern.

## Common Pitfalls

### Pitfall 1: serde Rename Mismatch
**What goes wrong:** Rust model fields don't match API response keys
**Why it happens:** Fugle API uses camelCase (lastPrice), Rust convention is snake_case (last_price)
**How to avoid:** Always use `#[serde(rename = "camelCase")]` on every field that doesn't match exactly
**Warning signs:** JSON deserialization errors, field values showing as None when API returns data

### Pitfall 2: Optional Field Handling
**What goes wrong:** Required fields marked as Option<T>, causing unwrap panics
**Why it happens:** Not all API endpoints return the same fields; marking everything as Option "just in case"
**How to avoid:** Analyze official SDK response types; only make fields optional if they're actually optional in the API contract
**Warning signs:** Tests pass but production panics on unwrap; compatibility tests fail on field access

### Pitfall 3: Blocking FFI Without spawn_blocking
**What goes wrong:** Language bindings hang or deadlock
**Why it happens:** Calling blocking ureq directly from async context holds event loop
**How to avoid:** Always wrap core REST calls in `tokio::task::spawn_blocking`
**Warning signs:** Python GIL deadlocks, Node.js event loop stalls, C# Task timeouts

### Pitfall 4: Inconsistent Timeframe Parameters
**What goes wrong:** Some endpoints accept "D|W|M|1|5|..." while others don't
**Why it happens:** Different endpoint categories have different parameter requirements
**How to avoid:** Verify parameter signature from official SDK for each endpoint; use builder pattern to constrain valid values
**Warning signs:** API returns 400 Bad Request on valid-looking parameters

### Pitfall 5: Missing UniFFI Model Attributes
**What goes wrong:** New models don't propagate to C#/Java/Go bindings
**Why it happens:** Forgot `#[derive(uniffi::Record)]` on new model types
**How to avoid:** Every new model in `core/src/models/` must have all three: `#[cfg_attr(feature = "python", pyo3::pyclass)]`, `#[cfg_attr(feature = "js", napi(object))]`, and be exported from `uniffi/src/models.rs`
**Warning signs:** Compilation succeeds but binding generation fails; C# project shows missing types

## Code Examples

Verified patterns from official sources and existing implementation:

### Historical Candles Endpoint Pattern
```rust
// Source: Verified from fugle-marketdata-node/src/rest/stock/historical/candles.ts
// Location: core/src/rest/stock/historical/candles.rs (to be created)

pub struct HistoricalCandlesRequestBuilder<'a> {
    client: &'a RestClient,
    symbol: Option<String>,
    from: Option<String>,
    to: Option<String>,
    timeframe: Option<String>,  // "D", "W", "M", "1", "5", "10", "15", "30", "60"
    fields: Option<String>,
    sort: Option<String>,       // "asc" or "desc"
    adjusted: Option<bool>,
}

impl<'a> HistoricalCandlesRequestBuilder<'a> {
    pub fn send(self) -> Result<HistoricalCandlesResponse, MarketDataError> {
        let symbol = self.symbol.ok_or(MarketDataError::InvalidSymbol { ... })?;
        let mut url = format!("{}/stock/historical/candles/{}", self.client.base_url(), symbol);

        // Build query params from optional fields
        let params = vec![
            self.from.map(|v| format!("from={}", v)),
            self.to.map(|v| format!("to={}", v)),
            self.timeframe.map(|v| format!("timeframe={}", v)),
            // ... additional params
        ].into_iter().flatten().collect::<Vec<_>>().join("&");

        if !params.is_empty() {
            url.push_str(&format!("?{}", params));
        }

        let response = self.client.agent().get(&url).call()?;
        Ok(response.into_json()?)
    }
}
```

### Technical Indicator Model Pattern
```rust
// Source: Verified from fugle-marketdata-node/src/rest/stock/technical/sma.ts
// Location: core/src/models/technical.rs (to be created)

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
#[cfg_attr(feature = "js", napi_derive::napi(object))]
pub struct SmaResponse {
    pub symbol: String,
    #[serde(rename = "type")]
    pub data_type: String,
    pub exchange: String,
    pub market: String,
    pub timeframe: String,
    pub data: Vec<SmaDataPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
#[cfg_attr(feature = "js", napi_derive::napi(object))]
pub struct SmaDataPoint {
    pub date: String,
    pub sma: f64,
}
```

### Snapshot Quotes Endpoint (Market Parameter)
```rust
// Source: Verified from fugle-marketdata-node/src/rest/stock/snapshot/quotes.ts
// Location: core/src/rest/stock/snapshot/quotes.rs (to be created)

pub struct SnapshotQuotesRequestBuilder<'a> {
    client: &'a RestClient,
    market: Option<String>,  // "TSE" | "OTC" | "ESB" | "TIB" | "PSB"
    type_filter: Option<String>,  // "ALL" | "ALLBUT0999" | "COMMONSTOCK"
}

impl<'a> SnapshotQuotesRequestBuilder<'a> {
    pub fn market(mut self, market: &str) -> Self {
        self.market = Some(market.to_string());
        self
    }

    pub fn type_filter(mut self, type_filter: &str) -> Self {
        self.type_filter = Some(type_filter.to_string());
        self
    }

    pub fn send(self) -> Result<SnapshotQuotesResponse, MarketDataError> {
        let market = self.market.ok_or(MarketDataError::InvalidParameter { ... })?;
        let url = format!("{}/stock/snapshot/quotes/{}", self.client.base_url(), market);
        // Add type query param if provided
        // Make request and deserialize
    }
}
```

### UniFFI Integration Pattern
```rust
// Source: Existing uniffi/src/client.rs pattern
// Add to StockClient impl block:

#[uniffi::export]
impl StockClient {
    /// Access historical endpoints
    pub fn historical(&self) -> Arc<StockHistoricalClient> {
        Arc::new(StockHistoricalClient::new(self.inner.clone()))
    }

    /// Access snapshot endpoints
    pub fn snapshot(&self) -> Arc<StockSnapshotClient> {
        Arc::new(StockSnapshotClient::new(self.inner.clone()))
    }

    /// Access technical indicator endpoints
    pub fn technical(&self) -> Arc<StockTechnicalClient> {
        Arc::new(StockTechnicalClient::new(self.inner.clone()))
    }

    /// Access corporate actions endpoints
    pub fn corporate_actions(&self) -> Arc<StockCorporateActionsClient> {
        Arc::new(StockCorporateActionsClient::new(self.inner.clone()))
    }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| JSON string returns | Typed models via serde | Phase 4.1 (UniFFI migration) | Type safety, IDE support, compile-time checks |
| Separate binding architectures | Unified UniFFI for C#/Java/Go | Phase 4.1-4.2 (2026-01-31) | Single source of truth, consistent API |
| UDL file definitions | Proc-macro attributes | Phase 4.1 (2026-01-31) | No duplicate type definitions, better ergonomics |
| Sync-only APIs | Async/sync dual methods | Phase 2-3 (Python/Node.js) | Non-blocking I/O, better performance |

**Deprecated/outdated:**
- **csbindgen approach:** Replaced by UniFFI in Phase 4.1; C# now uses uniffi-bindgen-cs
- **Manual JSON serialization:** All bindings now use typed models from core
- **Callback-only WebSocket:** Now has async iterator (Python) and EventEmitter (Node.js) patterns

## Open Questions

Things that couldn't be fully resolved:

1. **Fugle API official documentation site (developer.fugle.tw) returned configuration error**
   - What we know: Site exists but baseUrl misconfigured; cannot access official REST API docs
   - What's unclear: Exact API contract for edge cases (rate limits, error codes, pagination)
   - Recommendation: Use official Python/Node.js SDK source code as source of truth; they are maintained by Fugle team

2. **Batch tickers endpoint parameters unclear**
   - What we know: Endpoints exist (`/stock/intraday/tickers`, `/futopt/intraday/tickers`)
   - What's unclear: Full parameter schema (filtering, pagination, sorting options)
   - Recommendation: Analyze Python SDK `intraday.py` and Node.js SDK `tickers.ts` for exact signatures

3. **Technical indicator calculation parameters**
   - What we know: Endpoints exist (SMA, RSI, KDJ, MACD, BB); basic params include symbol, from, to, timeframe, period
   - What's unclear: Advanced parameters (smoothing factors, deviation multiples for BB)
   - Recommendation: Use Node.js SDK TypeScript interfaces as specification; they include full param types

## Sources

### Primary (HIGH confidence)
- **Official Python SDK:** `/Users/zackfan/Project/fugle/fugle-marketdata-python/fugle_marketdata/rest/stock/`
  - Verified: historical.py, snapshot.py, technical.py, corporate_actions.py, intraday.py
  - Provides: Exact endpoint paths and method signatures
- **Official Node.js SDK:** `/Users/zackfan/Project/fugle/fugle-marketdata-node/src/rest/stock/`
  - Verified: RestStockClient structure with all endpoint categories
  - Provides: TypeScript response types and parameter interfaces
- **Current Rust implementation:** `/Users/zackfan/Project/fugle/fugle-marketdata-sdk/core/src/`
  - Verified: Existing patterns for intraday endpoints, Quote model structure
  - Provides: Builder pattern, serde attributes, error handling

### Secondary (MEDIUM confidence)
- **Fugle GitHub Organization:** [https://github.com/fugle-dev](https://github.com/fugle-dev)
  - Context: Official SDK repositories and maintenance activity
- **PyPI fugle-marketdata:** [https://pypi.org/project/fugle-marketdata/](https://pypi.org/project/fugle-marketdata/)
  - Context: Python package metadata and version history
- **npm @fugle/marketdata:** [https://www.npmjs.com/package/@fugle/marketdata](https://www.npmjs.com/package/@fugle/marketdata)
  - Context: Node.js package metadata

### Tertiary (LOW confidence)
- **Fugle Developer Portal:** [https://developer.fugle.tw/](https://developer.fugle.tw/) (site configuration error)
  - Issue: Docusaurus baseUrl misconfigured; cannot access API documentation
  - Status: Must rely on SDK source code instead

## API Endpoint Inventory

Based on official SDK analysis, here are the missing endpoints:

### Stock - Historical (2 endpoints)
- `GET /stock/historical/candles/{symbol}` - Historical OHLCV data
  - Params: from, to, timeframe (D|W|M|1|5|10|15|30|60), fields, sort, adjusted
- `GET /stock/historical/stats/{symbol}` - Historical statistics
  - Params: symbol

### Stock - Snapshot (3 endpoints)
- `GET /stock/snapshot/quotes/{market}` - Market-wide quotes snapshot
  - Params: market (TSE|OTC|ESB|TIB|PSB), type (ALL|ALLBUT0999|COMMONSTOCK)
- `GET /stock/snapshot/movers/{market}` - Top movers (gainers/losers)
  - Params: market, direction?, change?
- `GET /stock/snapshot/actives/{market}` - Most active stocks by volume
  - Params: market, trade?

### Stock - Technical Indicators (5 endpoints)
- `GET /stock/technical/sma/{symbol}` - Simple Moving Average
  - Params: symbol, from, to, timeframe, period
- `GET /stock/technical/rsi/{symbol}` - Relative Strength Index
  - Params: symbol, from, to, timeframe, period
- `GET /stock/technical/kdj/{symbol}` - KDJ Stochastic Oscillator
  - Params: symbol, from, to, timeframe, period
- `GET /stock/technical/macd/{symbol}` - MACD Indicator
  - Params: symbol, from, to, timeframe, fast, slow, signal
- `GET /stock/technical/bb/{symbol}` - Bollinger Bands
  - Params: symbol, from, to, timeframe, period, stddev

### Stock - Corporate Actions (3 endpoints)
- `GET /stock/corporate-actions/capital-changes` - Capital structure changes
  - Params: date?, startDate?, endDate?
- `GET /stock/corporate-actions/dividends` - Dividend announcements
  - Params: date?, startDate?, endDate?
- `GET /stock/corporate-actions/listing-applicants` - IPO listings
  - Params: date?, startDate?, endDate?

### Stock - Intraday Batch (1 endpoint)
- `GET /stock/intraday/tickers` - Batch ticker information
  - Params: type?, exchange?, market?, industry?
  - Status: Already implemented in core (verified)

### FutOpt - Historical (2 endpoints)
- `GET /futopt/historical/candles/{symbol}` - Historical futures/options candles
  - Params: from, to, timeframe
- `GET /futopt/historical/daily/{symbol}` - Daily futures/options data
  - Params: from, to

### FutOpt - Intraday Batch (1 endpoint)
- `GET /futopt/intraday/tickers` - Batch contract information
  - Params: contractType?, exchange?
  - Status: Already implemented in core (verified)

**Total: 17 endpoints** (15 Stock + 2 FutOpt)
**Status: 0 implemented, 17 remaining**

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - All libraries already in use and proven
- Architecture: HIGH - Patterns verified in existing code (quote, ticker endpoints)
- Pitfalls: HIGH - Known issues from Phase 2-6 execution (serde renames, spawn_blocking)
- API inventory: HIGH - Verified from official SDK source code
- Official docs: LOW - Developer portal inaccessible; relying on SDK source

**Research date:** 2026-01-31
**Valid until:** 60 days (stable SDK APIs; official SDKs updated infrequently)

**Next steps for planning:**
1. Break down by endpoint category (Historical, Snapshot, Technical, Corporate)
2. Each category: Rust models → Core endpoints → UniFFI exports → Language bindings
3. Use compatibility tests from Phase 6 infrastructure for validation
4. Integration tests with VCR cassettes for regression prevention
