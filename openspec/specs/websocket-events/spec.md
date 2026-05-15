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

