## Why

Monitor (P3 dual-probe, P4 incident classifier, P5 Prometheus exporter) consumes core 0.6.0 in earnest, and SDK users have surfaced five concrete gaps that block clean reuse. None of them justify a breaking release on their own, but they share a single audience (downstream observability builders) and bundle naturally as a 0.7.0 minor:

- `tracing_compat` macros (`debug!`/`info!`/`warn!`/`error!`) are `pub(crate)`, but the no-op fallback necessarily exports `__tracing_noop` at crate root via `#[macro_export]` — opaque to consumers and at risk of becoming a de-facto SDK API if a future `tracing` crate upgrade changes the macro shape.
- `WebSocketErrorKind::Http(u16)` correctly funnels every handshake status (401/403/404/429/5xx), but the **status → `ErrorKind`** mapping (Auth vs RateLimit vs Network vs Client) only lives in the doc-comment on `MarketDataError::source_kind`. Monitor's incident-rule code reads `WebSocketErrorKind::Http(_)` first; the table needs to live there too.
- `MockWsServer` in `core::testing` only accepts **one** client per instance — `run_accept_loop` calls `listener.accept().await` exactly once and exits. Monitor's dual-probe topology (one core client per endpoint) cannot be exercised by a single mock instance. Two mock instances on two ports works, but the spec doesn't promise either pattern.
- `MockWsServer::close(code, reason)` produces `DisconnectIntent::Server` (graceful Close frame). There is no way to inject `DisconnectIntent::Network` (transport-level drop / TCP RST). Monitor's incident classifier discriminates server-initiated vs network-initiated outages — half the test matrix is unreachable.
- `WebSocketClient::messages_dropped_total()` and `events_dropped_total()` are exposed as polling getters. Every consumer wires the same `gauge.set(client.messages_dropped_total())` boilerplate. With a `metrics`-feature integration the SDK can register `fugle_marketdata_ws_messages_dropped_total` / `fugle_marketdata_ws_events_dropped_total` counters automatically, and monitor's P5 Prometheus exporter inherits them for free.

## What Changes

- **`tracing_compat` hygiene (non-breaking)**:
  - Document `__tracing_noop` as an internal macro: confirm `#[doc(hidden)]` is present, and add a module-level note in `core/src/tracing_compat.rs` calling out that the macro is **NOT** part of the public API and may change shape on `tracing` major-version upgrades.
  - Add a `cargo public-api` regression test in CI to assert no new symbols leak from `tracing_compat` (re-exports, macros, types) without explicit acknowledgement.
- **`WebSocketErrorKind::Http(u16)` mapping table relocation (non-breaking, doc-only)**:
  - Move the full `Http(status) → ErrorKind` table (401/403 → Auth, 429 → RateLimit, 500..=599 → Network, other 4xx → Client) into the doc-comment on the `WebSocketErrorKind::Http` variant itself.
  - Cross-link from `MarketDataError::source_kind` (current location) to `WebSocketErrorKind::Http` so both entry points land at the same table.
  - Add a `#[cfg(test)]` table-vs-impl assertion (compile-time consistency check that the doc table matches `source_kind`'s match arms).
- **`MockWsServer` multi-client (non-breaking, additive)**:
  - `MockWsServer::start_with_capacity(n: usize)` constructor — accept-loop accepts up to `n` clients and dispatches injections per-client.
  - `MockWsServer::start()` retains current 1-client semantic for backwards compatibility.
  - `inject_frame` / `next_subscribe_id` / `close` gain a `_for(client_idx, ...)` overload (`inject_frame_for(client_idx, frame)`); the no-arg form keeps current behaviour and panics if called against a multi-client mock (per-client targeting required).
  - New helper `aio_pair_n(n: usize) -> (MockWsServer, Vec<WebSocketClient>)` for monitor's dual-probe tests.
- **`MockWsServer` transport-drop intent injection (non-breaking, additive)**:
  - `MockWsServer::drop_transport()` — close the underlying TCP socket without sending a Close frame, forcing the client into `DisconnectIntent::Network`.
  - `MockWsServer::drop_transport_for(client_idx)` for the multi-client variant.
  - Spec requirement: the mock MUST be able to inject all three `DisconnectIntent` outcomes (`Client` is reachable via the SDK's own `client.disconnect()` so the mock only needs to expose `Server` (existing `close`) and `Network` (new `drop_transport`)).
- **`metrics` feature (non-breaking, additive)**:
  - New `core/Cargo.toml` feature: `metrics = ["dep:metrics"]` (off by default, no transitive cost for consumers who don't opt in).
  - When enabled, `WebSocketClient::new(...)` registers two counters at construction:
    - `fugle_marketdata_ws_messages_dropped_total` (channel back-pressure drops)
    - `fugle_marketdata_ws_events_dropped_total` (event-bus back-pressure drops)
  - Counters update **in addition to** the existing in-process atomic counters — the polling getters remain available so consumers without `metrics-exporter-prometheus` are unaffected.
  - Counter labels: `endpoint` (ws URL host), `client_id` (caller-supplied via `ConnectionConfig::builder().client_id(...)` — new builder field, defaults to empty string when unset).
  - `MIGRATION-0.7.md` covers the opt-in pattern and the recommended `metrics-exporter-prometheus` wiring.
- **REST + WebSocket dual-host documentation (doc-only, no code change)**:
  - 0.6.0 already supports independent hosts for `RestClient::base_url(...)` and `WebSocketFactory::base_url(...)` — bindings (py / js / uniffi) likewise expose them as separate kwargs. No code path forces them to match.
  - 0.7.0 makes the pattern discoverable: `MIGRATION-0.7.md` adds a dedicated section showing how to keep REST on `https://api.fugle.tw/marketdata/v1.0` while routing WebSocket through a separate host (placeholder `your-ws-host.example.com` in published docs — never reference internal-only hosts).
  - `core/README.md` "Which constructor should I use?" section gains a one-paragraph "Independent endpoints" note cross-referencing the migration doc. No `documentation-policy` spec change needed; the existing requirement already mandates the section.

## Capabilities

### New Capabilities
- `metrics-export`: defines the `metrics` feature gate, the two counter names, label conventions, and the registration contract (what `WebSocketClient::new` MUST register and when).

### Modified Capabilities
- `testing-utilities`: extends the `MockWsServer` contract with multi-client support and transport-drop injection. Adds the requirement that the mock MUST be able to produce all three `DisconnectIntent` outcomes the SDK emits.
- `websocket-events`: clarifies the `WebSocketErrorKind::Http(u16)` → `ErrorKind` mapping and pins it to the variant-level doc rather than `source_kind`'s doc-comment alone. No behavioural change.
- `documentation-policy`: adds a CI lint requiring `cargo public-api` to track `tracing_compat`-adjacent symbols so future `tracing` upgrades don't silently expand the public surface.

## Impact

- **Code**: `core/src/tracing_compat.rs` (docs), `core/src/errors.rs` (doc-comment relocation + compile-time table assertion), `core/src/testing/mod.rs` (multi-client accept loop + `drop_transport`), `core/src/websocket/{aio,sync}/client.rs` (counter registration when `metrics` feature on), `core/Cargo.toml` (new feature, optional `metrics` dep).
- **Dependencies**: optional `metrics = "0.23"` (current stable as of 2026-05). No required new deps.
- **MSRV**: `metrics` 0.23 needs Rust 1.75; core's MSRV is 1.82 — no bump.
- **Bindings**: Python / JS / UniFFI bindings unaffected — feature is core-only and additive. Binding test crates can opt in to `test-utils` + multi-client to mirror monitor's coverage.
- **Migration**: zero breaking changes. `MIGRATION-0.7.md` is opt-in guidance only (how to enable `metrics` feature, how to use `aio_pair_n` and `drop_transport`).
- **CHANGELOG**: 0.7.0 entry calls out the four additive changes plus the doc/hygiene improvements.
