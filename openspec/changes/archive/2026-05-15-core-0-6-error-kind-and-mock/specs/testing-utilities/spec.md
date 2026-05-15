## ADDED Requirements

### Requirement: pub mod testing behind test-utils feature

The crate SHALL expose `pub mod testing` from `core/src/testing/mod.rs` gated by `#[cfg(all(feature = "test-utils", feature = "tokio-comp"))]`. The `test-utils` feature SHALL be off by default and SHALL declare `tokio-comp`, `dep:tokio-tungstenite`, and `dep:futures-util` as its dependencies. Production builds (without `test-utils`) MUST NOT compile the module or pull its transitive deps.

#### Scenario: Feature is off by default
- **WHEN** `core/Cargo.toml` is read
- **THEN** the `[features]` table MUST contain `test-utils = [...]` AND `default = []` MUST NOT include `test-utils`

#### Scenario: Production build excludes testing module
- **WHEN** `cargo build -p fugle-marketdata-core` is run with default features
- **THEN** `core::testing` MUST NOT be a resolvable path AND `tokio-tungstenite` MUST NOT appear in the dependency graph beyond what `tokio-comp` already pulls

#### Scenario: test-utils gates testing module
- **WHEN** `cargo build --features test-utils -p fugle-marketdata-core` is run
- **THEN** `core::testing` MUST resolve and `MockWsServer` MUST be a public type within it

### Requirement: MockWsServer surface

`core::testing::MockWsServer` SHALL expose:

- `pub async fn start() -> Self` — bind to an ephemeral 127.0.0.1 port and begin accepting one client.
- `pub fn url(&self) -> String` — the `ws://127.0.0.1:<port>/...` URL a SDK config can point at.
- `pub fn address(&self) -> std::net::SocketAddr` — the underlying socket address.
- `pub fn next_subscribe_id(&self, id: impl Into<String>)` — force the next subscribe ACK to assign the given server-side id.
- `pub async fn inject_frame(&self, frame: crate::models::streaming::StreamMessage)` — push a frame the client receives on its next `messages().recv()`.
- `pub async fn close(&self, code: u16, reason: impl Into<String>)` — server-initiated WebSocket close.

The module SHALL also provide:

- `pub async fn aio_pair() -> (MockWsServer, crate::aio::WebSocketClient)` — convenience constructor pairing a server with a pre-configured async client.

The mock server MUST speak the same subscribe-ACK protocol that `core/src/websocket/protocol.rs` produces — both the single-symbol and batch shapes — so existing client tests can swap real-server for mock-server with no client-side changes.

#### Scenario: start binds to ephemeral port
- **WHEN** `MockWsServer::start().await` is called twice
- **THEN** each instance MUST return a distinct, non-zero `SocketAddr` port

#### Scenario: aio_pair connects without external network
- **WHEN** `let (server, client) = aio_pair().await; client.connect().await?` is called against a server with no auth requirement
- **THEN** the connection MUST succeed without any DNS lookup or network access beyond the local loopback interface

#### Scenario: Subscribe ACK uses configured id
- **WHEN** the server is primed with `server.next_subscribe_id("my-id-42")` and a client sends `SubscribeRequest::new(Channel::Trades, "2330")` then waits for the ACK
- **THEN** the client's `SubscriptionManager` MUST record the server-assigned id `"my-id-42"` for that subscription

#### Scenario: inject_frame delivers to client
- **WHEN** `server.inject_frame(StreamMessage::Authenticated).await` is called and the client polls `messages().recv()`
- **THEN** the client MUST receive the matching `StreamMessage::Authenticated`

#### Scenario: close emits Disconnected event with server intent
- **WHEN** `server.close(1001, "going away").await` is called on a connected client
- **THEN** the client MUST emit `ConnectionEvent::Disconnected { code: Some(1001), reason: "going away", intent: DisconnectIntent::Server }`
