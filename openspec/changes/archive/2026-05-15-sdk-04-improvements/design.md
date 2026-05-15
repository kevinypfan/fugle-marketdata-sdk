## Context

The `core/` Rust SDK shipped 0.3.0 with a redis-rs-style sync-default + tokio-comp feature pivot. Subsequent review surfaced 14 production-readiness gaps before the downstream `monitor` application can launch:

- Zero internal `tracing` / `log` instrumentation; debug-in-prod is a black box.
- `eprintln!` used in 5 sites for warnings and errors, including saturation signals.
- `Auth` and `ConnectionConfig` derive `Debug` plain — any `tracing::debug!(?config)` from a downstream user leaks raw API keys to logs.
- `ReconnectionConfig::default().enabled = false` — surprising default; first cable pull strands the user silently.
- REST client offers `MarketDataError::is_retryable()` (errors.rs:134) but no retry policy wiring.
- `ConnectionEvent` carries no source identifier; multi-connection consumers must wrap.
- `disconnect()` does not await drain or Close ACK — incompatible with k8s/systemd SIGTERM grace windows.
- Channel buffer caps (1024) hardcoded; not tunable in production.
- Subscription introspection limited to `subscriptions() -> Vec<SubscribeRequest>`; no membership or count helper.
- Endpoint URL strings inlined at three sites; v2.0 endpoint switch requires grep.
- `Auth::from_env()` boilerplate users keep rewriting.
- `Disconnected` event carries `code, reason` but no programmatic way to distinguish client/server/network origin.
- `SubscribeRequest::trades/candles/books/aggregates` constructors dangling after 0.2.0 re-export removal.

Rust crate version policy (per memory note): `core` crate at `0.x` track, decoupled from binding `3.x`. `0.4.0` is the next core minor; bindings stay on their respective lines, with binding-side compensation for the breaking changes in this release.

## Goals / Non-Goals

**Goals:**
- Production observability via opt-in `tracing` integration without forcing always-on dependency cost.
- Eliminate token leakage from any `Debug`-derived path.
- Align reconnect default with mainstream Rust ecosystem (reqwest / redis-rs / tokio-tungstenite).
- Make graceful shutdown actually graceful — drain writes, await Close ACK, capped by timeout.
- Add minimal multi-connection ergonomics (`connection_id`) and disconnect classification (`DisconnectIntent`) before more downstream consumers are onboarded.
- Bundle 13 items in one atomic 0.4.0 to amortize migration pain across a single `MIGRATION-0.4.md`.

**Non-Goals:**
- Mock/echo server test utilities (item #10 — deferred to 0.5.0; would significantly enlarge surface).
- Always-on `tracing` (rejected — adds ~100KB to consumers who don't subscribe).
- Always-on REST retry (rejected — observability use-case wants real failure visibility).
- Drop-oldest backpressure or `tokio::sync::broadcast` migration (would break current `events()` / `state_events()` API shape).
- Deprecation cycle for `SubscribeRequest::trades` etc. (only 3 internal test callers; not worth a 0.3.x release nobody asked for).
- Changes to runtime selection beyond `tokio-comp` (still no `async-std-comp` / `smol-comp`).

## Decisions

### D1 — Tracing as feature flag, not always-on
**Choice**: `[features] tracing = ["dep:tracing"]`. Spans only on cold path (`connect`/`subscribe`/`unsubscribe`/`disconnect`). Hot path uses `debug!` events (no span).
**Why X over Y**: Always-on `tracing` adds dependency weight to consumers who never install a subscriber. Always-on `log` is lighter but loses structured fields and OpenTelemetry compatibility (the monitor's planned Prometheus/OTLP path). `#[instrument]` on hot path costs ~2-5% per frame in span construction — unacceptable at peak trade rates.
**Alternatives considered**:
- `log` crate (rejected — no span/structured fields).
- `tracing` always-on (rejected — bloat for non-subscribers).
- `#[instrument]` on dispatch loop (rejected — overhead).

### D2 — Reconnect default flip + binding compensation
**Choice**: `ReconnectionConfig::default().enabled = true`. Each binding crate explicitly calls `ReconnectionConfig::disabled()` at the FFI boundary so wrapped end users observe no change.
**Why X over Y**: Symmetric ergonomic across Rust ecosystem; bindings can re-establish historical semantics without affecting Rust callers. The doc-only warning alternative leaves the trap in place.
**Alternatives considered**:
- Doc warning only (rejected — trap remains).
- New `with_auto_reconnect()` constructor + keep `new()` behavior (rejected — two ways to do the same thing).

### D3 — REST retry as opt-in builder
**Choice**: `RestClient::with_retry(RetryPolicy { max_attempts, initial_backoff, max_backoff })`. Off by default. Reuses existing `is_retryable()`.
**Why X over Y**: Observability tooling needs visibility into transient failures; default-on retry would mask real outages. Builder form is consistent with existing `RestClient` builder methods.

### D4 — `connection_id` rolled back before release (was on every variant of `ConnectionEvent`)
**Initial choice (Wave B)**: add `connection_id: Option<String>` to every `ConnectionEvent` variant + `ConnectionConfig::builder().id(...)` setter, intended for multi-connection event attribution.
**Final choice**: removed before 0.4.0 release. `ConnectionEvent` variants stay in their pre-Wave-B shape (`Connecting`, `Connected`, `Authenticated` are unit-like; `Reconnecting { attempt }`, `Disconnected { code, reason, intent }` etc.). `ConnectionConfig::connection_id` field and `ConnectionConfigBuilder::id(...)` method are removed.
**Why the rollback**:
- `tokio::select!` arms attribute events by Receiver source — caller already knows.
- `stream::select_all([...])` merge users can wrap with a 3-line label adapter (`ReceiverStream::new(rx).map(|e| ("stock", e))`).
- `tracing::info_span!("ws", id="...").in_scope(|| ...)` covers central log attribution at the call site.
- Bindings (Python / Node / UniFFI / etc.) wrap exactly one client and never need event-side labels.
- Field cost (~24 B per event idle, mandatory `..` wildcard in every downstream pattern matcher) outweighs the value for the singleton-per-channel + select! consumer model that 99% of callers have.
**`WebSocketFactory` retained** for shared-base-URL ergonomic (D10); only its `.id()` chain in tests/docs went.
**`DisconnectIntent` retained** (D7) — independent decision, separately useful programmatic Client/Server/Network discrimination that string parsing can't do reliably.
**Risk note R1 update**: items #5 (doc) + #13 still rewrite `connection_event.rs`; item #4 no longer applies.

### D5 — Graceful shutdown drain
**Choice**: `disconnect()` adds default 5s drain timeout. New `shutdown_with_timeout(Duration)`. Sync side mirrors aio: drain `WRITE_QUEUE_CAPACITY=64` write queue then send Close, then wait for server-side close frame within poll interval.
**Why X over Y**: WebSocket close handshake requires both sides to send Close before TCP teardown. Current `aio/client.rs:560` even has a comment "Must continue reading after close() until receiving ConnectionClosed error" but the impl skips that step. Default-5s preserves the ergonomic of `disconnect().await` while fixing the bug; `shutdown_with_timeout` exposes the knob.
**Alternatives considered**:
- New `shutdown_with_timeout` only, leave `disconnect()` fire-and-forget (rejected — silent footgun stays).
- Doc-only "remember to sleep after disconnect" (rejected — fixes nothing).

### D6 — Hand-rolled `Debug` redaction (no `secrecy` crate)
**Choice**: Manual `impl Debug` on `Auth` and `ConnectionConfig`. Output `Auth::ApiKey(***)`. Also redact known sensitive query params in `ConnectionConfig::url`.
**Why X over Y**: `secrecy::Secret<String>` is type-level and would cascade breaking changes through all `Auth` consumers (`apply_to_request` would need `.expose_secret()` everywhere). Hand-rolled `Debug` is one-file change with zero API surface impact.
**Alternatives considered**:
- Partial mask `abcd***wxyz` (rejected — partial leak).
- `secrecy` crate (rejected — cascade).

### D7 — `DisconnectIntent` on both event and state
**Choice**: Add `intent: DisconnectIntent { Client, Server, Network }` to BOTH `ConnectionEvent::Disconnected` and `ConnectionState::Closed`.
**Why X over Y**: The two enums are sibling representations of the same lifecycle moment. Diverging shape now would force a 0.4.1 patch when the first user files "why does state lack intent?".
**Alternatives considered**:
- Only on `ConnectionEvent::Disconnected` (rejected — divergence trap).

### D8 — Buffer caps configurable, message default bumped to 4096
**Choice**: Builder methods `message_buffer(usize)` and `event_buffer(usize)`. Defaults: `message_buffer = 4096` (up from the pre-0.4.0 hardcoded 1024), `event_buffer = 1024` (unchanged). `WRITE_QUEUE_CAPACITY=64` (sync owner thread internal queue) stays a `pub(crate) const`.
**Why X over Y**: Message and event buffers are the user-facing producer-consumer interfaces; the write queue is an internal sync-side optimization that only matters for `subscribe`/`unsubscribe` burst tolerance and would confuse users.

The default bump from 1024 → 4096 for `message_buffer` is justified by:
- **Free at idle**: tokio/std bounded `mpsc` channels are lazily allocated. A higher cap costs zero bytes when the consumer keeps up; cost is bounded by `cap × entry_size` only at saturation (~1.6 MB/client at 4096 vs ~400 KB at 1024 — both trivial).
- **Production headroom math**: Fugle live data delivers ~100–500 msg/s for typical subscriptions and bursts to ~2000 msg/s for multi-symbol consumers (50–200 symbols at TWSE 9:00 open). At 2000 msg/s, 1024 = 0.5 s consumer-pause tolerance; 4096 = 2 s. The downstream `monitor` use case sits squarely in that range.
- **No tuning required for the common case**: with 4096 default, the average user never thinks about the buffer; the existing `messages_dropped_total` counter exposes drops if they ever happen.

The `event_buffer` default of 1024 is left unchanged: lifecycle events fire ~once per heartbeat (30 s), so 1024 already represents ~8 hours of headroom. Reducing it (e.g. to 128) is unjustified churn before a binding-impact release; revisit in 0.5.0 once production telemetry confirms it is consistently underused.

### D10 — JS-style `WebSocketFactory` for shared base URL
**Choice**: Add `WebSocketFactory::new(auth).base_url(base).stock().build()` mirroring `fugle-marketdata-node/src/websocket/factory.ts` and `fugle-marketdata-python/fugle_marketdata/websocket/factory.py`. Internally derives `{base}/{API_VERSION}/{type}/streaming`. Returns `ConnectionConfigBuilder` (not `ConnectionConfig`) so callers can chain `.id(...)`, `.message_buffer(...)`, etc. before `.build()`. Also expose `urls::WS_BASE_ROOT` / `urls::REST_BASE_ROOT` / `urls::API_VERSION` constants so downstream code can compose URLs symbolically.

**Why X over Y**: monitor and other multi-environment consumers want to point both stock and futopt streams at staging or a local mock server in one place — the JS / Python SDKs already expose this shape and the binding-layer code that wraps the Rust core would otherwise have to re-implement the JS factory on top of the bare `ConnectionConfig::fugle_stock(auth)` API. Returning a builder rather than a fully-built config keeps the factory composable with every existing setter (`id`, `message_buffer`, `event_buffer`, `connect_timeout`, etc.) without duplicating each on the factory itself.

**Alternatives considered**:
- `ConnectionConfig::fugle_stock_at(base, auth)` / `fugle_futopt_at(base, auth)` standalone helpers (rejected — naming is awkward, and callers always need both stock + futopt with the same base, so the factory wrapping is more honest).
- A unified `MarketData` factory owning both REST and WS clients with a single shared base (rejected for 0.4.0 — REST already has `RestClient::base_url(...)`, and unifying both transports requires a deeper refactor that's better timed with the eventual REST-side ergonomic pass in 0.5.0).
- Builder `.url(...)` setter that overrides the constructor positional arg (rejected — the positional URL semantically conflicts with a setter that swaps it post-construction).

### D9 — Drop test utils (#10) to 0.5.0
**Choice**: Defer `pub mod testing { MockWebSocketServer }` to a separate release.
**Why X over Y**: Mock server with fixture replay is its own design problem (deciding between echo / scripted / record-replay flavors, dealing with TLS, fixture format). Bundling it would inflate this release by 30-40% and delay the security-critical items (#7, #2). 0.4.0 ships with `tokio::io::duplex` based integration tests for the new behaviors — sufficient for SDK-level correctness without exposing test infra.

## Risks / Trade-offs

- **R1 — `ConnectionEvent` triple-touch in same file**: items #4, #5 (doc), #13 all rewrite `connection_event.rs` → mitigation: land as ONE atomic Wave-B PR; estimate ~21 `emit_event` call sites updated together (`dispatch.rs ×4`, `reconnect.rs ×6`, `writer.rs ×1`, `sync/client.rs ×6`).
- **R2 — uniffi silent inheritance of #2 flip**: `uniffi/src/websocket.rs:95-100` constructs `ReconnectionConfig::default()` then mutates → mitigation: audit binding's `to_core` mapping for an explicit `enabled` field; add CI assertion `assert_eq!(binding_default.enabled, false)`.
- **R3 — Python/Node binding reconnect surface unknown**: grep shows zero `ReconnectionConfig` references in `fugle-marketdata-python/` and `fugle-marketdata-node/`; they may not expose reconnect to users at all → mitigation: per-binding investigation (grep `pyo3` / `napi`) before merging the flip; identify the wrapper that constructs `ConnectionConfig`.
- **R4 — Doctest pattern-match break**: `aio/client.rs:248,250` doc examples pattern-match `ConnectionEvent::Connected => println!(...)` → mitigation: `cargo test -p marketdata-core --doc` after #4 lands; update examples to use `..` or new fields.
- **R5 — `runtime.rs:116` is not panic boundary**: it's runtime construction failure, replace with `tracing::error!` (not `eprintln!`) → so #1 replaces 3 of 5 `eprintln!` (`connection_event.rs:69`, `runtime.rs:116`, `aio/client.rs:578`); keep 2 (`runtime.rs:25,39`) as `eprintln!` (real panic boundaries).
- **R6 — URL query token leak in `ConnectionConfig::url`**: some WS auth schemes pass token as query param → mitigation: `Debug` impl regex-redacts known sensitive query keys (`token`, `key`, `apikey`, `api_key`, `secret`, `password`).
- **R7 — Saturation doc obsolete after #14**: `connection_event.rs:65-66` hardcodes "cap of 1024" → mitigation: doc revision lands in same PR as #14.
- **R8 — Multi-binding propagation**: 6+ binding directories (`uniffi`, `python`, `node`, `bindings/{go,java,cpp,csharp}`) need atomic update with core release → mitigation: workspace-level CI test asserts core default `true` AND per-binding default `false`; release blocks until all green.

## Migration Plan

A new `MIGRATION-0.4.md` at the repo root (mirrors existing `MIGRATION-0.3.md`). Outline:

1. **`ConnectionEvent` shape change** — every variant gains `connection_id: Option<String>`; `Disconnected` gains `intent: DisconnectIntent`. Pattern-match callers must add `..` or destructure new fields. Example before/after.
2. **`ConnectionState::Closed` shape change** — same `intent` field added.
3. **`ReconnectionConfig::default()` flip** — Rust callers expecting `enabled: false` must explicitly call `ReconnectionConfig::disabled()`. Binding users see no change.
4. **`Auth` / `ConnectionConfig` Debug change** — output format changes from token-revealing to `Auth::ApiKey(***)` redacted form. Anyone parsing `format!("{:?}", ...)` will break (security feature, not a bug).
5. **`SubscribeRequest::{trades,candles,books,aggregates}` removed** — replace with `SubscribeRequest::new(Channel::*, symbol)`.
6. **`disconnect()` semantics** — now blocks up to 5s waiting for Close ACK. Callers expecting fire-and-forget should use `shutdown_with_timeout(Duration::from_millis(0))` if they truly want immediate return (rare).
7. **Tracing opt-in** — to enable, add `marketdata-core = { version = "0.4", features = ["tracing"] }` and install a `tracing_subscriber`.

Rollback strategy: `0.4.0` is a minor bump; downstream pins to `0.3.x` continue to work. No upgrade pressure beyond `monitor`'s production launch deadline.

## Open Questions

- **Q1**: Python and Node binding reconnect surfaces — what file:line is the construction site of `ConnectionConfig` (or its binding equivalent) in `fugle-marketdata-python/` and `fugle-marketdata-node/`? Investigation must complete before #2 merges.
- **Q2**: `RestClient::with_retry` — should there be a typed `RetryPolicy::default()` (e.g. 3 attempts, 100ms initial, 5s max) or force the caller to specify all three fields? Recommendation: provide `RetryPolicy::conservative()` and `RetryPolicy::aggressive()` factory functions; resolve during implementation.
- **Q3**: `DisconnectIntent::Network` granularity — should heartbeat-timeout disconnect use a distinct sub-variant (`Network::HeartbeatTimeout` vs `Network::TransportError`)? Recommendation: keep flat for 0.4.0; revisit if downstream needs richer classification.
- **Q4**: `messages_dropped_total` — should `events_dropped_total` also be exposed? Recommendation: yes, parallel symmetry, cheap to add — finalize in tasks.md.
