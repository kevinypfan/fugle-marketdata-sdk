# subscription-api Specification

## Purpose
TBD - created by archiving change sdk-04-improvements. Update Purpose after archive.
## Requirements
### Requirement: Subscription introspection helpers
Both sync and async `WebSocketClient` SHALL expose:
- `subscription_count(&self) -> usize` returning the number of currently active subscriptions
- `is_subscribed(&self, channel: &Channel, symbol: &str) -> bool` returning whether the given channel+symbol pair is currently subscribed

Both helpers MUST be backed by the existing `SubscriptionManager::contains()` (`core/src/websocket/subscription.rs:103`) and `SubscriptionManager::count()` (`subscription.rs:109`); no new internal data structures may be introduced.

#### Scenario: Count reflects active subscriptions
- **WHEN** a client subscribes to two distinct channel+symbol pairs and both succeed
- **THEN** `client.subscription_count()` MUST return 2

#### Scenario: is_subscribed positive case
- **WHEN** the client has an active `Channel::Trades` + `"2330"` subscription
- **THEN** `client.is_subscribed(&Channel::Trades, "2330")` MUST return `true`

#### Scenario: is_subscribed negative case
- **WHEN** the client has an active `Channel::Trades` + `"2330"` subscription but NO `Channel::Books` + `"2330"` subscription
- **THEN** `client.is_subscribed(&Channel::Books, "2330")` MUST return `false`

#### Scenario: Count after unsubscribe
- **WHEN** the client subscribes to two pairs then unsubscribes from one
- **THEN** `client.subscription_count()` MUST return 1

### Requirement: Legacy SubscribeRequest constructors removed
The constructor methods `SubscribeRequest::trades`, `SubscribeRequest::candles`, `SubscribeRequest::books`, and `SubscribeRequest::aggregates` (currently at `core/src/models/subscription.rs:212-229`) SHALL be removed. The canonical builder API (`SubscribeRequest::new(channel, symbol)` plus chainable modifiers) is the supported replacement.

**Reason**: The 0.2.0 changelog already removed `SubscribeRequest` from the public re-export surface, but these helper constructors remained dangling. They duplicate the canonical `new(channel, symbol)` form and create two ways to do the same thing, complicating future channel additions.

**Migration**: Replace `SubscribeRequest::trades(symbol)` with `SubscribeRequest::new(Channel::Trades, symbol)`. Same pattern for `candles`, `books`, `aggregates` with their corresponding `Channel::*` variants.

#### Scenario: Removed constructors do not compile
- **WHEN** downstream code calls `SubscribeRequest::trades("2330")` after upgrading to 0.4.0
- **THEN** the code MUST fail to compile with a method-not-found error pointing to the canonical `new(channel, symbol)` constructor

#### Scenario: Canonical form preserved
- **WHEN** downstream code calls `SubscribeRequest::new(Channel::Trades, "2330")`
- **THEN** the code MUST compile and produce a request equivalent to the removed `SubscribeRequest::trades("2330")` form

### Requirement: Symbols type with normalization helpers

The crate SHALL expose `pub enum Symbols { Single(String), Many(Vec<String>) }` from the module `core/src/websocket/models/symbols.rs`, re-exported at `core::websocket::models::Symbols` and at the crate root `fugle_marketdata_core::Symbols`. The enum SHALL provide:

- `From` impls covering at minimum: `&str`, `String`, `&String`, `Vec<String>`, `Vec<&str>`, `[&str; N]`, `[String; N]`, `&[&str]`, `&[String]`.
- `fn normalized(self) -> Self` — trim leading/trailing whitespace on each entry, drop empty entries (post-trim), deduplicate preserving first-seen insertion order, and collapse `Many` of length 1 down to `Single` for canonical form. The dedup comparison MUST be byte-for-byte case-sensitive — `"TXFB6"` and `"txfb6"` MUST remain distinct entries. Case folding is the caller's responsibility.
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

#### Scenario: Normalize preserves case (no case folding)
- **WHEN** `Symbols::from(vec!["TXFB6", "txfb6", "TxFb6"]).normalized()` is evaluated
- **THEN** the result MUST equal `Symbols::Many(vec!["TXFB6".into(), "txfb6".into(), "TxFb6".into()])` (all three retained as distinct entries)

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

