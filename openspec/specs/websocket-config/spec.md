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

The crate SHALL expose `WebSocketFactory` as a phantom-generic typestate builder mirroring databento-rs's `LiveClient::builder()` pattern. The factory tracks required-field state at the type level so `.stock()` / `.futopt()` only become callable once `auth` is set; calling them on an unconfigured factory MUST fail at compile time, not at runtime.

Type-level shape:

- `WebSocketFactory<Unset>`: initial state. Returned by `WebSocketFactory::new()`.
- `WebSocketFactory<WithAuth>`: state after `auth(AuthRequest)` has been called.

The factory SHALL expose:

- `WebSocketFactory::new() -> WebSocketFactory<Unset>` — defaults base URL to `urls::STOCK_WS` / `urls::FUTOPT_WS` (full canonical endpoints with version).
- `auth(self, auth: AuthRequest) -> WebSocketFactory<WithAuth>` — sets the auth credential, advancing the typestate. Only available on `WebSocketFactory<Unset>`.
- `base_url(self, base: impl Into<String>) -> Self` — chainable override of the full URL prefix that endpoint paths are appended to. **`base` MUST include the API version segment** (e.g. `"wss://example.com/marketdata/v1.0"`). The factory appends only `/{stock|futopt}/streaming`; it does NOT inject `/{API_VERSION}` for caller-supplied bases. Trailing slashes on `base` MUST be stripped before concatenation. Available in any typestate.
- `stock(&self) -> ConnectionConfigBuilder` — derived endpoint `{base_url}/stock/streaming`. Only available on `WebSocketFactory<WithAuth>`.
- `futopt(&self) -> ConnectionConfigBuilder` — derived endpoint `{base_url}/futopt/streaming`. Only available on `WebSocketFactory<WithAuth>`.

`base_url` semantics align with `RestClient::base_url(&str)` and with the OpenAI / Stripe / AWS / Anthropic convention — a single transferable mental model: "set `base_url` to everything before the endpoint path; the SDK appends the endpoint suffix".

This requirement formalises a **silent breaking semantic change** from 0.5.x: the same call `factory.base_url("wss://example.com/marketdata")` produced `wss://example.com/marketdata/v1.0/stock/streaming` in 0.5.x but produces `wss://example.com/marketdata/stock/streaming` in 0.6.0+. Migration requires callers to include the version segment in the string they pass.

Builders returned from `stock()` / `futopt()` MUST be independent.

#### Scenario: Default endpoint produces canonical stock URL
- **WHEN** `WebSocketFactory::new().auth(auth).stock().build()` is called without `base_url(...)`
- **THEN** the resulting config's `url` field MUST equal `urls::STOCK_WS` (which is `wss://api.fugle.tw/marketdata/v1.0/stock/streaming`)

#### Scenario: Default endpoint produces canonical futopt URL
- **WHEN** `WebSocketFactory::new().auth(auth).futopt().build()` is called without `base_url(...)`
- **THEN** the resulting config's `url` field MUST equal `urls::FUTOPT_WS`

#### Scenario: Custom base_url includes version and is appended once
- **WHEN** a factory is built with `WebSocketFactory::new().auth(auth).base_url("wss://staging.fugle.tw/marketdata/v1.0")` and `.stock().build()` is called
- **THEN** the resulting `url` MUST equal `"wss://staging.fugle.tw/marketdata/v1.0/stock/streaming"` (no second `/v1.0` injected)

#### Scenario: Custom base_url applied to futopt
- **WHEN** the same factory's `.futopt().build()` is called
- **THEN** the resulting `url` MUST equal `"wss://staging.fugle.tw/marketdata/v1.0/futopt/streaming"`

#### Scenario: Different API version segment is honored
- **WHEN** a factory is built with `WebSocketFactory::new().auth(auth).base_url("wss://api.fugle.tw/marketdata/v2.0")` and `.stock().build()` is called
- **THEN** the resulting `url` MUST equal `"wss://api.fugle.tw/marketdata/v2.0/stock/streaming"` (factory does NOT force the value of `urls::API_VERSION` onto user-supplied bases)

#### Scenario: Trailing slashes are stripped
- **WHEN** the factory is built with `WebSocketFactory::new().auth(auth).base_url("wss://example.com/marketdata/v1.0///")`
- **THEN** `.stock().build().url` MUST equal `"wss://example.com/marketdata/v1.0/stock/streaming"`

#### Scenario: Builders from same factory are independent
- **WHEN** a factory `f = WebSocketFactory::new().auth(auth)` yields two builders (`a = f.stock(); b = f.stock()`) with different chainable settings
- **THEN** the resulting configs MUST carry the respective settings without cross-contamination

#### Scenario: Calling stock before auth fails at compile time
- **WHEN** downstream code attempts `WebSocketFactory::new().stock()` without calling `.auth(...)` first
- **THEN** the compiler MUST reject the code with a method-not-found error pointing at the missing `WithAuth` state

#### Scenario: Calling futopt before auth fails at compile time
- **WHEN** downstream code attempts `WebSocketFactory::new().futopt()` without calling `.auth(...)` first
- **THEN** the compiler MUST reject the code with a method-not-found error pointing at the missing `WithAuth` state

#### Scenario: 0.5.x host-root caller produces non-canonical URL in 0.6.0 (regression test)
- **WHEN** code written for 0.5.x calls `WebSocketFactory::new().auth(auth).base_url("wss://api.fugle.tw/marketdata").stock().build()` (host root, no version) after upgrading to 0.6.0
- **THEN** the resulting `url` MUST equal `"wss://api.fugle.tw/marketdata/stock/streaming"` (which the Fugle gateway rejects with 404) — confirming the silent semantic shift documented in `MIGRATION-0.6.md`

