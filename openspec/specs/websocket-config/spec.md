# websocket-config Specification

## Purpose
TBD - created by archiving change sdk-04-improvements. Update Purpose after archive.
## Requirements
### Requirement: Configurable channel buffers
`ConnectionConfig::builder(...)` SHALL expose `message_buffer(usize)` and `event_buffer(usize)` chainable methods. When set, the corresponding tokio/std mpsc channels constructed by the client MUST use these capacities. When unset, `message_buffer` MUST default to 4096 and `event_buffer` MUST default to 1024. The 4096 message-channel default replaces the pre-0.4.0 hardcoded value of 1024 to give multi-symbol consumers ~2 seconds of headroom at TWSE 9:00 open burst (~2000 msg/s) before drop-newest backpressure begins. Bounded mpsc channels do not pre-allocate, so a higher cap costs nothing at idle. The event-channel default of 1024 is intentionally generous because lifecycle events fire only on the order of one per heartbeat (30 s).

#### Scenario: Custom message buffer applied
- **WHEN** `ConnectionConfig::builder(url, auth).message_buffer(2048).build()` is used to construct a client
- **THEN** the client's message channel MUST have capacity 2048

#### Scenario: Default message buffer is 4096
- **WHEN** `ConnectionConfig::builder(url, auth).build()` is used without `message_buffer(...)`
- **THEN** the client's message channel MUST have capacity 4096

#### Scenario: Default event buffer is 1024
- **WHEN** `ConnectionConfig::builder(url, auth).build()` is used without `event_buffer(...)`
- **THEN** the client's event channel MUST have capacity 1024

#### Scenario: Zero is rejected
- **WHEN** `message_buffer(0)` or `event_buffer(0)` is called
- **THEN** the builder MUST either panic with a clear message or return an error variant prior to client construction (no zero-cap channel may be constructed)

### Requirement: URL constants module
The crate SHALL expose `pub mod urls` containing the full canonical endpoints (`STOCK_WS`, `FUTOPT_WS`, `REST_BASE`) plus the host roots and version segment used to derive them at run time:

- `pub const STOCK_WS: &str = "wss://api.fugle.tw/marketdata/v1.0/stock/streaming"`
- `pub const FUTOPT_WS: &str = "wss://api.fugle.tw/marketdata/v1.0/futopt/streaming"`
- `pub const REST_BASE: &str = "https://api.fugle.tw/marketdata/v1.0"`
- `pub const REST_BASE_ROOT: &str = "https://api.fugle.tw/marketdata"` (host root, no version)
- `pub const WS_BASE_ROOT: &str = "wss://api.fugle.tw/marketdata"` (host root, no version)
- `pub const API_VERSION: &str = "v1.0"` (shared between REST and WebSocket)

The convenience constructors (`ConnectionConfig::fugle_stock`, `ConnectionConfig::fugle_futopt`, `RestClient::new`) SHALL read from the canonical full-URL constants, not inline string literals.

#### Scenario: Canonical full-URL constants are stable strings
- **WHEN** the crate is built
- **THEN** `urls::STOCK_WS` MUST equal `"wss://api.fugle.tw/marketdata/v1.0/stock/streaming"`, `urls::FUTOPT_WS` MUST equal `"wss://api.fugle.tw/marketdata/v1.0/futopt/streaming"`, and `urls::REST_BASE` MUST equal `"https://api.fugle.tw/marketdata/v1.0"`

#### Scenario: Roots reconstruct full URLs
- **WHEN** the crate is built
- **THEN** `format!("{}/{}/stock/streaming", urls::WS_BASE_ROOT, urls::API_VERSION)` MUST equal `urls::STOCK_WS`, `format!("{}/{}/futopt/streaming", urls::WS_BASE_ROOT, urls::API_VERSION)` MUST equal `urls::FUTOPT_WS`, and `format!("{}/{}", urls::REST_BASE_ROOT, urls::API_VERSION)` MUST equal `urls::REST_BASE`

#### Scenario: Convenience constructors use the constants
- **WHEN** `ConnectionConfig::fugle_stock(auth)` is called
- **THEN** the resulting config's `url` field MUST equal `urls::STOCK_WS`

### Requirement: WebSocketFactory shared base URL
The crate SHALL expose `WebSocketFactory` mirroring the JS / Python SDK's `WebSocketClient` factory shape: one auth credential plus an optional shared base URL produces both stock and futopt endpoint configurations. The factory SHALL expose:

- `WebSocketFactory::new(auth: AuthRequest) -> Self` — defaults base URL to `urls::WS_BASE_ROOT`.
- `base_url(self, base: impl Into<String>) -> Self` — chainable override of the shared host root. Trailing slashes on `base` MUST be stripped before concatenation.
- `stock(&self) -> ConnectionConfigBuilder` — derived endpoint `{base}/{API_VERSION}/stock/streaming`.
- `futopt(&self) -> ConnectionConfigBuilder` — derived endpoint `{base}/{API_VERSION}/futopt/streaming`.

Builders returned from `stock()` / `futopt()` MUST be independent (chaining `.id(...)` on one MUST NOT affect the other) so a single factory can produce multiple distinct configurations.

#### Scenario: Default base produces canonical endpoints
- **WHEN** `WebSocketFactory::new(auth).stock().build()` is called without `base_url(...)`
- **THEN** the resulting config's `url` field MUST equal `urls::STOCK_WS`

#### Scenario: Default base produces canonical futopt endpoint
- **WHEN** `WebSocketFactory::new(auth).futopt().build()` is called without `base_url(...)`
- **THEN** the resulting config's `url` field MUST equal `urls::FUTOPT_WS`

#### Scenario: Custom base applied to both endpoints
- **WHEN** a factory is built with `base_url("wss://staging.fugle.tw/marketdata")` and both `.stock().build()` and `.futopt().build()` are called
- **THEN** the resulting URLs MUST be `"wss://staging.fugle.tw/marketdata/v1.0/stock/streaming"` and `"wss://staging.fugle.tw/marketdata/v1.0/futopt/streaming"` respectively

#### Scenario: Trailing slashes are stripped
- **WHEN** the factory is built with `base_url("wss://example.com/marketdata///")`
- **THEN** `.stock().build().url` MUST equal `"wss://example.com/marketdata/v1.0/stock/streaming"`

#### Scenario: Builders from same factory are independent
- **WHEN** a factory yields two builders (`a = factory.stock(); b = factory.stock()`) with different chainable settings (e.g. distinct `message_buffer(...)` values)
- **THEN** the resulting configs MUST carry the respective settings without cross-contamination

