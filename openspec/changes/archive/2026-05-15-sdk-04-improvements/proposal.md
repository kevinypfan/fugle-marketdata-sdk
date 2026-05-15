## Why

The `core/` Rust SDK at 0.3.x has zero internal `tracing`/`log` instrumentation, leaks API tokens via `Debug`, ships with `enabled: false` reconnect defaults that silently strand production users on the first network blip, and lacks several ergonomic surfaces the downstream `monitor` application needs (multi-connection event labels, REST retry, graceful shutdown, subscription introspection, configurable buffers). 0.4.0 bundles these into one atomic release with a `MIGRATION-0.4.md`, accepting some breaking changes in exchange for production-grade observability and safety before further consumers are onboarded.

## What Changes

- **Add** opt-in `tracing` feature: `tracing` crate behind `[features] tracing = ["dep:tracing"]`. Hot-path `debug!`, lifecycle `info!/warn!`, errors `warn!/error!`. `#[tracing::instrument]` on cold path (`connect`/`subscribe`/`unsubscribe`/`disconnect`) only.
- **Add** `RestClient::with_retry(RetryPolicy)` — opt-in exponential backoff + jitter, default off, reuses existing `MarketDataError::is_retryable()`.
- **Add** `Auth::from_env()` helper — tries `FUGLE_API_KEY` → `FUGLE_BEARER_TOKEN` → `FUGLE_SDK_TOKEN`.
- **Add** `pub mod urls` exposing both the canonical full endpoints (`STOCK_WS`, `FUTOPT_WS`, `REST_BASE`) and the host roots + version (`REST_BASE_ROOT`, `WS_BASE_ROOT`, `API_VERSION`) used to derive them — centralize endpoint strings and enable run-time URL composition.
- **Add** `WebSocketFactory` — JS / Python SDK-equivalent factory that takes one auth + optional shared base URL, exposes `.stock()` / `.futopt()` returning `ConnectionConfigBuilder`. Replaces the need for hand-formatting per-channel URLs when targeting staging, internal proxies, or local mock servers.
- **Add** `is_subscribed(channel, symbol) -> bool` + `subscription_count() -> usize` on both sync + aio clients.
- **Add** atomic `messages_dropped_total` counter on each client + getter; replaces `eprintln!` saturation warning with `tracing::warn!`.
- **Add** `ConnectionConfig::builder().message_buffer(usize)` + `.event_buffer(usize)`. Default `message_buffer` bumped from the pre-0.4.0 hardcoded 1024 to **4096** for multi-symbol consumers; `event_buffer` stays at 1024 (already wildly oversized for ~one-event-per-heartbeat lifecycle volume).
- **Add** `WebSocketClient::shutdown_with_timeout(Duration)`; `disconnect()` gains default 5s drain timeout.
- **BREAKING** `ReconnectionConfig::default().enabled` flips `false` → `true`. Bindings (`uniffi`, `python`, `node`, `bindings/{go,java,cpp,csharp}`) explicitly call `ReconnectionConfig::disabled()` so end-user behavior is preserved.
- **BREAKING** Hand-roll `impl Debug` on `Auth` and `ConnectionConfig` to redact tokens (`Auth::ApiKey(***)` etc). Drops `#[derive(Debug)]`.
- **BREAKING** `ConnectionEvent::Disconnected` gains `intent: DisconnectIntent { Client, Server, Network }`. `ConnectionState::Closed` mirrors the `intent` field for symmetry. (An earlier draft also added `connection_id: Option<String>` to every variant; that field was removed before release as YAGNI for the singleton-per-channel + `tokio::select!` consumer model — see `design.md` D4 rollback note.)
- **BREAKING** Remove legacy constructors `SubscribeRequest::{trades, candles, books, aggregates}` — callers move to canonical builder API.

## Capabilities

### New Capabilities
- `websocket-observability`: opt-in `tracing` feature, instrumentation policy (cold-path `#[instrument]`, hot-path `debug!`, lifecycle `info!/warn!`), and the `messages_dropped_total` backpressure counter.
- `rest-retry`: opt-in `RestClient::with_retry(RetryPolicy)` builder with exponential backoff + jitter, gated on `MarketDataError::is_retryable()`.
- `auth-handling`: `Auth::from_env()` helper plus secret-redacting `Debug` impls on `Auth` and `ConnectionConfig`.
- `websocket-events`: `ConnectionEvent` shape — every variant carries `connection_id: Option<String>`; `Disconnected` carries `intent: DisconnectIntent`. `ConnectionState::Closed` mirrors `intent`.
- `websocket-lifecycle`: `ReconnectionConfig::default().enabled = true` plus `disconnect()` drain semantics and `shutdown_with_timeout(Duration)`.
- `websocket-config`: `ConnectionConfig::builder()` adds `id(...)`, `message_buffer(usize)`, `event_buffer(usize)`; introduces `urls` module of endpoint constants.
- `subscription-api`: `is_subscribed(channel, symbol)` + `subscription_count()` query helpers; removal of legacy `SubscribeRequest::{trades, candles, books, aggregates}` constructors.

### Modified Capabilities
None — `openspec/specs/` is empty (this is the first OpenSpec change for the project), so all behavior is captured as new capabilities.

## Impact

- **Code**: `core/src/{websocket/{aio,sync,connection_event,reconnection,config}.rs, rest/{auth,client}.rs, errors.rs, urls.rs (new), models/subscription.rs}`, `core/Cargo.toml`.
- **Bindings**: `uniffi/src/websocket.rs:95-100,378,443-474` audit needed; `fugle-marketdata-python/`, `fugle-marketdata-node/`, `bindings/{go,java,cpp,csharp}/` per-binding reconnect-disabled wiring.
- **APIs**: 4 breaking surface changes in one release; mitigated by `MIGRATION-0.4.md` (mirrors existing `MIGRATION-0.3.md`) + `rust/README.md:160-220` updates + `core/examples/websocket_basic.rs:59` pattern-match fix.
- **Dependencies**: optional `tracing = "0.1"` (gated). No always-on additions.
- **Versioning**: core crate `0.3.x` → `0.4.0`; bindings stay on their 3.x line per existing decoupling policy.
- **Tests**: new `core/tests/{shutdown,intent,backpressure,reconnect_default}.rs`; existing doctests in `aio/client.rs:248,250` need pattern-match audit after `connection_id` propagation.
