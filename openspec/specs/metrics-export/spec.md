# metrics-export Specification

## Purpose
Defines the optional `metrics`-crate integration for `fugle-marketdata-core`: a `metrics` Cargo feature that mirrors the in-process atomic counters (`messages_dropped_total`, `events_dropped_total`) onto the `metrics` facade so downstream consumers can wire any recorder (Prometheus, statsd, OTLP) without forking the SDK. Introduced in 0.7.0 to support production observability stacks while keeping the default build dependency-free.

## Requirements
### Requirement: metrics feature gate

`core/Cargo.toml` SHALL declare a new `[features]` entry `metrics = ["dep:metrics"]`. The feature MUST be off by default and MUST NOT appear in `default = []`. The optional `metrics` dependency SHALL pin to the latest stable major version (`metrics = "0.23"` as of 2026-05). Production builds without the feature MUST NOT pull `metrics` or any of its transitive dependencies.

#### Scenario: Feature is off by default
- **WHEN** `core/Cargo.toml` is read
- **THEN** the `[features]` table MUST contain `metrics = ["dep:metrics"]` AND `default = []` MUST NOT include `metrics`

#### Scenario: Default build excludes metrics dependency
- **WHEN** `cargo tree -p fugle-marketdata-core --no-default-features` is run
- **THEN** the output MUST NOT contain `metrics v0.` AND MUST NOT contain `metrics-util`

#### Scenario: metrics build pulls only the metrics facade
- **WHEN** `cargo tree -p fugle-marketdata-core --features metrics --no-default-features` is run
- **THEN** the output MUST contain `metrics v0.23` AND MUST NOT contain `metrics-exporter-prometheus` (consumers wire their own exporter)

### Requirement: Counter registration on WebSocketClient construction

When `feature = "metrics"` is enabled, both `core::websocket::aio::WebSocketClient::new(...)` and `core::websocket::sync::WebSocketClient::new(...)` SHALL register two `metrics` crate counters at construction time using `metrics::describe_counter!`:

- `fugle_marketdata_ws_messages_dropped_total` — channel back-pressure drops on the inbound `messages()` channel.
- `fugle_marketdata_ws_events_dropped_total` — broadcast back-pressure drops on the `connection_events()` bus.

Each counter SHALL carry two labels: `endpoint` (the WebSocket URL host extracted from `ConnectionConfig::url`) and `client_id` (the value of `ConnectionConfig::client_id`, empty string when unset).

When `feature = "metrics"` is disabled, no `metrics` calls SHALL be compiled into the binary and the existing atomic counters MUST continue to work unchanged.

#### Scenario: Counters describe on construction
- **WHEN** `WebSocketClient::new(config)` is called with `feature = "metrics"` enabled and a `metrics-util::debugging::DebuggingRecorder` installed
- **THEN** the recorder MUST observe two counter descriptions: `fugle_marketdata_ws_messages_dropped_total` and `fugle_marketdata_ws_events_dropped_total`

#### Scenario: Counter increments on drop
- **WHEN** `feature = "metrics"` is enabled and the inbound channel back-pressures, causing one message to be dropped
- **THEN** the `fugle_marketdata_ws_messages_dropped_total` counter MUST increment by 1 with labels `endpoint=<host>` and `client_id=<configured-id-or-empty>`

#### Scenario: Atomic counters remain authoritative
- **WHEN** `feature = "metrics"` is enabled and a drop occurs
- **THEN** `client.messages_dropped_total()` MUST return the same total it would return without the feature (atomic counter remains the source of truth for in-process callers)

#### Scenario: No metrics calls without feature
- **WHEN** `cargo build -p fugle-marketdata-core --no-default-features --features tokio-comp` is run (no `metrics` feature)
- **THEN** the build MUST succeed AND the resulting binary MUST NOT contain symbols matching `metrics::counter` or `metrics::describe_counter`

### Requirement: client_id builder field on ConnectionConfig

`ConnectionConfig::builder()` SHALL expose a `client_id(impl Into<String>)` setter and a `maybe_client_id(Option<impl Into<String>>)` setter. The field SHALL default to `None`. When unset, the metrics counter labels render `client_id` as the empty string. The setter SHALL accept any string up to 64 bytes; longer values SHALL be truncated to 64 bytes and a one-shot `tracing::warn!` SHALL be emitted (gated on `feature = "tracing"`) to flag the truncation.

#### Scenario: Default client_id is None
- **WHEN** `ConnectionConfig::builder(url, auth).build()` is called without invoking `client_id(...)`
- **THEN** the resulting config's `client_id()` accessor MUST return `None`

#### Scenario: client_id setter records value
- **WHEN** `ConnectionConfig::builder(url, auth).client_id("monitor-stock-probe").build()` is called
- **THEN** the resulting config's `client_id()` accessor MUST return `Some("monitor-stock-probe")`

#### Scenario: Long client_id is truncated with warning
- **WHEN** `ConnectionConfig::builder(url, auth).client_id("a".repeat(128)).build()` is called with `feature = "tracing"` enabled and a `tracing_subscriber` installed
- **THEN** the resulting config's `client_id()` accessor MUST return a 64-byte string AND the subscriber MUST observe one `WARN` event mentioning `client_id` truncation

### Requirement: Counter naming follows Prometheus convention

Counter names SHALL end in `_total` (Prometheus convention for monotonic counters). The `metrics-exporter-prometheus` crate is known to suffix counter names automatically; the SDK MUST NOT compensate for that — `_total` belongs in the SDK-side name so other recorders (statsd, OTLP) emit the conventional name without exporter-specific logic.

#### Scenario: Both counters end in _total
- **WHEN** the registered counter names are inspected
- **THEN** both names MUST match the regex `^fugle_marketdata_ws_(messages|events)_dropped_total$`

#### Scenario: Names contain no double-suffix
- **WHEN** the registered counter names are inspected
- **THEN** neither name MUST end in `_total_total` or contain duplicate `_total` segments
