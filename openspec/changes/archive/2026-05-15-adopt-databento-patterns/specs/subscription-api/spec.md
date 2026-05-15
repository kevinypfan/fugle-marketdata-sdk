## ADDED Requirements

### Requirement: Symbols type with normalization helpers

The crate SHALL expose `pub enum Symbols { Single(String), Many(Vec<String>) }` from the module `core/src/websocket/models/symbols.rs`, re-exported at `core::websocket::models::Symbols` and at the crate root `fugle_marketdata_core::Symbols`. The enum SHALL provide:

- `From` impls covering at minimum: `&str`, `String`, `&String`, `Vec<String>`, `Vec<&str>`, `[&str; N]`, `[String; N]`, `&[&str]`, `&[String]`.
- `fn normalized(self) -> Self` — trim leading/trailing whitespace on each entry, drop empty entries (post-trim), deduplicate preserving first-seen insertion order, and collapse `Many` of length 1 down to `Single` for canonical form.
- `fn len(&self) -> usize` — number of symbols.
- `fn is_empty(&self) -> bool` — true when no symbols remain.
- `fn iter(&self) -> impl Iterator<Item = &str>` — iterate symbols.
- `fn chunked(self, max_per_chunk: usize) -> Vec<Symbols>` — split into N-sized chunks; `max_per_chunk = 0` MUST panic with a clear message.

`Symbols` MUST `#[derive(Clone, Debug, PartialEq, Eq)]`.

#### Scenario: From &str produces Single
- **WHEN** `Symbols::from("2330")` is evaluated
- **THEN** the result MUST equal `Symbols::Single("2330".to_string())`

#### Scenario: From vec produces Many
- **WHEN** `Symbols::from(vec!["2330", "2454"])` is evaluated
- **THEN** the result MUST equal `Symbols::Many(vec!["2330".into(), "2454".into()])`

#### Scenario: Normalize trims whitespace
- **WHEN** `Symbols::from(vec!["  2330  ", "2454\n"]).normalized()` is evaluated
- **THEN** the result MUST equal `Symbols::Many(vec!["2330".into(), "2454".into()])`

#### Scenario: Normalize deduplicates preserving order
- **WHEN** `Symbols::from(vec!["2330", "2454", "2330", "2317"]).normalized()` is evaluated
- **THEN** the result MUST equal `Symbols::Many(vec!["2330".into(), "2454".into(), "2317".into()])` (duplicate "2330" dropped, original order preserved)

#### Scenario: Normalize drops empty entries
- **WHEN** `Symbols::from(vec!["2330", "", "  ", "2454"]).normalized()` is evaluated
- **THEN** the result MUST equal `Symbols::Many(vec!["2330".into(), "2454".into()])`

#### Scenario: Normalize collapses Many-of-one to Single
- **WHEN** `Symbols::from(vec!["2330"]).normalized()` is evaluated
- **THEN** the result MUST equal `Symbols::Single("2330".to_string())`

#### Scenario: Len reflects symbol count
- **WHEN** `Symbols::from(vec!["2330", "2454"]).len()` is evaluated
- **THEN** the result MUST equal `2`

#### Scenario: Chunked splits into requested sizes
- **WHEN** `Symbols::from(vec!["A", "B", "C", "D", "E"]).chunked(2)` is evaluated
- **THEN** the result MUST be a `Vec<Symbols>` of length 3 containing in order: `Many(["A", "B"])`, `Many(["C", "D"])`, `Single("E")`

#### Scenario: Chunked with zero panics
- **WHEN** `Symbols::from(vec!["A"]).chunked(0)` is called
- **THEN** the call MUST panic with a clear message indicating `max_per_chunk` must be non-zero

### Requirement: Subscription dispatch deduplicates symbols

`SubscribeRequest::with_symbols(channel, symbols)` and the channel-specific subscription constructors (`StockSubscription::new`, `FutOptSubscription::new`) SHALL invoke `Symbols::normalized()` on their input before producing the `SubscribeRequest` or expanding to internal per-symbol entries. After normalization, duplicate symbols MUST be collapsed to a single subscription such that `SubscriptionManager` records exactly one entry per unique normalized symbol.

#### Scenario: Duplicate symbols collapse to one subscription
- **WHEN** `SubscribeRequest::with_symbols(Channel::Trades, vec!["2330", "2330"])` is built and dispatched
- **THEN** `SubscriptionManager` MUST contain exactly one entry for `(Channel::Trades, "2330")`, and only one subscription frame MUST be sent on the wire

#### Scenario: Whitespace-only differences collapse
- **WHEN** `SubscribeRequest::with_symbols(Channel::Trades, vec!["2330", " 2330 "])` is built and dispatched
- **THEN** `SubscriptionManager` MUST contain exactly one entry for `(Channel::Trades, "2330")`

#### Scenario: Distinct symbols all retained
- **WHEN** `SubscribeRequest::with_symbols(Channel::Trades, vec!["2330", "2454", "2317"])` is built and dispatched
- **THEN** `SubscriptionManager` MUST contain three entries in insertion order: `("Trades", "2330")`, `("Trades", "2454")`, `("Trades", "2317")`

### Requirement: Subscription batch limit reservation

The crate SHALL expose `pub const SUBSCRIPTION_BATCH_LIMIT: Option<usize>` in `core::websocket::models::symbols`. As of this release the value MUST be `None`, signaling that the SDK does not enforce a per-frame symbol limit and the dispatch path emits a single subscription frame regardless of symbol count. The constant is reserved so that downstream code can branch on a future server-documented limit without an additional version bump.

#### Scenario: Constant is exported as None
- **WHEN** `fugle_marketdata_core::websocket::models::symbols::SUBSCRIPTION_BATCH_LIMIT` is referenced
- **THEN** the value MUST be `Option::<usize>::None`

#### Scenario: Single dispatch frame regardless of count
- **WHEN** `SubscribeRequest::with_symbols(Channel::Trades, (0..100).map(|i| format!("S{i}")).collect::<Vec<_>>())` is built and dispatched
- **THEN** exactly one subscription frame containing all 100 symbols MUST be sent (no auto-chunking)

## RENAMED Requirements

- FROM: `SymbolSpec`
- TO: `Symbols`

The public type `SymbolSpec` is renamed to `Symbols`. All `From` impls previously targeting `SymbolSpec` MUST retarget `Symbols`. No backward-compat alias is exposed; downstream Rust code MUST update `use` statements as part of upgrading to 0.5.0. The migration is mechanical (`sed -i '' 's/SymbolSpec/Symbols/g'` over downstream source files).
