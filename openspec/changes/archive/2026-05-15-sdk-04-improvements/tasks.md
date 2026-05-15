## 1. Wave A — Additive (no breaking changes)

- [x] 1.1 Add `tracing` Cargo feature in `core/Cargo.toml`: `[features] tracing = ["dep:tracing"]` and `tracing = { version = "0.1", optional = true }` under `[dependencies]`
- [x] 1.2 Replace `eprintln!` at `core/src/websocket/connection_event.rs:69` with `tracing::warn!` gated behind `cfg(feature = "tracing")` (fall back to `eprintln!` when feature off, OR drop entirely if monitor counter from 1.7 covers it — finalize during impl)
- [x] 1.3 Replace `eprintln!` at `core/src/websocket/aio/runtime.rs:116` with `tracing::error!` (NOT a panic boundary — confirmed in design.md R5)
- [x] 1.4 Replace `eprintln!` at `core/src/websocket/aio/client.rs:578` with `tracing::error!`; keep `runtime.rs:25,39` as `eprintln!` (real panic boundaries)
- [x] 1.5 Add `tracing::debug!(bytes = N, "ws frame received")` at `core/src/websocket/aio/dispatch.rs:84-100` (text) and `:102-116` (binary)
- [x] 1.6 Add lifecycle `tracing::info!`/`warn!` events: connect-established, authenticated, reconnect-attempt (with `attempt`, `delay_ms`), heartbeat-timeout (with `elapsed_ms`)
- [x] 1.7 Add `#[tracing::instrument(target = "fugle_marketdata::ws", name = "ws.<op>")]` on `connect`, `subscribe`, `unsubscribe`, `disconnect` for both `aio::WebSocketClient` (`core/src/websocket/aio/client.rs`) and sync `WebSocketClient` (`core/src/websocket/sync/client.rs`)
- [x] 1.8 Add atomic `messages_dropped_total: AtomicU64` field on both clients; expose `pub fn messages_dropped_total(&self) -> u64`; increment at `dispatch.rs:90` and `:106` when `try_send` fails. Add parallel `events_dropped_total` for symmetry (per design Q4)
- [x] 1.9 Add `pub mod urls` (new file `core/src/urls.rs`) with `STOCK_WS`, `FUTOPT_WS`, `REST_BASE` constants; reroute `core/src/websocket/config.rs:65,82` and `core/src/rest/client.rs:62` to use them
- [x] 1.10 Hand-roll `impl Debug for Auth` in `core/src/rest/auth.rs` — drop `#[derive(Debug)]`, output `Auth::ApiKey(***)` etc
- [x] 1.11 Hand-roll `impl Debug for ConnectionConfig` in `core/src/websocket/config.rs` — drop `#[derive(Debug)]`, embed `Auth`'s redacted Debug, regex-redact known sensitive query params (`token`, `key`, `apikey`, `api_key`, `secret`, `password`) from `url` field
- [x] 1.12 Add `Auth::from_env() -> Result<Auth, MarketDataError>` in `core/src/rest/auth.rs` — probe `FUGLE_API_KEY` → `FUGLE_BEARER_TOKEN` → `FUGLE_SDK_TOKEN`, treat empty string as unset, return `MarketDataError::ConfigError` naming all three vars when none found
- [x] 1.13 Add `RetryPolicy { max_attempts, initial_backoff, max_backoff }` struct + `RetryPolicy::conservative()` / `RetryPolicy::aggressive()` factory functions in `core/src/rest/`; add `RestClient::with_retry(RetryPolicy)` builder method gated on `MarketDataError::is_retryable()`; exponential backoff with `[0, initial_backoff)` jitter
- [x] 1.14 Add `is_subscribed(&self, channel: &Channel, symbol: &str) -> bool` and `subscription_count(&self) -> usize` on both sync + aio clients, backed by existing `SubscriptionManager::contains()` (`subscription.rs:103`) and `count()` (`:109`)
- [x] 1.15 Add `ConnectionConfig::builder().message_buffer(usize)` and `.event_buffer(usize)` chainable methods; defaults 1024/1024 (matching current hardcodes); reject zero with builder-time error/panic; route into `aio/client.rs:113-114` and `sync/client.rs:67-68`
- [x] 1.16 Update `connection_event.rs:65-66` doc comment to reference configurable cap and `messages_dropped_total` counter as the operator signal (no longer hardcoded "1024")

## 2. Wave B — Atomic event refactor (single PR)

- [x] 2.1 Add `connection_id: Option<String>` to every variant of `ConnectionEvent` enum at `core/src/websocket/connection_event.rs:30`
- [x] 2.2 Add `intent: DisconnectIntent` field (where `DisconnectIntent` is `enum { Client, Server, Network }`) to `ConnectionEvent::Disconnected` and `ConnectionState::Closed`
- [x] 2.3 Update all ~21 `emit_event` call sites: `dispatch.rs ×4` (lines 62, 90/106 area, 132), `reconnect.rs ×6`, `aio/writer.rs ×1`, `sync/client.rs ×6`, `aio/client.rs` lifecycle emits — propagate `connection_id` from owning client and `intent` classification at disconnect points (Client = local-initiated, Server = peer Close frame, Network = transport error / heartbeat timeout / EOF without Close)
- [x] 2.4 Add `ConnectionConfig::builder(url, auth).id(impl Into<String>)` chainable method; store `connection_id: Option<String>` on the config; thread into client constructors
- [x] 2.5 Update `core/src/websocket/connection_event.rs` module doc to describe drop-newest backpressure policy + reference `messages_dropped_total`
- [x] 2.6 Remove `SubscribeRequest::trades`, `::candles`, `::books`, `::aggregates` at `core/src/models/subscription.rs:212-229`; migrate the 3 internal test callers at `subscription.rs:488,496,703` to `SubscribeRequest::new(Channel::*, symbol)` form
- [x] 2.7 Audit and fix `cargo test -p marketdata-core --doc` — pattern-matches in doctests (e.g. `aio/client.rs:248,250`) need `..` or explicit new-field destructuring
- [x] 2.8 Update `core/examples/websocket_basic.rs:59` `Disconnected { code, reason }` pattern → `Disconnected { code, reason, connection_id, intent }` or use `..`

## 3. Reconnect default flip (#2)

- [x] 3.1 Flip `core/src/websocket/reconnection.rs:46` — change `enabled: false` → `enabled: true` in `Default` impl; update line 23-26 doc comment to reflect new semantic
- [x] 3.2 Add unit test in `core/tests/reconnect_default.rs` asserting `assert_eq!(ReconnectionConfig::default().enabled, true)` and `assert_eq!(ReconnectionConfig::disabled().enabled, false)`

## 4. Graceful shutdown (#6)

- [x] 4.1 Rewrite `aio::WebSocketClient::disconnect()` (`core/src/websocket/aio/client.rs:517`) to apply default 5s drain timeout: send Close frame via writer, await tungstenite `ConnectionClosed` from read half, drain pending writer queue, then force close on timeout
- [x] 4.2 Add `aio::WebSocketClient::shutdown_with_timeout(&self, timeout: Duration)` exposing the same drain logic with caller-supplied timeout
- [x] 4.3 Mirror in sync side: rewrite `sync::WebSocketClient::disconnect()` (`sync/client.rs:210`); modify `owner_thread.rs:209,323` to drain `WRITE_QUEUE_CAPACITY` queue before sending Close frame; add 5s default + `shutdown_with_timeout`
- [x] 4.4 Ensure `disconnect()` and `shutdown_with_timeout(...)` both classify the resulting `Disconnected` event as `DisconnectIntent::Client` (Wave-B dependency)

## 5. Bindings audit + flip compensation (#2 cross-binding)

- [x] 5.1 Audit `uniffi/src/websocket.rs:95-100,378,443-474` — does the binding-side `ReconnectionConfig` wrapper struct have its own `enabled` field? If yes, ensure binding default is `false`. If no, add the field and explicit `ReconnectionConfig::disabled()` mapping in `to_core()`
- [x] 5.2 Grep `fugle-marketdata-python/` for `pyo3` reconnect surface; identify wrapper that constructs `ConnectionConfig` or `ReconnectionConfig`; ensure `disabled()` is called by default
- [x] 5.3 Grep `fugle-marketdata-node/` for `napi` reconnect surface; same compensation
- [x] 5.4 Audit `bindings/{go,java,cpp,csharp}/` for any reconnect surface; add `disabled()` calls where exposed
- [x] 5.5 Update each binding for new `ConnectionEvent` shape — at minimum add wildcard `..` in pattern matches at `uniffi/src/websocket.rs:443-474` so new fields don't break compilation; map `connection_id` and `intent` into FFI types where the wrapper exposes events
- [x] 5.6 Update each binding for `SubscribeRequest::{trades, candles, books, aggregates}` removal (if any binding called these)

## 6. Migration documentation

- [x] 6.1 Create `MIGRATION-0.4.md` at repo root mirroring `MIGRATION-0.3.md` structure; cover `ConnectionEvent` shape, `ConnectionState::Closed` shape, reconnect default flip, Auth/Config Debug change, `SubscribeRequest` removal, `disconnect()` semantics, tracing opt-in
- [x] 6.2 Update `rust/README.md:160-220` — refresh `ReconnectionConfig` examples and `SubscribeRequest` usage
- [x] 6.3 Update `core/CHANGELOG.md` with 0.4.0 entry listing the 4 breaking changes + 9 additive items

## 7. Test coverage

- [x] 7.1 New `core/tests/shutdown.rs` — `tokio::io::duplex` + `tokio_tungstenite::WebSocketStream::from_raw_socket` to assert Close frame appears on duplex sink within `shutdown_with_timeout`; sync-side equivalent using mpsc fixture
- [x] 7.2 New `core/tests/intent.rs` — three scenarios injecting Client / Server / Network close conditions, assert `DisconnectIntent` matches each
- [x] 7.3 New `core/tests/backpressure.rs` — construct `ConnectionConfig::builder().message_buffer(1)`, force-saturate, assert `messages_dropped_total > 0`; capture `tracing::warn!` via `tracing_test` if available, otherwise structural check
- [x] 7.4 Workspace-level binding test: assert `ReconnectionConfig::default().enabled == true` AND assert each binding's effective default is `false` (file path per binding TBD after task 5)
- [x] 7.5 Add unit tests for `Auth::from_env()` precedence (ApiKey > Bearer > Sdk), empty-string-as-unset, all-unset error
- [x] 7.6 Add unit tests for `Auth` and `ConnectionConfig` Debug redaction — assert raw secret never appears in `format!("{:?}", ...)` output
- [x] 7.7 Add unit tests for `is_subscribed()` + `subscription_count()` covering subscribe / unsubscribe / multi-pair scenarios
- [x] 7.8 Add unit tests for `RestClient::with_retry()` — default no-retry, success on second attempt, exhaustion preserves last error, non-retryable not retried (use `httpmock` or local `wiremock`)
- [x] 7.9 Run full matrix: `cargo test -p marketdata-core` (default), `--features tracing`, `--no-default-features` (sync only); `cargo test -p marketdata-core --doc`

## 8. Release

- [x] 8.1 Run `openspec validate sdk-04-improvements --strict`
- [x] 8.2 Bump `core/Cargo.toml` version to `0.4.0`
- [x] 8.3 Tag and publish: `cargo publish -p marketdata-core` (after CI green on all binding workspaces)
- [x] 8.4 Update `.planning/intel/` notes (if any) reflecting the binding compensation pattern for future core bumps  
  *(N/A — `.planning/intel/` does not exist in this repo; binding compensation pattern is documented in `MIGRATION-0.4.md` "Bindings" section and enforced by `core/tests/binding_compensation.rs`.)*
- [x] 8.5 Run `openspec archive sdk-04-improvements` after release lands
