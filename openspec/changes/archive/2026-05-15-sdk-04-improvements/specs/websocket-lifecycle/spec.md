## ADDED Requirements

### Requirement: Reconnect default is enabled
`ReconnectionConfig::default()` SHALL return a configuration with `enabled: true`, `max_attempts`, `initial_delay`, and `max_delay` matching the values previously documented as the explicit `ReconnectionConfig::new(...)` defaults.

#### Scenario: Default enabled
- **WHEN** `ReconnectionConfig::default()` is invoked
- **THEN** the returned config's `enabled` field MUST be `true`

#### Scenario: Disabled helper still available
- **WHEN** `ReconnectionConfig::disabled()` is invoked
- **THEN** the returned config's `enabled` field MUST be `false`

### Requirement: Bindings preserve disabled default
Each binding crate that exposes `ReconnectionConfig` (or constructs `ConnectionConfig` containing one) SHALL explicitly call `ReconnectionConfig::disabled()` at the FFI boundary so end users of the binding observe no behavior change from the core default flip. Affected: `uniffi/`, `fugle-marketdata-python/`, `fugle-marketdata-node/`, `bindings/{go, java, cpp, csharp}/`.

#### Scenario: Binding default is disabled
- **WHEN** a binding wrapper constructs its default reconnection configuration without the user explicitly opting in
- **THEN** the underlying `marketdata_core::ReconnectionConfig::enabled` field MUST be `false`

### Requirement: Disconnect drains by default
`WebSocketClient::disconnect()` (both sync and async) SHALL apply a default 5 second drain timeout. Within this window the client MUST send the Close frame, await the peer's Close acknowledgement (async: tungstenite `ConnectionClosed`; sync: server-side close frame within poll interval), and drain any in-flight outbound writes from the writer queue. After the timeout expires, the client MUST forcibly close the transport.

#### Scenario: Close ack received within window
- **WHEN** the peer sends Close ACK 100ms after `disconnect()` is called
- **THEN** `disconnect()` MUST return `Ok(())` within 200ms (allowing for scheduling)

#### Scenario: No ack triggers force close at timeout
- **WHEN** the peer never acknowledges the Close frame
- **THEN** `disconnect()` MUST return within `5s + 250ms` and the underlying transport MUST be closed

#### Scenario: In-flight writes drained
- **WHEN** two outbound subscribe messages are queued immediately before `disconnect()` and the writer task is still alive
- **THEN** both messages MUST be flushed to the wire before the Close frame is sent

### Requirement: Custom shutdown timeout
A new public method `shutdown_with_timeout(&self, timeout: Duration)` SHALL be available on both sync and async clients with semantics equivalent to `disconnect()` but using the supplied `timeout` instead of the 5 second default.

#### Scenario: Custom 100ms timeout
- **WHEN** the caller invokes `shutdown_with_timeout(Duration::from_millis(100))` and the peer never acknowledges
- **THEN** the call MUST return within `100ms + 50ms`
