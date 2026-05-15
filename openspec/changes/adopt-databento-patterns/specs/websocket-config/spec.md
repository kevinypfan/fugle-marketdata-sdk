## MODIFIED Requirements

### Requirement: WebSocketFactory shared base URL

The crate SHALL expose `WebSocketFactory` as a phantom-generic typestate builder mirroring databento-rs's `LiveClient::builder()` pattern. The factory tracks required-field state at the type level so `.stock()` / `.futopt()` only become callable once `auth` is set; calling them on an unconfigured factory MUST fail at compile time, not at runtime.

Type-level shape:

- `WebSocketFactory<Unset>`: initial state. Returned by `WebSocketFactory::new()`.
- `WebSocketFactory<WithAuth>`: state after `auth(AuthRequest)` has been called.

The factory SHALL expose:

- `WebSocketFactory::new() -> WebSocketFactory<Unset>` — defaults base URL to `urls::WS_BASE_ROOT`.
- `auth(self, auth: AuthRequest) -> WebSocketFactory<WithAuth>` — sets the auth credential, advancing the typestate. Only available on `WebSocketFactory<Unset>`.
- `base_url(self, base: impl Into<String>) -> Self` — chainable override of the shared host root, available in any state. Trailing slashes on `base` MUST be stripped before concatenation.
- `stock(&self) -> ConnectionConfigBuilder` — derived endpoint `{base}/{API_VERSION}/stock/streaming`. Only available on `WebSocketFactory<WithAuth>`.
- `futopt(&self) -> ConnectionConfigBuilder` — derived endpoint `{base}/{API_VERSION}/futopt/streaming`. Only available on `WebSocketFactory<WithAuth>`.

Builders returned from `stock()` / `futopt()` MUST be independent (chaining `.id(...)` on one MUST NOT affect the other) so a single factory can produce multiple distinct configurations.

#### Scenario: Default base produces canonical endpoints
- **WHEN** `WebSocketFactory::new().auth(auth).stock().build()` is called without `base_url(...)`
- **THEN** the resulting config's `url` field MUST equal `urls::STOCK_WS`

#### Scenario: Default base produces canonical futopt endpoint
- **WHEN** `WebSocketFactory::new().auth(auth).futopt().build()` is called without `base_url(...)`
- **THEN** the resulting config's `url` field MUST equal `urls::FUTOPT_WS`

#### Scenario: Custom base applied to both endpoints
- **WHEN** a factory is built with `WebSocketFactory::new().auth(auth).base_url("wss://staging.fugle.tw/marketdata")` and both `.stock().build()` and `.futopt().build()` are called
- **THEN** the resulting URLs MUST be `"wss://staging.fugle.tw/marketdata/v1.0/stock/streaming"` and `"wss://staging.fugle.tw/marketdata/v1.0/futopt/streaming"` respectively

#### Scenario: Trailing slashes are stripped
- **WHEN** the factory is built with `WebSocketFactory::new().auth(auth).base_url("wss://example.com/marketdata///")`
- **THEN** `.stock().build().url` MUST equal `"wss://example.com/marketdata/v1.0/stock/streaming"`

#### Scenario: Builders from same factory are independent
- **WHEN** a factory `f = WebSocketFactory::new().auth(auth)` yields two builders (`a = f.stock(); b = f.stock()`) with different chainable settings (e.g. distinct `message_buffer(...)` values)
- **THEN** the resulting configs MUST carry the respective settings without cross-contamination

#### Scenario: Calling stock before auth fails at compile time
- **WHEN** downstream code attempts `WebSocketFactory::new().stock()` without calling `.auth(...)` first
- **THEN** the compiler MUST reject the code with a method-not-found error pointing at the missing `WithAuth` state; no runtime fallback or default-auth behavior may exist

#### Scenario: Calling futopt before auth fails at compile time
- **WHEN** downstream code attempts `WebSocketFactory::new().futopt()` without calling `.auth(...)` first
- **THEN** the compiler MUST reject the code with a method-not-found error pointing at the missing `WithAuth` state
