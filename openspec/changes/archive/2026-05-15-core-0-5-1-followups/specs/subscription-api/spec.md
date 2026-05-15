## MODIFIED Requirements

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
