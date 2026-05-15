# error-classification Specification

## Purpose
Defines the coarse-grained classification surface for `MarketDataError`. The `ErrorKind` enum + `source_kind()` helper let downstream consumers (especially monitor integration) branch on failure *category* (Network / Protocol / Auth / RateLimit / Client) without pattern-matching every variant or string-matching error messages. Introduced in 0.5.1 to bridge the gap before the 0.6.0 `WebSocketError` enum split refines the Protocol category.
## Requirements
### Requirement: ErrorKind classification helper

The crate SHALL expose `pub enum ErrorKind` in `core::errors` with the variants `Network`, `Protocol`, `Auth`, `RateLimit`, `Client`. The enum MUST be `#[non_exhaustive]` so future additions are non-breaking. `MarketDataError` SHALL provide `pub fn source_kind(&self) -> ErrorKind` that returns the variant best describing the source of the failure, per the table below.

`RateLimit` is distinct from `Network` because operational response differs: rate-limit rejections require the caller to **reduce** request volume (de-parallelize, slow down), whereas `Network` failures call for retry with backoff. Conflating the two leads monitor incident playbooks to take the wrong action (adding parallel retries when the right move is to throttle).

| `MarketDataError` variant | `ErrorKind` |
|---|---|
| `ConnectionError`, `TimeoutError`, `HeartbeatTimeout` | `Network` |
| `WebSocketError` | `Protocol` (refined in 0.6.0 once the variant is split) |
| `AuthError` | `Auth` |
| `ApiError { status: 401 \| 403 }` | `Auth` |
| `ApiError { status: 429 }` | `RateLimit` |
| `ApiError { status: 500..=599 }` | `Network` |
| `ApiError { status: other 4xx }` | `Client` |
| `InvalidSymbol`, `InvalidParameter`, `ConfigError`, `DeserializationError`, `ClientClosed` | `Client` |
| `RuntimeError`, `Other` | `Client` |

#### Scenario: Network category for transport failures
- **WHEN** `MarketDataError::ConnectionError { msg: "x".into() }.source_kind()` is evaluated
- **THEN** the result MUST equal `ErrorKind::Network`

#### Scenario: Protocol category for WebSocket failures (coarse pre-0.6.0)
- **WHEN** `MarketDataError::WebSocketError { msg: "x".into() }.source_kind()` is evaluated in 0.5.1
- **THEN** the result MUST equal `ErrorKind::Protocol`

#### Scenario: Auth category for 401/403 API errors
- **WHEN** `MarketDataError::ApiError { status: 401, message: "x".into() }.source_kind()` is evaluated
- **THEN** the result MUST equal `ErrorKind::Auth`

#### Scenario: RateLimit category for 429
- **WHEN** `MarketDataError::ApiError { status: 429, message: "throttle".into() }.source_kind()` is evaluated
- **THEN** the result MUST equal `ErrorKind::RateLimit`

#### Scenario: Network category for 5xx (not for 429)
- **WHEN** `MarketDataError::ApiError { status: 503, message: "x".into() }.source_kind()` is evaluated
- **THEN** the result MUST equal `ErrorKind::Network`

#### Scenario: Client category for validation failures
- **WHEN** `MarketDataError::InvalidParameter { name: "x".into(), reason: "y".into() }.source_kind()` is evaluated
- **THEN** the result MUST equal `ErrorKind::Client`

#### Scenario: ErrorKind requires wildcard arm
- **WHEN** downstream code writes `match err.source_kind() { ErrorKind::Network => .., ErrorKind::Protocol => .., ErrorKind::Auth => .., ErrorKind::RateLimit => .., ErrorKind::Client => .. }` without a wildcard
- **THEN** the code MUST fail to compile with a `non-exhaustive` error pointing at the missing `_` arm

