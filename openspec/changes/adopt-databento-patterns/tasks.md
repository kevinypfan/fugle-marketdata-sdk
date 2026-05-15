## 1. `bon` adoption — low-risk builders first

- [ ] 1.1 Add `bon = "3"` to `core/Cargo.toml` `[dependencies]`
- [ ] 1.2 Replace `RetryPolicy` hand-rolled builder with `#[derive(bon::Builder)]`; keep `conservative()` / `aggressive()` preset constructors as inherent methods
- [ ] 1.3 Update `RetryPolicy` tests to use the new builder shape (`RetryPolicy::builder()...build()`)
- [ ] 1.4 Replace `ReconnectionConfig` hand-rolled builder with `#[derive(bon::Builder)]`; keep `disabled()` / `default()` preset constructors
- [ ] 1.5 Update `ReconnectionConfig` tests + `tests/reconnect_default.rs` CI gate
- [ ] 1.6 Run `cargo test --all-features -p fugle-marketdata-core` and verify green
- [ ] 1.7 Commit `refactor(core): adopt bon for RetryPolicy and ReconnectionConfig builders`

## 2. `Symbols` module split + helpers

- [ ] 2.1 Create `core/src/websocket/models/symbols.rs` with `pub enum Symbols { Single(String), Many(Vec<String>) }`
- [ ] 2.2 Move all 9 existing `From` impls from `core/src/websocket/models/subscription.rs` into the new module, retargeting `SymbolSpec` → `Symbols`
- [ ] 2.3 Implement `Symbols::normalized()`: trim each entry, drop empty, dedup preserving first-seen order, collapse `Many` of len 1 to `Single`
- [ ] 2.4 Implement `Symbols::len()`, `Symbols::is_empty()`, `Symbols::iter()`
- [ ] 2.5 Implement `Symbols::chunked(max_per_chunk)`: validate non-zero (panic if zero with clear message), split preserving order
- [ ] 2.6 Add `pub const SUBSCRIPTION_BATCH_LIMIT: Option<usize> = None;` to the module
- [ ] 2.7 Add unit tests covering all 9 scenarios from `specs/subscription-api/spec.md` (Symbols normalization, chunking, len, dedup, whitespace, empty drop, Many→Single collapse)
- [ ] 2.8 Re-export `Symbols` from `core/src/websocket/models/mod.rs` and `core/src/lib.rs`; remove old `SymbolSpec` declaration from `subscription.rs`
- [ ] 2.9 Search workspace for `SymbolSpec` references; rename to `Symbols` (in `core/`, `py/`, `js/`, `uniffi/`, and tests)
- [ ] 2.10 Commit `refactor(core)!: rename SymbolSpec to Symbols, move to dedicated module`

## 3. Subscription dispatch dedup

- [ ] 3.1 Modify `SubscribeRequest::with_symbols(channel, symbols)` to call `.normalized()` on the input before producing the request
- [ ] 3.2 Modify `StockSubscription::new(channel, symbols)` and `FutOptSubscription::new(channel, symbols)` similarly
- [ ] 3.3 Add integration test in `core/tests/integration_websocket.rs` covering "duplicate symbols collapse to one subscription"
- [ ] 3.4 Add integration test covering "whitespace-only differences collapse"
- [ ] 3.5 Add integration test covering "distinct symbols all retained in insertion order"
- [ ] 3.6 Verify `SubscriptionManager` count assertions still pass
- [ ] 3.7 Commit `feat(core)!: deduplicate symbols in subscription dispatch via Symbols::normalized`

## 4. `bon` for ConnectionConfig + SubscribeRequest

- [ ] 4.1 Add `#[derive(bon::Builder)]` to `ConnectionConfig`; configure defaults (`message_buffer = 4096`, `event_buffer = 1024`) via `#[builder(default = ...)]` attributes
- [ ] 4.2 Verify `ConnectionConfig::builder(url, auth)` call sites still work (keep thin wrapper if signature differs)
- [ ] 4.3 Update `websocket-config` spec scenarios — verify all 5 existing scenarios still pass post-refactor
- [ ] 4.4 Add `#[derive(bon::Builder)]` to `SubscribeRequest`; keep `with_symbols(channel, symbols)` hand-written (it's the dedup gate from step 3.1)
- [ ] 4.5 Delete old `ConnectionConfigBuilder` and `SubscribeRequest` builder boilerplate
- [ ] 4.6 Run `cargo test --all-features -p fugle-marketdata-core` and verify green
- [ ] 4.7 Commit `refactor(core): bon::Builder for ConnectionConfig and SubscribeRequest`

## 5. Typestate `WebSocketFactory`

- [ ] 5.1 Add zero-sized marker types `Unset` and `WithAuth(AuthRequest)` to `core/src/websocket/factory.rs`
- [ ] 5.2 Parameterize `WebSocketFactory<S = Unset>` over the state type; `state: S` field stores the marker
- [ ] 5.3 Implement `WebSocketFactory::new() -> WebSocketFactory<Unset>` with default base URL = `urls::WS_BASE_ROOT`
- [ ] 5.4 Implement `auth(self, auth: AuthRequest) -> WebSocketFactory<WithAuth>` on `WebSocketFactory<Unset>`
- [ ] 5.5 Move `stock()` / `futopt()` into `impl WebSocketFactory<WithAuth>` so they're only callable post-auth
- [ ] 5.6 Keep `base_url(...)` as a `impl<S> WebSocketFactory<S>` method so it works in any state
- [ ] 5.7 Add doctest verifying `WebSocketFactory::new().stock()` fails to compile (use `compile_fail` annotation)
- [ ] 5.8 Update all internal call sites: `WebSocketFactory::new(auth)` → `WebSocketFactory::new().auth(auth)` (examples, tests, integration tests)
- [ ] 5.9 Update spec scenarios in `openspec/specs/websocket-config/spec.md` (post-archive); for now align change spec scenarios
- [ ] 5.10 Run full test suite + FFI binding builds
- [ ] 5.11 Commit `feat(core)!: typestate-enforced WebSocketFactory builder`

## 6. Migration documentation

- [ ] 6.1 Create `docs/MIGRATION-0.5.md` covering:
  - `SymbolSpec → Symbols` rename (with `sed` recipe)
  - `WebSocketFactory::new(auth)` → `WebSocketFactory::new().auth(auth)` migration
  - Dedup behavioral change (note that duplicate symbols now collapse)
  - `bon::Builder` API parity (same method names, same defaults)
- [ ] 6.2 Add link to migration guide in `core/README.md`
- [ ] 6.3 Update `CHANGELOG.md` with 0.5.0 section marking BREAKING changes per `Migration Plan` in `design.md`

## 7. Validation

- [ ] 7.1 Run `cargo test --all-features -p fugle-marketdata-core`
- [ ] 7.2 Run `cargo build -p fugle-marketdata-py -p fugle-marketdata-js -p fugle-marketdata-uniffi` to confirm FFI bindings compile unchanged
- [ ] 7.3 Run `cargo clippy --all-features --workspace -- -D warnings`
- [ ] 7.4 Manually verify example `core/examples/websocket_basic.rs` compiles and runs against staging if available
- [ ] 7.5 Verify `compile_fail` doctest from 5.7 still fails (regression guard for typestate)

## 8. Release prep

- [ ] 8.1 Bump workspace version to 0.5.0 in relevant `Cargo.toml` files
- [ ] 8.2 Tag-test: `cargo publish --dry-run -p fugle-marketdata-core`
- [ ] 8.3 Open PR; wait for CI green; merge to main
- [ ] 8.4 Tag `v0.5.0`; publish `fugle-marketdata-core` and `fugle-marketdata` to crates.io
- [ ] 8.5 Archive this OpenSpec change once merged: `openspec archive adopt-databento-patterns`
