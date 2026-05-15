## 1. ErrorKind classification helper

- [ ] 1.1 Add `pub enum ErrorKind { Network, Protocol, Auth, Client }` with `#[non_exhaustive]` to `core/src/errors.rs`
- [ ] 1.2 Add `pub fn source_kind(&self) -> ErrorKind` to `impl MarketDataError`, implementing the mapping table from the spec
- [ ] 1.3 Re-export `ErrorKind` from `core/src/lib.rs`
- [ ] 1.4 Add unit tests covering all 6 spec scenarios (Network/Protocol/Auth/Network-5xx/Client-validation/wildcard-required)
- [ ] 1.5 Document the WebSocket-variant coarse-grained mapping (everything → Protocol pre-0.6.0) in the `source_kind()` rustdoc
- [ ] 1.6 `cargo test --all-features -p fugle-marketdata-core --lib` clean

## 2. Event drop counter

- [ ] 2.1 Refactor `core/src/websocket/connection_event.rs` — change `emit_event` signature to take `&Arc<AtomicU64>` counter alongside the sender; increment on `mpsc::TrySendError::Full`
- [ ] 2.2 Add `events_dropped_total: Arc<AtomicU64>` field to `aio::WebSocketClient` shared state (mirror `messages_dropped_total` plumbing)
- [ ] 2.3 Add `events_dropped_total: Arc<AtomicU64>` field to sync `WebSocketClient` shared state
- [ ] 2.4 Add `pub fn events_dropped_total(&self) -> u64` accessor on both clients
- [ ] 2.5 Add unit test covering "zero at construction", "increments on saturation", "monotonic"
- [ ] 2.6 Verify FFI bindings (py/js/uniffi) compile — re-exports are additive only

## 3. Symbols case policy

- [ ] 3.1 Update `core/src/models/symbols.rs` module-level rustdoc to state: "Dedup is byte-for-byte case-sensitive. `Symbols::normalized` MUST NOT case-fold."
- [ ] 3.2 Add unit test `normalize_preserves_case` asserting `["TXFB6", "txfb6", "TxFb6"]` produces three distinct entries
- [ ] 3.3 Update `MIGRATION-0.5.md` (or add a note) referencing the case policy for FFI binding implementers

## 4. Stability docs on FFI-load-bearing constructors

- [ ] 4.1 Add `#[must_use]` + `## Stability` rustdoc section to `ReconnectionConfig::disabled()`
- [ ] 4.2 Add `#[must_use]` + `## Stability` rustdoc section to `ReconnectionConfig::default()` (note `Default` trait impl forwards to the inherent fn so the doc is reachable)
- [ ] 4.3 Add `#[must_use]` + `## Stability` rustdoc section to `RetryPolicy::conservative()`
- [ ] 4.4 Add `#[must_use]` + `## Stability` rustdoc section to `RetryPolicy::aggressive()`
- [ ] 4.5 Verify `cargo doc --all-features -p fugle-marketdata-core` still clean
- [ ] 4.6 Verify `cargo clippy --all-features -p fugle-marketdata-core -- -D warnings` still clean

## 5. Idiomatic constructor section

- [ ] 5.1 Add "## Which constructor should I use?" section to `core/README.md`, positioned above "## API Reference"
- [ ] 5.2 Section covers four paths: bon builder, positional new(), typestate WebSocketFactory, convenience constructors
- [ ] 5.3 All code samples tagged `rust,ignore`
- [ ] 5.4 Run `cargo test --doc -p fugle-marketdata-core` — verify no new doctest failures

## 6. Release prep

- [ ] 6.1 Bump `core/Cargo.toml` 0.5.0 → 0.5.1
- [ ] 6.2 Bump `rust/Cargo.toml` 0.5.0 → 0.5.1
- [ ] 6.3 Bump workspace dep `marketdata-core` 0.5.0 → 0.5.1 in root `Cargo.toml`
- [ ] 6.4 Add `[Rust 0.5.1]` section to `CHANGELOG.md` with `### Added` (source_kind, events_dropped_total, stability docs, idiomatic constructor section) and `### Documentation` (case policy)
- [ ] 6.5 `cargo publish --dry-run -p fugle-marketdata-core --allow-dirty` clean
- [ ] 6.6 Open PR; merge after CI green
- [ ] 6.7 Tag `v0.5.1`; publish `fugle-marketdata-core` and `fugle-marketdata` to crates.io
- [ ] 6.8 Archive change: `openspec archive core-0-5-1-followups`
