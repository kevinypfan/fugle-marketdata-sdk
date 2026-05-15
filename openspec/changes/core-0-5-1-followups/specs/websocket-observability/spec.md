## ADDED Requirements

### Requirement: Event drop counter

Both sync `WebSocketClient` and async `aio::WebSocketClient` SHALL expose `pub fn events_dropped_total(&self) -> u64`. The counter MUST be backed by an `Arc<AtomicU64>` shared with the `emit_event` saturation site in `core/src/websocket/connection_event.rs` and MUST be incremented once for every `ConnectionEvent` dropped by the bounded event channel's drop-newest backpressure policy.

The semantics mirror the existing `messages_dropped_total()` counter for the inbound message channel.

#### Scenario: Counter starts at zero
- **WHEN** a `WebSocketClient` is constructed and no events have fired
- **THEN** `client.events_dropped_total()` MUST return `0`

#### Scenario: Saturation increments the counter
- **WHEN** the event channel is full and `emit_event` is invoked with a new event
- **THEN** the call MUST drop the new event without blocking AND the next read of `client.events_dropped_total()` MUST observe a value strictly greater than the previous read

#### Scenario: Counter is monotonic
- **WHEN** any sequence of saturating-then-non-saturating event emissions occurs
- **THEN** every subsequent read of `client.events_dropped_total()` MUST be greater than or equal to every previous read
