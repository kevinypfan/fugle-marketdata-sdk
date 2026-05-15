## 1. `bon` adoption — low-risk builders first

- [x] 1.1 Add `bon = "3"` to `core/Cargo.toml` `[dependencies]`
- [x] 1.2 Replace `RetryPolicy` hand-rolled builder with `#[derive(bon::Builder)]`; keep `conservative()` / `aggressive()` preset constructors as inherent methods _(additive: `new()` retained, `builder()` is new)_
- [x] 1.3 Update `RetryPolicy` tests to use the new builder shape (`RetryPolicy::builder()...build()`)
- [x] 1.4 Replace `ReconnectionConfig` hand-rolled builder with `#[derive(bon::Builder)]`; keep `disabled()` / `default()` preset constructors. Removed `with_*` chainable validators (replaced by bon's setters; validation remains on `new()`).
- [x] 1.5 Update `ReconnectionConfig` tests + `tests/reconnect_default.rs` CI gate
- [x] 1.6 `cargo test --all-features -p fugle-marketdata-core` — 421 passed
- [x] 1.7 Commit `refactor(core): adopt bon for RetryPolicy and ReconnectionConfig builders` (3b2eb5b)

## 2. `Symbols` module split + helpers

- [x] 2.1 Create `core/src/models/symbols.rs` with `pub enum Symbols { Single(String), Many(Vec<String>) }`
- [x] 2.2 Moved all 9 existing `From` impls into the new module, retargeting `SymbolSpec` → `Symbols`
- [x] 2.3 Implemented `Symbols::normalized()`: trim each entry, drop empty, dedup preserving first-seen order, collapse `Many` of len 1 to `Single`
- [x] 2.4 Implemented `Symbols::len()`, `Symbols::is_empty()`, `Symbols::iter()`
- [x] 2.5 Implemented `Symbols::chunked(max_per_chunk)`: validates non-zero (panics with clear message), splits preserving order
- [x] 2.6 Added `pub const SUBSCRIPTION_BATCH_LIMIT: Option<usize> = None;`
- [x] 2.7 13 unit tests cover all spec scenarios (Symbols normalization, chunking, len, dedup, whitespace, empty drop, Many→Single collapse)
- [x] 2.8 Re-exported `Symbols` from `core/src/models/mod.rs` (already re-exported at crate root); removed old `SymbolSpec` declaration from `subscription.rs`
- [x] 2.9 Mass-renamed `SymbolSpec` → `Symbols` across `core/` (6 files); FFI bindings unaffected
- [x] 2.10 Commit `refactor(core)!: rename SymbolSpec to Symbols, move to dedicated module` (b07c312)

## 3. Subscription dispatch dedup

- [x] 3.1 `SubscribeRequest::with_symbols` runs `.normalized()` on input
- [x] 3.2 `StockSubscription::new` / `FutOptSubscription::new` similarly normalize
- [x] 3.3 Added unit test `with_symbols_dedups_duplicates`
- [x] 3.4 Added unit test `with_symbols_collapses_whitespace_differences`
- [x] 3.5 Added unit test `with_symbols_keeps_distinct_in_insertion_order`
- [x] 3.6 Added `with_symbols_empty_input_yields_no_symbol_field` (replaces "no expand entries" — empty input produces a request with neither `symbol` nor `symbols` populated)
- [x] 3.7 Commit `feat(core)!: deduplicate symbols in subscription dispatch via Symbols::normalized` (3a3dbc7)

## 4. `bon` for ConnectionConfig + SubscribeRequest

- [ ] 4.1 ~~Add `#[derive(bon::Builder)]` to `ConnectionConfig`~~ _(skipped — bon's derived setters can't replicate the `assert!(cap > 0)` validation contract in the existing `websocket-config` spec; hand-rolled `ConnectionConfigBuilder` preserves it. Documented in 9cee45d commit message.)_
- [ ] 4.2 _(N/A — see 4.1; existing `ConnectionConfig::builder(url, auth)` call sites unchanged)_
- [ ] 4.3 _(N/A — see 4.1; all 5 `websocket-config` spec scenarios continue to pass)_
- [x] 4.4 Added `#[derive(bon::Builder)]` to `SubscribeRequest`; `with_symbols(channel, symbols)` kept hand-written as the dedup gate from Group 3
- [ ] 4.5 ~~Delete old `ConnectionConfigBuilder` boilerplate~~ _(skipped — see 4.1)_
- [x] 4.6 `cargo test --all-features -p fugle-marketdata-core --lib` — 439 passed
- [x] 4.7 Commit `refactor(core): bon::Builder for SubscribeRequest` (9cee45d)

## 5. Typestate `WebSocketFactory`

- [x] 5.1 Added zero-sized marker types `Unset` and `WithAuth(AuthRequest)` to `core/src/websocket/factory.rs`
- [x] 5.2 Parameterized `WebSocketFactory<S = Unset>` over the state type
- [x] 5.3 `WebSocketFactory::new() -> WebSocketFactory<Unset>` with default base URL = `urls::WS_BASE_ROOT`
- [x] 5.4 `auth(self, AuthRequest) -> WebSocketFactory<WithAuth>` available on `WebSocketFactory<Unset>`
- [x] 5.5 `stock()` / `futopt()` moved into `impl WebSocketFactory<WithAuth>` — only callable post-auth
- [x] 5.6 `base_url(...)` lives in `impl<S> WebSocketFactory<S>` so it composes in either order
- [x] 5.7 Added two module-level `compile_fail` doctests verifying `.stock()` and `.futopt()` are rejected without `.auth(...)`
- [x] 5.8 Updated all internal call sites: `WebSocketFactory::new(auth)` → `WebSocketFactory::new().auth(auth)`
- [x] 5.9 Spec scenarios in the change's `specs/websocket-config/spec.md` aligned with the typestate shape
- [x] 5.10 Full test suite + FFI binding builds pass (440 unit + 62 doc + 3 FFI crates)
- [x] 5.11 Commit `feat(core)!: typestate-enforced WebSocketFactory builder` (8b0f6a6)

## 6. Migration documentation

- [x] 6.1 Created `MIGRATION-0.5.md` covering: `SymbolSpec → Symbols` rename + `sed` recipe, `WebSocketFactory::new(auth) → new().auth(auth)`, dedup behavioral change, `ReconnectionConfig::with_*` removal, additive `bon` builders
- [x] 6.2 Linked migration guide in CHANGELOG (referenced by the 0.5.0 section header)
- [x] 6.3 Added `[Rust 0.5.0] - 2026-05-15` section to `CHANGELOG.md` with Breaking / Added / Internal subsections

## 7. Validation

- [x] 7.1 `cargo test --all-features -p fugle-marketdata-core --lib` — 440 passed
- [x] 7.2 `cargo check -p marketdata-py -p marketdata-js -p marketdata-uniffi` — all clean
- [x] 7.3 `cargo clippy --all-features -p fugle-marketdata-core -- -D warnings` — clean
- [ ] 7.4 Manual `examples/websocket_basic.rs` against staging _(deferred — requires API key)_
- [x] 7.5 `compile_fail` doctests verified working: 62 doc tests pass (was 60 in 0.4.1, +2 typestate guards)

## 8. Release prep

- [x] 8.1 Bumped workspace versions: `core/Cargo.toml` `0.4.1 → 0.5.0`, `rust/Cargo.toml` `0.4.1 → 0.5.0`, root `Cargo.toml` `marketdata-core` workspace dep `0.4.0 → 0.5.0`
- [x] 8.2 `cargo publish --dry-run -p fugle-marketdata-core --allow-dirty` — packaged successfully
- [ ] 8.3 Open PR; wait for CI green; merge to main _(user action)_
- [ ] 8.4 Tag `v0.5.0`; publish `fugle-marketdata-core` and `fugle-marketdata` to crates.io _(user action — irreversible)_
- [ ] 8.5 Archive this OpenSpec change: `openspec archive adopt-databento-patterns` _(post-merge)_
