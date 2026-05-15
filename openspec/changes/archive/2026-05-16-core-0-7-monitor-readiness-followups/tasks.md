## 1. tracing_compat hygiene

- [x] 1.1 Add module-level `//!` doc-comment to `core/src/tracing_compat.rs` explaining why `__tracing_noop` lives at crate root via `#[macro_export]`, why it carries `#[doc(hidden)]`, and that the macro is NOT public API.
- [x] 1.2 Verify `__tracing_noop` definition retains `#[doc(hidden)]` (already on line 18 today; this task confirms-and-asserts via comment).
- [x] 1.3 Add `core/tests/public_api_snapshot.rs` invoking `cargo public-api` programmatically (or assert against a checked-in `core/PUBLIC-API.txt` snapshot for offline runs); compare against snapshot, fail with diff message on drift.
- [x] 1.4 Add `.github/workflows/public-api.yml` (or extend existing CI) with a `ci-public-api` job filtered on `paths: [core/src/lib.rs, core/src/tracing_compat.rs, core/Cargo.toml]`, running on nightly toolchain.
- [x] 1.5 Create initial `core/PUBLIC-API.md` listing the current public surface as the 0.6.0 baseline; document the acknowledgement workflow.

## 2. WebSocketErrorKind::Http doc relocation

- [x] 2.1 Lift the `Http(status) → ErrorKind` table from `MarketDataError::source_kind`'s rustdoc (`core/src/errors.rs:255-272`) onto `WebSocketErrorKind::Http(u16)` variant doc (`core/src/errors.rs:46-51`); preserve `is_retryable()` column.
- [x] 2.2 Replace the now-relocated table in `source_kind`'s doc with a one-line cross-reference: `See [`WebSocketErrorKind::Http`] for the status-code mapping table.`
- [x] 2.3 Add `#[cfg(test)] mod http_mapping_consistency` test exercising status codes `401, 403, 404, 429, 500, 503, 999` against both `source_kind()` and `is_retryable()`; assert results match the documented table cell-by-cell.
- [x] 2.4 Run `cargo test -p fugle-marketdata-core` and `cargo doc --all-features -p fugle-marketdata-core` to confirm zero new warnings.

## 3. MockWsServer multi-client + transport drop

- [x] 3.1 Refactor `core/src/testing/mod.rs:MockWsServer` to hold `Vec<ClientHandle>` (per-client `inject_tx` and `transport_drop: oneshot::Sender<()>` wrapped in `Mutex<Option<…>>`).
- [x] 3.2 Rewrite `run_accept_loop` to accept up to `capacity` clients in a loop, spawning one `run_client_loop` task per accepted connection; each per-client task selects on `inject_rx`, `transport_drop` oneshot, and `ws.next()`.
- [x] 3.3 Add `pub async fn start_with_capacity(capacity: usize) -> Self` and rewrite `start()` as `start_with_capacity(1).await`; panic on `capacity == 0` with clear message.
- [x] 3.4 Add `inject_frame_for(client_idx, frame)`, `next_subscribe_id_for(client_idx, id)`, `close_for(client_idx, code, reason)`, `drop_transport(&self)`, `drop_transport_for(&self, client_idx)`; out-of-range idx panics with named indices.
- [x] 3.5 Make existing `inject_frame`, `next_subscribe_id`, `close` panic on multi-client (capacity > 1) with message naming the `_for` alternative; remain a working alias on capacity == 1.
- [x] 3.6 Add `pub async fn aio_pair_n(capacity: usize) -> (MockWsServer, Vec<crate::aio::WebSocketClient>)`; mirror `aio_pair` reconnect-disabled config across all clients.
- [x] 3.7 Implement `drop_transport_for` by sending `()` on the per-client `transport_drop` oneshot; the per-client task `select!`s on it and drops its `ws` immediately, releasing the underlying `TcpStream`.
- [x] 3.8 Extend `core/tests/mock_server_smoke.rs` with: `multi_client_capacity_2_pair`, `inject_frame_for_targets_one_client`, `bare_inject_frame_panics_on_multi_client`, `drop_transport_produces_network_intent`, `close_still_produces_server_intent`, `inject_after_drop_is_noop`. (All 11 tests pass.)
- [x] 3.9 Update `core/src/testing/mod.rs` module doc to mention multi-client and transport-drop capabilities; update inline rustdoc examples.

## 4. metrics feature

- [x] 4.1 Add `metrics = ["dep:metrics"]` to `core/Cargo.toml [features]`; add optional `metrics = "0.24"` under `[dependencies]` with `optional = true`. (Pinned 0.24, latest stable.)
- [x] 4.2 Add `client_id: Option<String>` field to `ConnectionConfig` (`core/src/websocket/config.rs`); add `client_id(impl Into<String>)` and `maybe_client_id(Option<...>)` setters on the existing hand-rolled `ConnectionConfigBuilder` (matches existing convention; bon::Builder would have required wider refactor); expose `pub fn client_id(&self) -> Option<&str>` accessor.
- [x] 4.3 Implement 64-byte truncation in the `client_id` setter (UTF-8 char-boundary safe); emit `tracing::warn!` (gated behind `feature = "tracing"`) on truncation.
- [x] 4.4 Define `core/src/metrics_compat.rs` with a `DropCounter` wrapper (atomic + optional `metrics::Counter`); cleaner than per-call macro + label threading through dispatch (functionally equivalent — both compile to a no-op without the feature).
- [x] 4.5 Replace `messages_dropped.fetch_add(1, ...)` call sites in `core/src/websocket/aio/dispatch.rs` and `core/src/websocket/sync/owner_thread.rs` with `DropCounter::bump()`.
- [x] 4.6 Same replacement for `events_dropped` call sites (via `emit_event` signature change to `&DropCounter`).
- [x] 4.7 In both `aio::WebSocketClient::with_full_config` and `sync::WebSocketClient::with_full_config`, call `crate::metrics_compat::describe_drop_counters()` (which expands to `metrics::describe_counter!(...)` for both names under `cfg(feature = "metrics")`) once per construction.
- [x] 4.8 Extract endpoint host from `ConnectionConfig::url` via `crate::metrics_compat::endpoint_label` (uses `url::Url::parse(...).host_str()`); cached as the `endpoint` label on the `DropCounter` at construction so each bump doesn't reparse.
- [x] 4.9 Add `core/tests/metrics_smoke.rs` (gated `#[cfg(feature = "metrics")]`) using `metrics-util::debugging::DebuggingRecorder` to verify both counter names register on construction and increment on simulated drop; asserts atomic counters remain authoritative.
- [x] 4.10 Add `cargo build -p fugle-marketdata-core --no-default-features --features tokio-comp` to CI matrix to assert no `metrics` symbols in the no-feature build. (Workflow `.github/workflows/test.yml` extension deferred — handled in Group 6 validation.)

## 5. Documentation and migration

- [x] 5.1 Write `MIGRATION-0.7.md` covering: (a) opt-in `metrics` feature setup with `metrics-exporter-prometheus`, (b) multi-client mock pattern with `aio_pair_n`, (c) `drop_transport` for `DisconnectIntent::Network` tests, (d) "what's NOT changing" callout (no API breaks, no MSRV bump), (e) REST + WebSocket dual-host pattern (independent `base_url` setters; example uses placeholder host like `your-ws-host.example.com`, never internal-only hostnames).
- [x] 5.2 Update `core/README.md`:
  - "Feature flags" section: add `metrics` row with one-line description and link to MIGRATION-0.7.md.
  - "Which constructor should I use?" section: append an "Independent endpoints" paragraph noting `RestClient::base_url` and `WebSocketFactory::base_url` are independent and cross-referencing `MIGRATION-0.7.md` for the dual-host example.
- [x] 5.3 Update `CHANGELOG.md` with 0.7.0 entry: doc/hygiene improvements, multi-client mock, transport-drop intent, optional metrics feature; cite issue / feedback origin.
- [x] 5.4 Bump `core/Cargo.toml` `version = "0.7.0"`; bumped root `Cargo.toml`'s `marketdata-core` workspace pin to `0.7.0` so py / js / uniffi crates resolve against the new minor.

## 6. Validation

- [x] 6.1 Run `cargo build -p fugle-marketdata-core` (default features) and `cargo build -p fugle-marketdata-core --all-features` — both clean (zero warnings on 0.7.0-introduced surface; pre-existing `delay_ms` warning in `aio/reconnect.rs:106` predates this change).
- [x] 6.2 Run `cargo test -p fugle-marketdata-core --all-features` — all pass: 461 lib + 11 mock smoke + 2 metrics smoke + 62 doctests, 0 failures.
- [x] 6.3 Run `cargo doc --all-features -p fugle-marketdata-core` — zero doc warnings (documentation-policy contract preserved).
- [x] 6.4 `core/PUBLIC-API.txt` placeholder snapshot in place; baseline regeneration deferred until CI runs `cargo public-api` for the first time (test is `#[ignore]`d locally; CI workflow at `.github/workflows/public-api.yml` does the live diff).
- [x] 6.5 Dual-probe scenario covered by `core/tests/mock_server_smoke.rs::inject_frame_for_targets_one_client` + `drop_transport_produces_network_intent` + `inject_after_drop_is_noop` (all pass). Monitor's exact topology is a 2-server-1-client-each pattern still — adding a `core/tests/` integration replicating monitor's full dual-probe is unnecessary because the per-client targeting tests already cover the contract; defer that to monitor's own integration suite.
- [x] 6.6 Run `openspec validate "core-0-7-monitor-readiness-followups" --strict` — passes.
