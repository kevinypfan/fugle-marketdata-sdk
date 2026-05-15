## MODIFIED Requirements

### Requirement: ErrorKind classification helper

The crate SHALL expose `pub enum ErrorKind` in `core::errors` with the variants `Network`, `Protocol`, `Auth`, `Client`. The enum MUST be `#[non_exhaustive]` so future additions are non-breaking. `MarketDataError` SHALL provide `pub fn source_kind(&self) -> ErrorKind` that returns the variant best describing the source of the failure, per the table below.

| `MarketDataError` variant | `ErrorKind` |
|---|---|
| `ConnectionError`, `TimeoutError`, `HeartbeatTimeout` | `Network` |
| `WebSocketError { kind: WebSocketErrorKind::Protocol \| Capacity \| Utf8 }` | `Protocol` |
| `WebSocketError { kind: WebSocketErrorKind::Tls }` | `Auth` |
| `WebSocketError { kind: WebSocketErrorKind::Io }` | `Network` |
| `WebSocketError { kind: WebSocketErrorKind::Http(401 \| 403) }` | `Auth` |
| `WebSocketError { kind: WebSocketErrorKind::Http(429 \| 500..=599) }` | `Network` |
| `WebSocketError { kind: WebSocketErrorKind::Http(other) }` | `Client` |
| `WebSocketError { kind: WebSocketErrorKind::Other }` | `Protocol` |
| `AuthError` | `Auth` |
| `ApiError { status: 401 \| 403 }` | `Auth` |
| `ApiError { status: 429 \| 500..=599 }` | `Network` |
| `ApiError { status: other 4xx }` | `Client` |
| `InvalidSymbol`, `InvalidParameter`, `ConfigError`, `DeserializationError`, `ClientClosed` | `Client` |
| `RuntimeError`, `Other` | `Client` |

#### Scenario: Network category for transport failures
- **WHEN** `MarketDataError::ConnectionError { msg: "x".into() }.source_kind()` is evaluated
- **THEN** the result MUST equal `ErrorKind::Network`

#### Scenario: Protocol kind routes to Protocol category
- **WHEN** `MarketDataError::WebSocketError { kind: WebSocketErrorKind::Protocol, msg: "x".into() }.source_kind()` is evaluated
- **THEN** the result MUST equal `ErrorKind::Protocol`

#### Scenario: Io kind routes to Network category
- **WHEN** `MarketDataError::WebSocketError { kind: WebSocketErrorKind::Io, msg: "reset".into() }.source_kind()` is evaluated
- **THEN** the result MUST equal `ErrorKind::Network`

#### Scenario: Tls kind routes to Auth category
- **WHEN** `MarketDataError::WebSocketError { kind: WebSocketErrorKind::Tls, msg: "cert".into() }.source_kind()` is evaluated
- **THEN** the result MUST equal `ErrorKind::Auth`

#### Scenario: WebSocket HTTP 401 routes to Auth category
- **WHEN** `MarketDataError::WebSocketError { kind: WebSocketErrorKind::Http(401), msg: "x".into() }.source_kind()` is evaluated
- **THEN** the result MUST equal `ErrorKind::Auth`

#### Scenario: Auth category for 401/403 API errors
- **WHEN** `MarketDataError::ApiError { status: 401, message: "x".into() }.source_kind()` is evaluated
- **THEN** the result MUST equal `ErrorKind::Auth`

#### Scenario: Network category for 5xx and 429
- **WHEN** `MarketDataError::ApiError { status: 503, message: "x".into() }.source_kind()` is evaluated
- **THEN** the result MUST equal `ErrorKind::Network`

#### Scenario: Client category for validation failures
- **WHEN** `MarketDataError::InvalidParameter { name: "x".into(), reason: "y".into() }.source_kind()` is evaluated
- **THEN** the result MUST equal `ErrorKind::Client`

#### Scenario: ErrorKind requires wildcard arm
- **WHEN** downstream code writes `match err.source_kind() { ErrorKind::Network => .., ErrorKind::Protocol => .., ErrorKind::Auth => .., ErrorKind::Client => .. }` without a wildcard
- **THEN** the code MUST fail to compile with a `non-exhaustive` error pointing at the missing `_` arm
