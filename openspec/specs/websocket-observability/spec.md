# websocket-observability Specification

## Purpose
TBD - created by archiving change sdk-04-improvements. Update Purpose after archive.
## Requirements
### Requirement: Opt-in tracing feature
The crate SHALL expose a Cargo feature `tracing` that, when enabled, pulls in `tracing = "0.1"` as a runtime dependency. When the feature is disabled, no `tracing` symbols and no extra dependencies SHALL be compiled into the produced binary.

#### Scenario: Default build excludes tracing
- **WHEN** a downstream user runs `cargo build -p marketdata-core` with default features
- **THEN** the resulting binary MUST NOT contain `tracing` crate symbols and the `tracing` crate MUST NOT appear in `Cargo.lock` as a direct or non-test transitive dependency contributed by `marketdata-core`

#### Scenario: Feature-on build emits tracing events
- **WHEN** a downstream user enables `--features tracing` and installs a `tracing_subscriber::fmt` collector
- **THEN** the collector MUST receive at minimum one `info!`-level event for connection established and one `warn!`-level event for reconnect attempt during a reconnect cycle

### Requirement: Cold-path instrumentation policy
When the `tracing` feature is enabled, the SDK SHALL apply `#[tracing::instrument(target = "fugle_marketdata::ws", name = "ws.<op>")]` to exactly four cold-path operations (`connect`, `subscribe`, `unsubscribe`, `disconnect`) on both sync and async clients, with explicit `name` to disambiguate sync vs async modules. The hot-path frame dispatch loop MUST NOT carry `#[instrument]`.

#### Scenario: Subscribe span emitted
- **WHEN** the user calls `client.subscribe(req).await` with feature on
- **THEN** the collector MUST receive a span entered named `ws.subscribe` with `target = "fugle_marketdata::ws"`

#### Scenario: Frame dispatch loop has no span
- **WHEN** 10,000 frames are processed under feature-on
- **THEN** no per-frame span MUST be created (verified by counting spans matching `ws.frame*` is zero)

### Requirement: Hot-path debug events
When the `tracing` feature is enabled, the SDK SHALL emit `tracing::debug!` with structured fields (`bytes`, `channel` where applicable) at the WebSocket frame receive site for every successfully decoded text or binary frame.

#### Scenario: Frame debug event
- **WHEN** a text frame of N bytes is decoded
- **THEN** a `debug!` event MUST be emitted carrying `bytes = N`

### Requirement: Lifecycle and error event coverage
When the `tracing` feature is enabled, the SDK SHALL emit at minimum:
- `info!` on connect established and on authenticated
- `warn!` on reconnect attempt (carrying `attempt`, `delay_ms`)
- `warn!` on heartbeat timeout (carrying `elapsed_ms`)
- `warn!` on event-channel saturation (carrying dropped event variant)
- `warn!` on message-channel saturation (carrying dropped count)
- `error!` on runtime construction failure (replacing the existing `eprintln!` at `core/src/websocket/aio/runtime.rs:116`)
- `error!` on close-frame send failure (replacing the existing `eprintln!` at `core/src/websocket/aio/client.rs:578`)

The two `eprintln!` calls at `core/src/websocket/aio/runtime.rs:25` and `:39` (the FFI panic boundary) SHALL remain as `eprintln!` because they fire after the tracing subscriber may already be torn down.

#### Scenario: Reconnect attempt warn
- **WHEN** a reconnection attempt is initiated
- **THEN** a `warn!` event MUST be emitted with both `attempt` and `delay_ms` fields populated

#### Scenario: Panic boundary keeps eprintln
- **WHEN** the FFI panic boundary catches a panic
- **THEN** the message MUST be written via `eprintln!` (not `tracing::error!`), so it is visible even if the tracing subscriber has been dropped

### Requirement: Backpressure drop counter
Each WebSocket client (sync and async) SHALL maintain a monotonically increasing atomic counter of messages dropped due to message-channel saturation. The counter SHALL be readable via a public method `messages_dropped_total(&self) -> u64`. When the `tracing` feature is enabled, every drop SHALL also emit a `tracing::warn!` event.

#### Scenario: Counter increments on saturation
- **WHEN** the message channel is configured with capacity 1 and 5 frames arrive while the consumer is blocked
- **THEN** `client.messages_dropped_total()` MUST return at least 4

#### Scenario: Counter starts at zero
- **WHEN** a fresh client is constructed
- **THEN** `client.messages_dropped_total()` MUST return 0

### Requirement: Backpressure policy documented
The `core/src/websocket/connection_event.rs` module-level doc SHALL document the drop-newest backpressure policy explicitly, including the rationale (avoiding receiver-side mutation of `std::sync::mpsc`) and the relationship between configurable buffer caps and the saturation signal.

#### Scenario: Doc references both policy and counter
- **WHEN** rustdoc is generated for the `connection_event` module
- **THEN** the rendered doc MUST mention both "drop-newest" and the `messages_dropped_total` counter as the operator signal

### Requirement: Event drop counter

Both sync `WebSocketClient` and async `aio::WebSocketClient` SHALL expose `pub fn events_dropped_total(&self) -> u64`. The counter MUST be backed by an `Arc<AtomicU64>` shared with the `emit_event` saturation site in `core/src/websocket/connection_event.rs` and MUST be incremented once for every `ConnectionEvent` dropped by the bounded event channel's drop-newest backpressure policy.

The semantics mirror the existing `messages_dropped_total()` counter for the inbound message channel.

#### Scenario: Counter starts at zero
- **WHEN** a `WebSocketClient` is constructed and no events have fired
- **THEN** `client.events_dropped_total()` MUST return `0`

#### Scenario: Saturation increments the counter
- **WHEN** the event channel is full and `emit_event` is invoked with a new event
- **THEN** the call MUST drop the new event without blocking AND the next read of `client.events_dropped_total()` MUST observe a value strictly greater than the previous read

#### Scenario: Counter is monotonic
- **WHEN** any sequence of saturating-then-non-saturating event emissions occurs
- **THEN** every subsequent read of `client.events_dropped_total()` MUST be greater than or equal to every previous read

