---
phase: 07-complete-rest-api-coverage
plan: 08
subsystem: nodejs-binding
tags: [napi-rs, typescript, nodejs, rest-api, async]

dependency-graph:
  requires: [07-01, 07-02, 07-03, 07-04, 07-05]
  provides: [StockHistoricalClient, StockSnapshotClient, StockTechnicalClient, StockCorporateActionsClient, FutOptHistoricalClient]
  affects: [07-09 UniFFI bindings]

tech-stack:
  added: [typescript-dev-dependency]
  patterns: [spawn_blocking, napi-getter, ts_return_type-annotations]

key-files:
  created: []
  modified:
    - js/src/client.rs
    - js/types.d.ts
    - js/index.d.ts
    - js/index.js
    - js/package.json

decisions:
  - id: D-07-08-1
    choice: "Use intermediate let bindings for client chains to satisfy Rust lifetime requirements"
    reason: "Core clients return references that don't live long enough when chained directly; let bindings extend lifetime"
  - id: D-07-08-2
    choice: "Pass after_hours as Option<bool> instead of flag method"
    reason: "Core API requires after_hours(bool) not after_hours() flag; Option enables conditional application"
  - id: D-07-08-3
    choice: "Add TypeScript as dev dependency for type checking"
    reason: "Enables npx tsc --noEmit validation of generated type definitions"

metrics:
  duration: 6 min
  completed: 2026-01-31
---

# Phase 7 Plan 8: Node.js Bindings for New REST Endpoints Summary

Node.js binding layer extended with 5 new client classes for historical, snapshot, technical, corporate actions, and FutOpt historical endpoints.

## One-liner

5 new napi-rs client classes (720 LoC) + TypeScript definitions (614 LoC) for all Wave 1 REST endpoints

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add Stock Historical and Snapshot clients | 9f2882d | js/src/client.rs |
| 2 | Add Technical, Corporate Actions, and FutOpt Historical clients | 9f2882d | js/src/client.rs |
| 3 | Update TypeScript type definitions | 9f2882d | js/types.d.ts |
| - | Update generated files and dependencies | 6751847 | js/index.d.ts, js/index.js, etc. |

## Artifacts Produced

### New Node.js Client Classes (5)

| Client | Methods | Description |
|--------|---------|-------------|
| StockHistoricalClient | candles(), stats() | Historical OHLCV and stats data |
| StockSnapshotClient | quotes(), movers(), actives() | Market-wide snapshot data |
| StockTechnicalClient | sma(), rsi(), kdj(), macd(), bb() | Technical indicators |
| StockCorporateActionsClient | capital_changes(), dividends(), listing_applicants() | Corporate events |
| FutOptHistoricalClient | candles(), daily() | FutOpt historical data |

### New Async Methods (15)

All methods follow the established pattern:
- `spawn_blocking` wraps core sync HTTP calls
- Returns `Promise<Value>` (serde_json serialized)
- Has `#[napi(ts_return_type = "Promise<T>")]` annotation

### TypeScript Type Definitions

52 interfaces total (34 new), including:
- Historical: HistoricalCandle, HistoricalCandlesResponse, StatsResponse
- Snapshot: SnapshotQuote, SnapshotQuotesResponse, Mover, MoversResponse, Active, ActivesResponse
- Technical: SmaResponse, RsiResponse, KdjResponse, MacdResponse, BbResponse + data points
- Corporate: CapitalChange, CapitalChangesResponse, Dividend, DividendsResponse, ListingApplicant, ListingApplicantsResponse
- FutOpt: FutOptHistoricalCandle, FutOptHistoricalCandlesResponse, FutOptDailyData, FutOptDailyResponse
- Client interfaces: StockHistoricalClient, StockSnapshotClient, StockTechnicalClient, StockCorporateActionsClient, FutOptHistoricalClient

## Technical Details

### Lifetime Management Pattern

Core clients return reference-based clients that require careful lifetime management:

```rust
// WRONG: Temporary dropped while borrowed
let mut builder = inner.stock().historical().candles().symbol(&symbol);

// CORRECT: Intermediate bindings extend lifetime
let stock = inner.stock();
let hist = stock.historical();
let mut builder = hist.candles().symbol(&symbol);
```

### Client Access Hierarchy

```
RestClient
├── stock: StockClient
│   ├── intraday: StockIntradayClient
│   ├── historical: StockHistoricalClient (NEW)
│   ├── snapshot: StockSnapshotClient (NEW)
│   ├── technical: StockTechnicalClient (NEW)
│   └── corporate_actions: StockCorporateActionsClient (NEW)
└── futopt: FutOptClient
    ├── intraday: FutOptIntradayClient
    └── historical: FutOptHistoricalClient (NEW)
```

## Usage Example

```javascript
const { RestClient } = require('@fugle/marketdata');

const client = new RestClient('your-api-key');

// Historical candles
const candles = await client.stock.historical.candles('2330', '2024-01-01', '2024-01-31', 'D');

// Snapshot quotes for market
const quotes = await client.stock.snapshot.quotes('TSE');

// Technical indicators
const sma = await client.stock.technical.sma('2330', '2024-01-01', '2024-01-31', 'D', 20);
const macd = await client.stock.technical.macd('2330', null, null, 'D', 12, 26, 9);

// Corporate actions
const dividends = await client.stock.corporate_actions.dividends(null, '2024-01-01', '2024-12-31');

// FutOpt historical
const futoptCandles = await client.futopt.historical.candles('TXFC4', '2024-01-01', '2024-01-31');
```

## Deviations from Plan

None - plan executed exactly as written.

## Verification Results

| Check | Status | Details |
|-------|--------|---------|
| cargo check -p marketdata-js | PASS | All Rust code compiles |
| npm run build | PASS | Native addon built successfully |
| npx tsc --noEmit index.d.ts | PASS | TypeScript types valid |
| npm test | PASS | 73 passed, 22 skipped |
| Module importable | PASS | RestClient and WebSocketClient exported |

## Success Criteria Verification

| Criteria | Status |
|----------|--------|
| 5 new Node.js client classes with #[napi] | PASS (7 total, 5 new) |
| All async methods use spawn_blocking | PASS (28 uses) |
| TypeScript types for all new response types | PASS (52 interfaces) |
| All methods have ts_return_type annotations | PASS (26 annotations) |
| npm run build succeeds | PASS |

## Commits

| Hash | Type | Message |
|------|------|---------|
| 9f2882d | feat | feat(07-08): add Node.js bindings for new REST endpoints |
| 6751847 | chore | chore(07-08): update generated files and dependencies |

## Next Steps

- Plan 07-09: UniFFI bindings for C#/Java/Go
- Integration tests with real API key
- Documentation updates for new endpoints
