## ADDED Requirements

### Requirement: Idiomatic constructor selection guide

`core/README.md` SHALL include a section titled "Which constructor should I use?" that documents the idiomatic choice for each of the four construction paths exposed by the crate. The section MUST appear before the "API Reference" section (so users encounter it on the landing page of docs.rs via the `#![doc = include_str!("../README.md")]` re-export). The required content:

- `bon` builders (`SubscribeRequest::builder()`, `RetryPolicy::builder()`, `ReconnectionConfig::builder()`) — recommended for default-filled construction with `maybe_*` Option setters; no validation.
- Positional constructors (`ReconnectionConfig::new(...)`) — recommended when validation matters; return `Result<Self, MarketDataError>`.
- Typestate factory (`WebSocketFactory::new().auth(...).stock().build()`) — recommended for endpoint derivation across stock + futopt.
- Convenience constructors (`ConnectionConfig::fugle_stock(auth)` etc.) — recommended for one-shot config in examples and scripts.

Code samples in the section MUST be tagged `rust,ignore` to keep `cargo test --doc` green.

#### Scenario: Section exists at expected location
- **WHEN** `core/README.md` is read
- **THEN** the file MUST contain a heading matching `## Which constructor should I use?` positioned before any heading matching `## API Reference`

#### Scenario: Section covers all four paths
- **WHEN** the section is read
- **THEN** the prose MUST mention all four construction paths by name: `bon` builder, positional `new(...)`, `WebSocketFactory`, convenience constructors

#### Scenario: Code samples are tagged ignore
- **WHEN** code samples appear in the section
- **THEN** every fenced ` ```rust ` block MUST be tagged `rust,ignore` so doctests do not execute partial snippets

### Requirement: Stability promise on FFI-load-bearing constructors

The following constructors SHALL carry both `#[must_use]` and a rustdoc `## Stability` section documenting their public-API status. The Stability section MUST state that FFI binding crates (Python, Node, UniFFI, Go, Java, C#) depend on the name and signature, and that the function will be preserved across all 0.x releases:

- `ReconnectionConfig::disabled()`
- `ReconnectionConfig::default()`
- `RetryPolicy::conservative()`
- `RetryPolicy::aggressive()`

#### Scenario: disabled() carries must_use
- **WHEN** `core/src/websocket/reconnection.rs` is read
- **THEN** `pub fn disabled()` MUST be preceded by `#[must_use]`

#### Scenario: disabled() carries stability section
- **WHEN** `cargo doc --all-features -p fugle-marketdata-core` renders `ReconnectionConfig::disabled`
- **THEN** the rendered docs MUST contain a heading `Stability` and prose stating that FFI bindings depend on this function and it is preserved across 0.x releases

#### Scenario: All four constructors carry must_use
- **WHEN** all four functions are read
- **THEN** each MUST be preceded by `#[must_use]`
