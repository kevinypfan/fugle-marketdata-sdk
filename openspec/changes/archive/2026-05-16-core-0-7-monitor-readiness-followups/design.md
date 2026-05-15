## Context

Core 0.6.0 shipped on 2026-05-15 with `WebSocketErrorKind`, `MockWsServer`, and the OpenAI-style `base_url(...)` semantic. Monitor (this repo's `monitor` crate, building toward P5 Prometheus exporter) is the first non-binding consumer to integrate the new surface, and its review surfaced five distinct gaps:

1. **Macro hygiene** — `core/src/tracing_compat.rs:13` re-exports `tracing::{debug,info,warn,error}` as `pub(crate)`. The no-op fallback (lines 15-30) cannot use the same `pub(crate) macro_rules!` shape because `macro_export` is the only mechanism `macro_rules!` has for cross-module visibility. The current code declares `__tracing_noop` with `#[macro_export]` + `#[doc(hidden)]` and re-binds it under crate-private names — correct, but undocumented.
2. **Doc topology** — `MarketDataError::source_kind` (`core/src/errors.rs:255-272`) has the only authoritative `Http(status) → ErrorKind` table. A monitor incident-rule reading `match err { MarketDataError::WebSocketError { kind: WebSocketErrorKind::Http(s), .. } => ... }` lands at `WebSocketErrorKind::Http`'s 2-line doc-comment first; the table is one hop away.
3. **Mock cardinality** — `core/src/testing/mod.rs:132-211` runs a single accept-then-exit loop. Monitor needs to pair *one* mock with *two* clients (dual-probe pattern shared by stock + futopt endpoints) without spawning two ports.
4. **Mock fidelity** — `core::testing::MockWsServer::close` exits the dispatch loop after sending a Close frame (`core/src/testing/mod.rs:195-205`). The SDK classifies that as `DisconnectIntent::Server` (`core/src/websocket/aio/dispatch.rs:181`). To exercise `DisconnectIntent::Network`, the test would need to drop the underlying TCP socket without sending Close — currently impossible from the public mock API.
5. **Metrics ergonomics** — `WebSocketClient` exposes `messages_dropped_total()` / `events_dropped_total()` as polling getters. Every monitor-style consumer hand-rolls `gauge.set(client.messages_dropped_total())` on a tick. The Rust ecosystem has converged on the `metrics` crate (with `metrics-exporter-prometheus` as the de-facto Prometheus bridge), and the SDK can register counters on its own.

## Goals / Non-Goals

**Goals:**
- Land all 5 followups in a single 0.7.0 minor release (zero breaking changes) so consumers absorb the upgrade once.
- Promise the multi-client + transport-drop mock surface as part of `testing-utilities` so monitor and binding test crates can rely on it.
- Make Prometheus-style metrics export a one-liner (`features = ["metrics"]`) with no boilerplate at the call site.
- Keep the default build (`cargo build -p fugle-marketdata-core`) unchanged in features and dependencies — every addition is opt-in.

**Non-Goals:**
- Not adding `metrics-exporter-prometheus` as a transitive dependency. Consumers wire their own exporter; the SDK only records.
- Not changing `WebSocketErrorKind` variants or the `Http(u16) → ErrorKind` mapping behaviour. Doc relocation only.
- Not exposing a synchronous mock. `test-utils` remains async-only per the 0.6.0 design (`core/src/testing/mod.rs:31`).
- Not adding async-std / smol metrics support. The `metrics` crate is runtime-agnostic, but counter registration runs on the construction thread regardless of runtime.

## Decisions

### D1: `tracing_compat` documents internal macro export, no API change

The `__tracing_noop` macro is exported at crate root through `#[macro_export]` because `macro_rules!` has no way to declare module-private macros that other modules can `use`. Removing the macro export would force every call site to switch to function-style logging (loses the lazy-formatting property of `tracing` macros) or to a `cfg(feature = "tracing")` ladder at every call site (the exact problem `tracing_compat` was created to solve).

**Choice**: Keep the macro export. Strengthen the documentation:
- Add a `//!` module-level note explaining why `__tracing_noop` exists at crate root and that callers MUST NOT use it.
- Confirm `#[doc(hidden)]` is in place (already present on line 18).
- Add a `cargo public-api` snapshot test under `core/tests/public_api_snapshot.rs` so future `tracing` upgrades that add re-exports trip CI rather than silently expanding the public surface.

**Alternatives considered**:
- `pub(crate) macro_rules! __tracing_noop` — invalid syntax. `macro_rules!` only supports `#[macro_export]` (crate-root pub) or no annotation (single-module scope).
- Replace macros with `#[inline] fn debug(...)` — kills lazy-formatting and `tracing::Span` propagation; the upstream `tracing` crate uses macros for this exact reason.
- Use `tracing-core` directly — same macro shape, same problem.

### D2: `WebSocketErrorKind::Http` doc-comment owns the mapping table

Move the `Http(status) → ErrorKind` table from `MarketDataError::source_kind`'s rustdoc to `WebSocketErrorKind::Http`'s variant-level rustdoc. Keep a one-line cross-reference in `source_kind` pointing at `WebSocketErrorKind::Http`. The new table on the variant lists every status family with its `ErrorKind` and `is_retryable()` verdict in one grid.

Add a `#[cfg(test)]` consistency assertion in `core/src/errors.rs` that constructs `MarketDataError::WebSocketError { kind: WebSocketErrorKind::Http(s), .. }` for representative status codes (401, 403, 404, 429, 500, 503, 999) and asserts `source_kind()` and `is_retryable()` match the documented table. Doc drift now causes a test failure.

**Alternatives considered**:
- Generate a markdown doc page from a const table — overkill for a 6-row table; rustdoc-on-variant is more discoverable from IDEs.
- Leave the table on `source_kind` only — perpetuates the discoverability problem the user reported.

### D3: `MockWsServer` accept-loop refactor for capacity-N

The single-accept loop becomes a multi-accept loop bounded by `capacity`. Per-client state moves into a `Vec<ClientHandle>` indexed by accept order:

```text
ClientHandle {
    inject_tx: mpsc::UnboundedSender<MockInjection>,
    transport_drop: oneshot::Sender<()>,
}
```

The accept loop spawns one `run_client_loop` task per accepted connection; the per-client task selects on `inject_rx`, `transport_drop_rx`, and `ws.next()`. `start()` becomes a thin wrapper for `start_with_capacity(1)` so existing tests are unchanged.

Targeting becomes explicit:
- `inject_frame(frame)` panics on multi-client (capacity > 1).
- `inject_frame_for(client_idx, frame)` works on both; idx ≥ capacity panics.

**Alternatives considered**:
- Keep one task and round-robin — simpler but kills any test that needs frame ordering per client.
- Use one mock per client — works today but doubles the port footprint and forces tests to coordinate two URLs. Doesn't satisfy the "one mock instance, N clients" use case the spec wants to promise.

### D4: `drop_transport` shuts the underlying TCP socket without WS Close

`MockWsServer::drop_transport()` (and `_for(client_idx)` for multi-client) signals the per-client task via the `transport_drop` oneshot. The task immediately drops the `ws` value, which drops the inner `TcpStream` — the kernel sends FIN/RST and the client's read returns `tungstenite::Error::Io(ConnectionReset)`. The 0.6.0 dispatch path (`core/src/websocket/aio/dispatch.rs:96`) already classifies that as `DisconnectIntent::Network`, so no SDK change is required.

**Alternatives considered**:
- Send an Abort frame — there is no such thing in WS; the closest is a malformed close which would route to `DisconnectIntent::Server`.
- Inject latency until heartbeat fires — produces `HeartbeatTimeout` (a different error path), not `DisconnectIntent::Network`.

### D5: `metrics` feature wires `metrics::counter!` at counter-bump sites

When `feature = "metrics"` is enabled, the existing `messages_dropped_total` / `events_dropped_total` increments call `metrics::counter!("fugle_marketdata_ws_messages_dropped_total", "endpoint" => host, "client_id" => id).increment(1)` in addition to the atomic bump. Counter registration happens at first increment (the `metrics` crate auto-registers; explicit `describe_counter!` runs once at `WebSocketClient::new`).

The two counter call sites (one per metric, one per client variant — sync and aio) become a single `track_drop!(metric, client)` macro that expands to `self.messages_dropped_total.fetch_add(1, Ordering::Relaxed)` always, plus `metrics::counter!(...).increment(1)` under `cfg(feature = "metrics")`. Polling getters keep reading the atomic — no behavioural change for consumers without the feature.

`ConnectionConfig` gains a `client_id: Option<String>` builder field for the label; default is `None` which renders as the empty string in the counter label.

**Alternatives considered**:
- Histogram of drop rate over time — the `metrics` ecosystem prefers `monotonic counter + Prometheus rate()` over SDK-side rate calculation; matches Grafana convention.
- Gauge instead of counter — gauges go up and down; drops only go up. Counters are the right primitive.
- Bake `metrics-exporter-prometheus` in — forces a transitive dep that consumers might not want (they may use OTLP, statsd, or a custom recorder).

## Risks / Trade-offs

- **`metrics` crate API stability** → `metrics` 0.23 is the current stable; the 0.x → 0.x bumps have historically been minor. Pin to a `^0.23` range and re-test on each upgrade. → Mitigation: integration test under `core/tests/metrics_smoke.rs` exercising `register_counter` + `increment` + read-back via `metrics-util::debugging::DebuggingRecorder`.
- **Multi-client mock test flakiness** → tokio task scheduling is non-deterministic; a test asserting "client 0 sees frame X before client 1 sees frame Y" would be racy. → Mitigation: spec says per-client targeting only. Frame ordering across clients is undefined; tests MUST coordinate via per-client `inject_frame_for` calls and per-client assertion channels.
- **`drop_transport` and `close` interleave** → If a test calls `close()` then `drop_transport()`, the second is a no-op (socket already gone). → Mitigation: document idempotent-after-close behaviour; both methods return `Result<(), MockClosed>` in the spec.
- **`cargo public-api` CI cost** → The tool is slow on cold caches (~30s) and requires a nightly toolchain. → Mitigation: gate behind a `ci-public-api` job that runs only on PRs touching `core/src/lib.rs` or `core/src/tracing_compat.rs`; not on every commit.
- **Counter cardinality blow-up** → If consumers pass a high-cardinality `client_id` (e.g., per-request UUID), Prometheus storage explodes. → Mitigation: doc-comment on `ConnectionConfig::builder().client_id(...)` warns "low-cardinality identifier (deployment / instance, not per-request)"; cap label length at 64 chars in the SDK with a `tracing::warn!` if exceeded.

## Migration Plan

- **0.6.x → 0.7.0**: zero breaking changes. Existing call sites compile unchanged. To opt in:
  1. `metrics` export: add `features = ["metrics"]` to the core dependency and wire any `metrics::Recorder` (typically `metrics-exporter-prometheus`).
  2. Multi-client mock: replace `MockWsServer::start()` with `MockWsServer::start_with_capacity(2)` and `inject_frame(...)` calls with `inject_frame_for(idx, ...)`.
  3. Transport-drop tests: call `mock.drop_transport().await` where the test currently calls `mock.close(1011, "...").await` and asserts `DisconnectIntent::Server`.
- **MIGRATION-0.7.md**: covers the three opt-in patterns above plus a "what's NOT changing" callout (no `WebSocketErrorKind` variants added, no `base_url` semantic shift, no MSRV bump).
- **Rollback**: feature is opt-in, so disabling `features = ["metrics"]` removes the integration; reverting to `start()` from `start_with_capacity(N)` removes multi-client behaviour. No data-shape migrations.

## Open Questions

- Should `metrics` counter names include a `_total` suffix per Prometheus convention even though the `metrics` crate's `metrics-exporter-prometheus` adds it automatically? **Answer**: yes — name the counter `..._total` in core; `metrics-exporter-prometheus` is smart enough not to double-suffix. Other recorders (statsd, OTLP) get the conventional name regardless.
- Should `drop_transport` be on a per-client `ClientHandle` returned from `aio_pair_n`, or kept as `&self` methods on `MockWsServer`? **Decision**: stay on `MockWsServer` with `_for(idx)` overloads. Matches the existing `inject_frame_for` pattern; avoids forcing tests to thread two values through scope.
- Do we need a `metrics-tracing-context` integration so `client_id` can come from a `tracing::Span` rather than `ConnectionConfig`? **Decision**: defer to 0.8.0. Adds complexity (transitive dep, span-fetch overhead per increment) and the explicit `client_id` builder field covers monitor's needs.
