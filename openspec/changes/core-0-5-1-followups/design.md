## Context

0.5.0 just shipped to crates.io (tags `v0.4.1` / `v0.5.0`, published 2026-05-15). The audit produced eight findings; three are deferred to 0.6.0 (enum split, mock server, factory rename) and one is a no-op (`tracing_compat` already `pub(crate)`). The remaining five are small, additive, doc-leaning fixes that benefit from shipping together as `0.5.1` rather than rolling into the next minor.

Stakeholders:
- **Monitor integration**: needs to classify `WebSocketError` failures without substring matching.
- **FFI bindings**: lean on `ReconnectionConfig::disabled()` at the boundary; want the stability promise codified.
- **First-time Rust users**: hit two builder paths (`bon` builder vs `new()`) and don't know which is idiomatic.

Constraints:
- No public API removals or renames — this is a patch, not a minor.
- No new runtime dependencies.
- FFI binding crates (py/js/uniffi) must build unchanged.

## Goals / Non-Goals

**Goals:**
- Programmatic error classification without breaking the `MarketDataError` enum shape.
- Symmetric observability: `events_dropped_total()` mirrors `messages_dropped_total()`.
- Explicit case-sensitivity contract on `Symbols::normalized`.
- Stability promise on FFI-load-bearing constructors (`disabled()`, `default()`, presets).
- One canonical answer to "which constructor is idiomatic" in README + crate-level rustdoc.

**Non-Goals:**
- Splitting `WebSocketError` into a structured-kind enum. (Deferred to 0.6.0 — that's a breaking change.)
- Building `pub mod testing` / `MockWsServer`. (Deferred to 0.6.0 — new module + feature flag is minor territory.)
- Renaming `WebSocketFactory::base_url(...)` outright. We will add the `host_root(...)` alias here so the 0.6.0 removal is mechanical, but neither name is removed in 0.5.1.
- Reworking the `tracing_compat` macros. They're already `pub(crate)`.

## Decisions

### D1: `source_kind()` helper, not enum split

**Choice**: add `pub fn source_kind(&self) -> ErrorKind` to `MarketDataError` returning a four-variant non-exhaustive enum: `Network`, `Protocol`, `Auth`, `Client`.

**Mapping**:

| `MarketDataError` variant | `ErrorKind` |
|---|---|
| `ConnectionError`, `TimeoutError`, `HeartbeatTimeout` | `Network` |
| `WebSocketError` (collapsed) | `Protocol` *for now* — refined in 0.6.0 when the variant is split |
| `AuthError`, `ApiError { status: 401\|403 }` | `Auth` |
| `InvalidSymbol`, `InvalidParameter`, `ConfigError`, `DeserializationError`, `ClientClosed` | `Client` |
| `ApiError { status: 5xx\|429 }` | `Network` |
| `ApiError { status: 4xx }` (excl. 401/403) | `Client` |
| `RuntimeError`, `Other` | `Client` |

**Rationale**: A helper is non-breaking. Downstream `match err.source_kind() { ErrorKind::Network => /* retry */, ErrorKind::Protocol => /* fail fast */, ... }` already buys 80% of the value an enum split would. When 0.6.0 splits `WebSocketError`, the helper's return value gets refined automatically — no API churn for consumers.

**Trade-off**: `WebSocketError`'s entire population maps to `Protocol` in 0.5.1, which is wrong for the IO subset (they should be `Network`). Documented as a known coarse-grained classification that 0.6.0 fixes.

**Alternatives considered**:
- *Split the enum now (option 1b)*: rejected — breaking. Goes in 0.6.0.
- *`is_protocol()` / `is_network()` predicate fan-out*: rejected — explodes the API surface; one `ErrorKind` enum reads better at the call site.

### D2: `events_dropped_total()` shares the saturation site with `tracing::warn!`

**Choice**: lift the existing `emit_event` (`connection_event.rs:122`) into a path that increments an `Arc<AtomicU64>` exposed via `WebSocketClient::events_dropped_total()` on both sync and async clients.

**Implementation sketch**:

- Move the saturation counter from `tracing::warn!`-only to a stored atomic owned by the client and shared with `emit_event`.
- Mirror the `MessagesDroppedTotal` plumbing in `aio/client.rs` — it already does the same thing for the message channel.
- `emit_event` becomes `emit_event(tx, counter, event)`; the `connection_event.rs` doc comment updates accordingly.

### D3: Symbols dedup is case-preserving and case-sensitive

**Choice**: `Symbols::normalized()` MUST trim whitespace + drop empty + dedup, but MUST NOT case-fold. `"TXFB6"` and `"txfb6"` remain distinct.

**Rationale**: TWSE / Fugle wire format is case-sensitive (mostly uppercase). Lowercase symbol names exist (e.g. `"txfb6"` for backwards compat with old gateway clients) and are NOT equivalent to their uppercase form server-side. Dedup MUST match the wire contract.

**Trade-off**: A user who passes `["2330", "2330", "txfb6", "TXFB6"]` gets four entries, not two. They can `.into_iter().map(str::to_ascii_uppercase).collect()` if they want case folding.

**Verification**: new scenario in `subscription-api/spec.md` plus unit test `Symbols::normalized_preserves_case`.

### D4: Stability docs on FFI-load-bearing constructors

**Choice**: add `#[must_use]` and a `## Stability` rustdoc section to:
- `ReconnectionConfig::disabled()` — load-bearing for FFI bindings to preserve historical "no auto-reconnect" semantics.
- `ReconnectionConfig::default()` — flipped from disabled to enabled in 0.4.0; bindings must not regress.
- `RetryPolicy::conservative()` / `aggressive()` — preset names are part of the API surface; renaming would break consumers.

**Rationale**: FFI bindings depend on these by name. Marking `#[must_use]` is cheap insurance against accidental no-op calls; the `## Stability` block documents the contract so future contributors don't refactor them away.

### D5: Idiomatic constructor section

**Choice**: a single "Which constructor should I use?" subsection in `core/README.md`, surfaced as crate-level rustdoc via the existing `#![doc = include_str!("../README.md")]`. Content:

- **Prefer the `bon` builder** (`SubscribeRequest::builder()`, `RetryPolicy::builder()`, `ReconnectionConfig::builder()`) when you want default-filled construction with `maybe_*` Option setters. No validation.
- **Use the positional constructor** (`ReconnectionConfig::new(max, init, max)?`) when you need validation. Returns `Result<Self, MarketDataError>`.
- **Use the typestate factory** (`WebSocketFactory::new().auth(...).stock().build()`) for endpoint derivation across stock + futopt.
- **Use the convenience constructors** (`ConnectionConfig::fugle_stock(auth)` etc.) for one-shot config in `examples/`.

Code samples are 30 lines total. Tagged `rust,ignore` since they're partial snippets.

## Risks / Trade-offs

- **[Risk] `source_kind()` for `WebSocketError` is coarse pre-0.6.0** → Mitigation: rustdoc on the helper explicitly says "WebSocket failures all return `Protocol` until 0.6.0 splits the variant; use `MarketDataError::WebSocketError { msg }`'s message text for finer detail in the interim".
- **[Risk] `events_dropped_total()` counter exposes drop-newest behaviour to consumers who didn't know about it** → Mitigation: doc comment links to the connection-event module-level docs that explain the drop-newest policy. No new behaviour, only a counter for behaviour that already exists.
- **[Risk] Stability rustdoc on `disabled()` etc. implies a deprecation contract** → Mitigation: phrasing is "FFI bindings depend on this; will be preserved across 0.x", not "frozen forever". 0.x crates are not under semver stability anyway.

## Migration Plan

1. Implement `ErrorKind` enum + `source_kind()` on `MarketDataError`. Unit-test the mapping table.
2. Refactor `connection_event::emit_event` to take the counter atomic; expose `events_dropped_total()` on both clients.
3. Document `Symbols::normalized` case policy in module rustdoc; add unit test.
4. Add `#[must_use]` + stability docs to `disabled()` / `default()` / `conservative()` / `aggressive()`.
5. Author the README "Which constructor?" section. Verify crate-level doctest still passes.
6. Bump versions: `core/Cargo.toml` 0.5.0 → 0.5.1, `rust/Cargo.toml` 0.5.0 → 0.5.1, workspace dep `marketdata-core` 0.5.0 → 0.5.1.
7. CHANGELOG entry under `[Rust 0.5.1]` with `### Added` and `### Internal` subsections; no Breaking section.
8. `cargo test --all-features` + FFI compile check + `cargo publish --dry-run` → tag + push + publish.

## Open Questions

- Should `source_kind()` be `#[non_exhaustive]` on the return enum, or `non_exhaustive` on `ErrorKind` itself? Lean **enum-level `non_exhaustive`** so consumers must include a `_` arm — gives us room to add variants in 0.6.0+ without a breaking change.
- The `events_dropped_total()` counter — atomic ordering: `Relaxed` (cheaper, sufficient for monotonic counter) or `SeqCst` (matches the message-channel counter)? Match `messages_dropped_total()`'s ordering for consistency.
