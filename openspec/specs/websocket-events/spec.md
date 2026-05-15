# websocket-events Specification

## Purpose
TBD - created by archiving change sdk-04-improvements. Update Purpose after archive.
## Requirements
### Requirement: DisconnectIntent classification
`ConnectionEvent::Disconnected` SHALL include an `intent: DisconnectIntent` field where `DisconnectIntent` is `enum { Client, Server, Network }` representing who triggered the disconnect.

- `Client` MUST be set when the local caller invoked `disconnect()` or `shutdown_with_timeout(...)`.
- `Server` MUST be set when the disconnect originated from a server-sent Close frame (regardless of code).
- `Network` MUST be set when the transport returned an error or the stream ended without a Close frame (including heartbeat timeout escalation).

#### Scenario: Local disconnect classified as Client
- **WHEN** the caller invokes `client.disconnect().await` and a `Disconnected` event is then observed
- **THEN** the event MUST carry `intent: DisconnectIntent::Client`

#### Scenario: Server-sent close classified as Server
- **WHEN** the peer sends a Close frame with code 1000
- **THEN** the resulting `Disconnected` event MUST carry `intent: DisconnectIntent::Server`

#### Scenario: Transport error classified as Network
- **WHEN** the underlying TCP stream returns EOF without a Close frame
- **THEN** the resulting `Disconnected` event MUST carry `intent: DisconnectIntent::Network`

#### Scenario: Heartbeat timeout classified as Network
- **WHEN** no inbound frame is received within the configured `heartbeat_timeout` and the connection tears down
- **THEN** the subsequent `Disconnected` event MUST carry `intent: DisconnectIntent::Network`

### Requirement: ConnectionState mirrors intent
`ConnectionState::Closed` SHALL carry the same `intent: DisconnectIntent` field as `ConnectionEvent::Disconnected` so that state inspection by the caller does not lose classification information.

#### Scenario: State carries matching intent
- **WHEN** a `Disconnected` event with `intent: DisconnectIntent::Client` has been emitted
- **THEN** the corresponding `ConnectionState::Closed { intent, .. }` MUST carry `intent: DisconnectIntent::Client`

### Requirement: WebSocketErrorKind structured classification

The crate SHALL expose `pub enum WebSocketErrorKind` in `core::errors` with the following non-exhaustive variants:

- `Protocol` — frame-format violation, illegal state transition, reserved bits set. Never retryable.
- `Capacity` — frame exceeds the configured `max_message_size` / `max_frame_size`. Never retryable.
- `Utf8` — UTF-8 decoding failed on a text frame. Never retryable.
- `Tls` — TLS / certificate failure during handshake. Never retryable.
- `Io` — transport IO failure (reset, EOF, write error). Retryable.
- `Http(u16)` — HTTP error during the WebSocket upgrade. Retryability depends on the status (see `is_retryable()` requirement below).
- `Other` — anything not classified above. Retryable (conservative default).

The enum MUST be `#[non_exhaustive]` so future additions are non-breaking. `MarketDataError::WebSocketError` SHALL reshape from `{ msg: String }` to `{ kind: WebSocketErrorKind, msg: String }`. The `From<tungstenite::Error>` impl SHALL map each upstream variant to the appropriate kind:

| `tungstenite::Error` variant | `WebSocketErrorKind` |
|---|---|
| `Protocol(_)` | `Protocol` |
| `Capacity(_)` | `Capacity` |
| `Utf8(_)` | `Utf8` |
| `Tls(_)` | `Tls` |
| `Io(_)`, `ConnectionClosed`, `AlreadyClosed` | `Io` |
| `Http(resp)` | `Http(resp.status().as_u16())` |
| any other / future | `Other` |

`MarketDataError::is_retryable()` SHALL honour the kind:

| Kind | Retryable |
|---|---|
| `Protocol`, `Capacity`, `Utf8`, `Tls` | no |
| `Io`, `Other` | yes |
| `Http(401)`, `Http(403)` | no |
| `Http(429)`, `Http(500..=599)` | yes |
| `Http(other)` | no |

#### Scenario: WebSocketError carries kind
- **WHEN** any code constructs `MarketDataError::WebSocketError { kind: WebSocketErrorKind::Protocol, msg: "x".into() }`
- **THEN** the variant MUST compile with the two-field shape

#### Scenario: Protocol violations are non-retryable
- **WHEN** `MarketDataError::WebSocketError { kind: WebSocketErrorKind::Protocol, msg: "frame".into() }.is_retryable()` is evaluated
- **THEN** the result MUST be `false`

#### Scenario: IO failures are retryable
- **WHEN** `MarketDataError::WebSocketError { kind: WebSocketErrorKind::Io, msg: "reset".into() }.is_retryable()` is evaluated
- **THEN** the result MUST be `true`

#### Scenario: HTTP 401/403 are non-retryable
- **WHEN** `MarketDataError::WebSocketError { kind: WebSocketErrorKind::Http(401), msg: "x".into() }.is_retryable()` is evaluated
- **THEN** the result MUST be `false`

#### Scenario: HTTP 429/5xx are retryable
- **WHEN** `MarketDataError::WebSocketError { kind: WebSocketErrorKind::Http(503), msg: "x".into() }.is_retryable()` is evaluated
- **THEN** the result MUST be `true`

#### Scenario: tungstenite::Error::Protocol routes to Protocol kind
- **WHEN** a `tungstenite::Error::Protocol(_)` is converted via `From<tungstenite::Error>`
- **THEN** the resulting `MarketDataError::WebSocketError` MUST carry `kind == WebSocketErrorKind::Protocol`

#### Scenario: tungstenite::Error::Io routes to Io kind
- **WHEN** a `tungstenite::Error::Io(_)` is converted via `From<tungstenite::Error>`
- **THEN** the resulting `MarketDataError::WebSocketError` MUST carry `kind == WebSocketErrorKind::Io`

#### Scenario: WebSocketErrorKind requires wildcard arm
- **WHEN** downstream code writes `match kind { WebSocketErrorKind::Protocol => .., WebSocketErrorKind::Capacity => .., WebSocketErrorKind::Utf8 => .., WebSocketErrorKind::Tls => .., WebSocketErrorKind::Io => .., WebSocketErrorKind::Http(_) => .., WebSocketErrorKind::Other => .. }` without a wildcard
- **THEN** the code MUST fail to compile with a `non-exhaustive` error

