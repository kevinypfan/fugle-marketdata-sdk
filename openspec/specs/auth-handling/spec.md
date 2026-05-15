# auth-handling Specification

## Purpose
TBD - created by archiving change sdk-04-improvements. Update Purpose after archive.
## Requirements
### Requirement: Auth Debug redaction
`Auth` SHALL implement `core::fmt::Debug` manually (NOT via `#[derive]`) such that the underlying secret value is never written to the formatter. The redacted form SHALL preserve the variant name and SHALL substitute the secret with the literal `***`.

#### Scenario: ApiKey is redacted
- **WHEN** `format!("{:?}", Auth::ApiKey("secret-key-123".into()))` is called
- **THEN** the result MUST be exactly `Auth::ApiKey(***)` (no portion of `"secret-key-123"` MUST appear)

#### Scenario: BearerToken is redacted
- **WHEN** `format!("{:?}", Auth::BearerToken("eyJ...".into()))` is called
- **THEN** the result MUST be exactly `Auth::BearerToken(***)`

#### Scenario: SdkToken is redacted
- **WHEN** `format!("{:?}", Auth::SdkToken("sdk-abc".into()))` is called
- **THEN** the result MUST be exactly `Auth::SdkToken(***)`

### Requirement: ConnectionConfig Debug redaction
`ConnectionConfig` SHALL implement `core::fmt::Debug` manually such that the embedded `Auth` is rendered via its redacted Debug. If the `url` field contains a query string, the value of any query parameter whose name case-insensitively matches `token`, `key`, `apikey`, `api_key`, `secret`, or `password` SHALL be replaced with `***` in the rendered output.

#### Scenario: Embedded auth is redacted
- **WHEN** `format!("{:?}", ConnectionConfig::stock_websocket(Auth::ApiKey("k".into())))` is called
- **THEN** the rendered output MUST contain `Auth::ApiKey(***)` and MUST NOT contain `"k"`

#### Scenario: URL query token is redacted
- **WHEN** the config is built with URL `wss://example.com/stream?token=secret&v=1`
- **THEN** the Debug output MUST render the URL with `token=***&v=1` and MUST NOT contain `secret`

### Requirement: Environment-variable Auth helper
`Auth` SHALL expose an associated function `from_env() -> Result<Auth, MarketDataError>` that resolves authentication from the process environment. The helper SHALL probe the variables `FUGLE_API_KEY`, then `FUGLE_BEARER_TOKEN`, then `FUGLE_SDK_TOKEN` in that order and return the first non-empty match wrapped in the corresponding `Auth` variant.

#### Scenario: ApiKey precedence
- **WHEN** both `FUGLE_API_KEY=k1` and `FUGLE_BEARER_TOKEN=t1` are set
- **THEN** `Auth::from_env()` MUST return `Ok(Auth::ApiKey("k1".into()))`

#### Scenario: Falls back to bearer
- **WHEN** only `FUGLE_BEARER_TOKEN=t1` is set
- **THEN** `Auth::from_env()` MUST return `Ok(Auth::BearerToken("t1".into()))`

#### Scenario: No vars returns ConfigError
- **WHEN** none of `FUGLE_API_KEY`, `FUGLE_BEARER_TOKEN`, `FUGLE_SDK_TOKEN` are set
- **THEN** `Auth::from_env()` MUST return `Err(MarketDataError::ConfigError { .. })` whose message names all three checked variables

#### Scenario: Empty string treated as unset
- **WHEN** `FUGLE_API_KEY=""` is set and `FUGLE_BEARER_TOKEN=t1` is set
- **THEN** `Auth::from_env()` MUST return `Ok(Auth::BearerToken("t1".into()))`

