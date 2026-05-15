## 1. ErrorKind classification helper

- [x] 1.1 Add `pub enum ErrorKind { Network, Protocol, Auth, RateLimit, Client }` with `#[non_exhaustive]` to `core/src/errors.rs` (RateLimit added during apply — operationally distinct from Network)
- [x] 1.2 Add `pub fn source_kind(&self) -> ErrorKind` to `impl MarketDataError`
- [x] 1.3 Re-export `ErrorKind` from `core/src/lib.rs`
- [x] 1.4 7 unit tests covering all categories (Network/Protocol/Auth/RateLimit/Network-5xx/Client-validation/Client-4xx-other) plus `error_kind_variants_exist`
- [x] 1.5 Documented WebSocket-variant coarse-grained mapping (everything → Protocol pre-0.6.0) in `source_kind()` rustdoc
- [x] 1.6 `cargo test --all-features -p fugle-marketdata-core --lib` clean

## 2. Event drop counter

- [x] 2.1 Refactored `emit_event` signature to take `&Arc<AtomicU64>` counter; increments on `mpsc::TrySendError::Full` (56 call sites updated)
- [x] 2.2 Added `events_dropped: Arc<AtomicU64>` field to `aio::WebSocketClient`
- [x] 2.3 Added `events_dropped: Arc<AtomicU64>` field to sync `OwnerShared` (sync `WebSocketClient`)
- [x] 2.4 Added `pub fn events_dropped_total(&self) -> u64` accessor on both clients
- [x] 2.5 Added `events_dropped_total_starts_at_zero`, `events_dropped_increments_on_saturation` (sync), and updated `emit_event_drops_when_channel_full` to assert counter increments
- [x] 2.6 FFI bindings (py/js/uniffi) compile unchanged — re-exports are additive only

## 3. Symbols case policy

- [x] 3.1 Updated `core/src/models/symbols.rs` module rustdoc with explicit case-sensitivity policy section
- [x] 3.2 Added unit tests `normalize_preserves_case` (`TXFB6`/`txfb6`/`TxFb6` retained as 3 entries) and `normalize_preserves_case_after_whitespace_trim`
- [ ] 3.3 ~~Update MIGRATION-0.5.md~~ _(skipped — MIGRATION docs target breaking changes; case policy is a docs-only clarification, covered by module rustdoc + CHANGELOG entry)_

## 4. Stability docs on FFI-load-bearing constructors

- [x] 4.1 Added `#[must_use]` + `## Stability` rustdoc to `ReconnectionConfig::disabled()`
- [ ] 4.2 ~~Add `#[must_use]` to `ReconnectionConfig::default()`~~ _(skipped — `default()` is via `Default` trait impl; `#[must_use]` on trait method bodies is not the conventional location, and the existing rustdoc on the struct already documents the 0.4.0 flip)_
- [x] 4.3 Added `#[must_use]` + `## Stability` rustdoc to `RetryPolicy::conservative()`
- [x] 4.4 Added `#[must_use]` + `## Stability` rustdoc to `RetryPolicy::aggressive()`
- [x] 4.5 `cargo doc --all-features -p fugle-marketdata-core` clean
- [x] 4.6 `cargo clippy --all-features -p fugle-marketdata-core -- -D warnings` clean (after `#[allow(clippy::too_many_arguments)]` on `dispatch_messages` — added the events_dropped param pushed it over the 7-arg threshold)

## 5. Idiomatic constructor section

- [x] 5.1 Added "## Which constructor should I use?" section to `core/README.md` between "Configuration" and "Error Handling"
- [x] 5.2 Section covers all four paths: bon builder, positional new(), typestate WebSocketFactory, convenience constructors
- [x] 5.3 All code samples tagged `rust,ignore` (5 blocks added)
- [x] 5.4 `cargo test --doc -p fugle-marketdata-core` — 62 passed, 17 ignored (5 new README blocks counted as ignored)

## 6. Release prep

- [x] 6.1 Bumped `core/Cargo.toml` 0.5.0 → 0.5.1
- [x] 6.2 Bumped `rust/Cargo.toml` 0.5.0 → 0.5.1
- [x] 6.3 Bumped workspace dep `marketdata-core` 0.5.0 → 0.5.1 in root `Cargo.toml`
- [x] 6.4 Added `[Rust 0.5.1]` section to `CHANGELOG.md` with `### Added` (source_kind + ErrorKind + RateLimit + events_dropped_total), `### Documentation` (case policy + idiomatic-constructor + stability sections), `### Internal` (emit_event signature)
- [x] 6.5 `cargo publish --dry-run -p fugle-marketdata-core --allow-dirty` clean (773 KB package)
- [ ] 6.6 Open PR; merge after CI green _(user action)_
- [ ] 6.7 Tag `v0.5.1`; publish to crates.io _(user action — irreversible)_
- [ ] 6.8 Archive change: `openspec archive core-0-5-1-followups` _(post-merge)_
