## ADDED Requirements

### Requirement: Multi-client capacity on MockWsServer

`MockWsServer` SHALL expose `pub async fn start_with_capacity(capacity: usize) -> Self` that accepts up to `capacity` clients before refusing further connections. Each accepted client SHALL be assigned a zero-based `client_idx` in accept order. The existing `start()` method MUST continue to work as a thin wrapper over `start_with_capacity(1)`. `capacity == 0` MUST panic with a clear message.

The module SHALL also expose `pub async fn aio_pair_n(capacity: usize) -> (MockWsServer, Vec<crate::aio::WebSocketClient>)` returning `capacity` clients all paired against the same server, with auto-reconnect disabled on each.

#### Scenario: start_with_capacity(2) accepts two clients
- **WHEN** `let (server, clients) = aio_pair_n(2).await; for c in &clients { c.connect().await? }` is executed
- **THEN** both clients MUST connect successfully against the same `server.url()`

#### Scenario: start() preserves single-client behaviour
- **WHEN** `let (server, client) = aio_pair().await; client.connect().await?` is executed
- **THEN** the connection MUST succeed exactly as in 0.6.0 with no behavioural change

#### Scenario: Capacity zero panics
- **WHEN** `MockWsServer::start_with_capacity(0).await` is called
- **THEN** the call MUST panic with a message containing "capacity must be > 0"

#### Scenario: Excess connections are refused
- **WHEN** a third client attempts to connect to a `start_with_capacity(2)` mock that already has two connected clients
- **THEN** the third connection MUST fail at the TCP-accept or WebSocket-handshake layer (the mock MUST NOT silently accept and stall)

### Requirement: Per-client targeting methods

`MockWsServer` SHALL expose `_for(client_idx, ...)` overloads of every state-injecting method:

- `pub async fn inject_frame_for(&self, client_idx: usize, frame: crate::models::streaming::StreamMessage)`
- `pub async fn next_subscribe_id_for(&self, client_idx: usize, id: impl Into<String>)`
- `pub async fn close_for(&self, client_idx: usize, code: u16, reason: impl Into<String>)`

The existing no-`_for` methods (`inject_frame`, `next_subscribe_id`, `close`) MUST continue to work on a `start()` / `start_with_capacity(1)` server and MUST panic with a message naming the offending method when called against a multi-client server (capacity > 1). Out-of-range `client_idx` MUST panic with a message including the requested index and the configured capacity.

#### Scenario: inject_frame_for delivers to the targeted client
- **WHEN** `let (s, cs) = aio_pair_n(2).await;` both clients are connected, and `s.inject_frame_for(0, StreamMessage::Authenticated).await` is called
- **THEN** `cs[0].messages().recv()` MUST return the matching frame AND `cs[1].messages().recv()` MUST NOT receive that frame

#### Scenario: Bare inject_frame panics on multi-client mock
- **WHEN** `s.inject_frame(StreamMessage::Authenticated).await` is called on a server constructed via `start_with_capacity(2)`
- **THEN** the call MUST panic with a message containing "use inject_frame_for"

#### Scenario: Out-of-range client_idx panics
- **WHEN** `s.inject_frame_for(5, frame).await` is called on a `start_with_capacity(2)` server
- **THEN** the call MUST panic with a message containing both "5" and "2"

### Requirement: Transport-drop intent injection

`MockWsServer` SHALL expose `pub async fn drop_transport(&self)` and `pub async fn drop_transport_for(&self, client_idx: usize)` that close the underlying TCP socket of the targeted client without sending a WebSocket Close frame. The targeted client MUST observe the disconnect as `DisconnectIntent::Network` (not `DisconnectIntent::Server`).

The mock as a whole MUST be capable of injecting all three `DisconnectIntent` outcomes the SDK can emit:

- `DisconnectIntent::Server` — via the existing `close(...)` (and `close_for(...)`) graceful Close frame.
- `DisconnectIntent::Network` — via the new `drop_transport(...)` (and `drop_transport_for(...)`) transport-level drop.
- `DisconnectIntent::Client` — produced by the SDK when the caller invokes `client.disconnect().await`; the mock does not need a separate hook for this case.

After `drop_transport_for(idx)` runs, subsequent `_for(idx, ...)` calls against the same client_idx MUST be no-ops (the per-client task has exited); the mock MUST NOT panic.

#### Scenario: drop_transport produces Network intent
- **WHEN** a single client is connected and `mock.drop_transport().await` is called
- **THEN** the client MUST emit a `ConnectionEvent::Disconnected` event carrying `intent: DisconnectIntent::Network` AND the `code` MUST be `None` or a transport-error sentinel (no application-level close code)

#### Scenario: close still produces Server intent
- **WHEN** a single client is connected and `mock.close(1001, "going away").await` is called
- **THEN** the client MUST emit `ConnectionEvent::Disconnected { code: Some(1001), reason: "going away", intent: DisconnectIntent::Server }`

#### Scenario: drop_transport_for targets one client only
- **WHEN** a 2-capacity mock has both clients connected and `mock.drop_transport_for(0).await` is called
- **THEN** client 0 MUST emit a `Disconnected` event with `intent: DisconnectIntent::Network` AND client 1 MUST remain connected with no event

#### Scenario: Inject after drop is a no-op
- **WHEN** `mock.drop_transport_for(0).await` has completed and `mock.inject_frame_for(0, frame).await` is called afterward
- **THEN** the call MUST NOT panic AND no frame MUST be delivered (the per-client task has exited)
