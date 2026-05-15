## MODIFIED Requirements

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
