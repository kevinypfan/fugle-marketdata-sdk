## 1. WebSocketErrorKind structured classification

- [ ] 1.1 Add `pub enum WebSocketErrorKind` with `#[non_exhaustive]` to `core/src/errors.rs` (variants: Protocol, Capacity, Utf8, Tls, Io, Http(u16), Other)
- [ ] 1.2 Reshape `MarketDataError::WebSocketError` from `{ msg: String }` to `{ kind: WebSocketErrorKind, msg: String }`
- [ ] 1.3 Rewire `From<tungstenite::Error> for MarketDataError` to map each upstream variant to the correct `WebSocketErrorKind`
- [ ] 1.4 Update `MarketDataError::is_retryable()` — Protocol/Capacity/Utf8/Tls/Http(401|403) non-retryable; Io/Other/Http(429|5xx) retryable; Http(other) non-retryable
- [ ] 1.5 Update `MarketDataError::source_kind()` mapping (per the modified error-classification spec)
- [ ] 1.6 Update all internal `From` conversions and `WebSocketError` construction sites in `core/src/websocket/**/*.rs`
- [ ] 1.7 Update tests in `errors.rs` and `websocket/aio/client.rs` / `websocket/sync/client.rs` for the new shape
- [ ] 1.8 Re-export `WebSocketErrorKind` from `core/src/lib.rs`
- [ ] 1.9 Run full test suite; verify all assertions on `WebSocketError` matching now include `kind`

## 2. WebSocketFactory::base_url semantic shift (OpenAI alignment)

- [ ] 2.1 Refactor `WebSocketFactory::endpoint_for(kind: &str)` in `core/src/websocket/factory.rs`: when `base_url` is `None`, fall back to `urls::STOCK_WS` / `urls::FUTOPT_WS` directly (don't reconstruct from `WS_BASE_ROOT + API_VERSION`); when `base_url` is `Some(base)`, return `format!("{}/{}/streaming", base.trim_end_matches('/'), kind)` (no `/{API_VERSION}` injection)
- [ ] 2.2 Update the existing 7 factory unit tests: every test that previously passed a host-root string must now include `/v1.0` in its expected URL fixtures
- [ ] 2.3 Add the 0.5.x-caller regression test from `websocket-config/spec.md` ("0.5.x host-root caller produces non-canonical URL")
- [ ] 2.4 Add the "different API version segment is honored" scenario test (passing `/v2.0` and asserting the SDK does NOT force it back to `/v1.0`)
- [ ] 2.5 Update module-level doctest in `factory.rs` example to use `.base_url("wss://staging.fugle.tw/marketdata/v1.0")` (include version segment)
- [ ] 2.6 Update `core/src/urls.rs` module doc that references `WebSocketFactory::base_url` to clarify the OpenAI-style "full prefix" semantic

## 3. core::testing module with MockWsServer

- [ ] 3.1 Add `test-utils = ["tokio-comp", "dep:tokio-tungstenite", "dep:futures-util"]` feature to `core/Cargo.toml`
- [ ] 3.2 Create `core/src/testing/mod.rs` with module-level docs explaining feature gate and intended use
- [ ] 3.3 Implement `MockWsServer::start() -> Self` — bind ephemeral 127.0.0.1 port, spawn accept loop
- [ ] 3.4 Implement `MockWsServer::url(&self) -> String` and `MockWsServer::address(&self) -> SocketAddr`
- [ ] 3.5 Implement subscribe-ACK handler that speaks the same wire protocol as `core/src/websocket/protocol.rs`
- [ ] 3.6 Implement `next_subscribe_id(id)` — server-side queue of pre-assigned IDs for the next ACKs
- [ ] 3.7 Implement `inject_frame(&self, frame: StreamMessage)` — server pushes a frame to the connected client
- [ ] 3.8 Implement `close(&self, code, reason)` — server-initiated WebSocket Close
- [ ] 3.9 Implement `aio_pair() -> (MockWsServer, aio::WebSocketClient)` convenience constructor
- [ ] 3.10 Gate the module with `#[cfg(all(feature = "test-utils", feature = "tokio-comp"))]` and re-export from `core/src/lib.rs`
- [ ] 3.11 Add `tests/mock_server_smoke.rs` covering all 5 spec scenarios for MockWsServer
- [ ] 3.12 Verify production build (`cargo build -p fugle-marketdata-core` without `test-utils`) does not include the module or pull `tokio-tungstenite` beyond what `tokio-comp` already does

## 4. Migration documentation

- [ ] 4.1 Create `MIGRATION-0.6.md` with sections in this order: (1) `### SILENT BREAKING: WebSocketFactory::base_url semantic` — most prominent placement with before/after URL table + sed recipe; (2) `WebSocketError` reshape with before/after `match` examples + retry-verdict diff table; (3) `test-utils` feature additive note
- [ ] 4.2 Include explicit one-liner advice: "If your code passes a custom `base_url` to `WebSocketFactory`, append `/v1.0` (or the version segment you want) to the string"
- [ ] 4.3 Link prominently from `core/README.md` and from CHANGELOG entry
- [ ] 4.4 Tag CHANGELOG breaking entry with `[SILENT]` so release-notes scanners flag it

## 5. Validation

- [ ] 5.1 `cargo test --all-features -p fugle-marketdata-core --lib` clean (including new WebSocketErrorKind tests + 4 new factory scenarios)
- [ ] 5.2 `cargo test --all-features -p fugle-marketdata-core --tests` clean (including new mock server smoke test)
- [ ] 5.3 `cargo test --doc -p fugle-marketdata-core --all-features` clean (including updated factory doctest)
- [ ] 5.4 `cargo build -p fugle-marketdata-core` (default features) — verify testing module + tokio-tungstenite are NOT pulled in
- [ ] 5.5 `cargo build --features test-utils -p fugle-marketdata-core` — verify testing module compiles
- [ ] 5.6 `cargo check -p marketdata-py -p marketdata-js -p marketdata-uniffi` — FFI bindings compile post-reshape
- [ ] 5.7 `cargo clippy --all-features -p fugle-marketdata-core -- -D warnings` clean
- [ ] 5.8 Manual smoke: write an integration test that uses `WebSocketFactory::new().auth(a).base_url("wss://api.fugle.tw/marketdata/v1.0").stock().build()` and assert URL is correct; remove after release

## 6. Release prep

- [ ] 6.1 Bump `core/Cargo.toml` 0.5.1 → 0.6.0
- [ ] 6.2 Bump `rust/Cargo.toml` 0.5.1 → 0.6.0
- [ ] 6.3 Bump workspace dep `marketdata-core` 0.5.1 → 0.6.0 in root `Cargo.toml`
- [ ] 6.4 Add `[Rust 0.6.0]` section to `CHANGELOG.md` with: `### Breaking (SILENT)` for `WebSocketFactory::base_url` shift (prominent placement, before/after URL row); `### Breaking` for `WebSocketError` reshape; `### Added` for `WebSocketErrorKind`, `test-utils` feature, `MockWsServer`; `### Migration` linking MIGRATION-0.6.md
- [ ] 6.5 `cargo publish --dry-run -p fugle-marketdata-core --allow-dirty` clean
- [ ] 6.6 Open PR; merge after CI green
- [ ] 6.7 Tag `v0.6.0`; publish `fugle-marketdata-core` and `fugle-marketdata` to crates.io
- [ ] 6.8 Archive change: `openspec archive core-0-6-error-kind-and-mock`
- [ ] 6.9 Update FFI binding test crates (py/js/uniffi) to consume `features = ["test-utils"]` and drop their hand-rolled echo servers (follow-up PR, not blocking 0.6.0 release)
