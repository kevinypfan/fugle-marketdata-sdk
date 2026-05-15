## Why

Five non-breaking gaps surfaced in the 0.5.0 audit that are too small to justify their own changes but too sharp to leave for 0.6.0:

- Downstream consumers (especially the monitor integration) can't programmatically distinguish protocol-violation `WebSocketError` from transport-IO `WebSocketError` — both flow through the same string-only variant with `is_retryable() == true`.
- The lifecycle event channel uses drop-newest backpressure with no observability counter (the message channel has `messages_dropped_total()`).
- The `Symbols::normalized` dedup contract is silent about case sensitivity, so `"TXFB6"` vs `"txfb6"` behaviour is implementation-defined.
- `ReconnectionConfig::disabled()` is load-bearing for FFI bindings but has no `#[must_use]` and no doc-level stability promise.
- Users encounter both the unvalidated `::builder()` and validating `::new(...)` paths with no idiomatic guidance on which to pick.

All five are additive or doc-only; ship as a 0.5.1 patch to harden the surface before any 0.6.0 work begins.

## What Changes

- **`MarketDataError::source_kind() -> ErrorKind { Network, Protocol, Auth, Client }` helper** (additive). Lets `WebSocketError { msg }` callers classify failures programmatically without forcing a breaking enum split — protocol violations route to `Protocol`, IO/timeout to `Network`, etc. Mapping is documented per variant.
- **`events_dropped_total() -> u64`** counter on both sync and async `WebSocketClient`, mirroring the existing `messages_dropped_total()` on the message channel. Incremented at the `emit_event` saturation site in `connection_event.rs`.
- **Symbols dedup case-sensitivity is documented + enforced.** `Symbols::normalized` MUST preserve case; `"TXFB6"` and `"txfb6"` MUST remain distinct entries. New scenario in the subscription-api spec; unit test in `models/symbols.rs`.
- **`ReconnectionConfig::disabled()` doc-stability promise.** Add `#[must_use]`, expand rustdoc to say "stable public API; FFI bindings depend on this at the boundary". Same treatment for `RetryPolicy::conservative()` / `aggressive()` and `ReconnectionConfig::default()` while we're there.
- **Idiomatic-builder documentation.** Crate-level rustdoc and README each gain a "Which constructor should I use?" section: prefer `bon::Builder` for non-validating cases (`RetryPolicy`, `ReconnectionConfig`, `SubscribeRequest`); prefer positional `new(...)` constructors when validation is required (`ReconnectionConfig::new`).

## Capabilities

### New Capabilities
(none)

### Modified Capabilities
- `websocket-observability`: adds the `events_dropped_total()` counter requirement alongside the existing message-channel counter.
- `subscription-api`: refines the `Symbols::normalized` contract to fix the case-sensitivity policy.
- `documentation-policy`: adds the "idiomatic constructor selection" requirement so the README + crate docs always pin the right answer.

## Impact

- **Affected files**:
  - `core/src/errors.rs` — new `ErrorKind` enum + `source_kind()` method on `MarketDataError`
  - `core/src/websocket/connection_event.rs` — counter increment at the saturation site; expose the counter via a shared atomic
  - `core/src/websocket/sync/client.rs` + `core/src/websocket/aio/client.rs` — `events_dropped_total()` accessor mirroring `messages_dropped_total()`
  - `core/src/models/symbols.rs` — explicit case-preservation doc + unit test
  - `core/src/websocket/reconnection.rs` — `#[must_use]` + doc upgrade on `disabled()`
  - `core/src/rest/retry.rs` — `#[must_use]` + doc upgrade on `conservative()` / `aggressive()`
  - `core/README.md` + `core/src/lib.rs` (via `include_str!`) — idiomatic constructor section
- **APIs**: all additions; no removals; no renames. Existing call sites compile unchanged.
- **Dependencies**: none added.
- **Downstream**: bindings (py/js/uniffi) gain the new counter via re-export but are otherwise unaffected.
- **Release**: 0.4.x style patch — `0.5.0 -> 0.5.1`. CHANGELOG entry under "Added" with no Breaking subsection.
