## Context

The 0.5.1 patch (sibling change `core-0-5-1-followups`) added a `source_kind()` helper that papers over `WebSocketError`'s string-only shape but does not fix it. Three structural issues survive:

1. **`MarketDataError::WebSocketError { msg }` collapses every `tungstenite::Error` variant** into a single retryable bucket. Protocol violations (frame format, capacity exceeded, UTF-8 decode) should never retry — they signal an SDK bug or version mismatch — but `is_retryable()` returns `true` for the whole class.
2. **No `pub mod testing` / `MockWsServer`** means every consumer reinvents an echo server. The monitor crate, the Python pytest suite, and the JS Jest suite all have their own implementations.
3. **`base_url(...)` semantic divergence**. `RestClient::base_url(&str)` already follows the OpenAI / Stripe / AWS / Anthropic convention — the caller passes the full URL prefix including the API version segment. `WebSocketFactory::base_url(...)` is the odd one: it expects a host root and silently injects `/{API_VERSION}`. Aligning the WS factory with the rest of the ecosystem gives "set base_url once, append endpoint" as a single transferable mental model.

Stakeholders:
- **Monitor integration**: needs reliable error classification at the disconnect site; today greps stringified messages.
- **All three FFI binding test suites**: each maintain a hand-rolled echo server; rust mock server would replace them.
- **First-time SDK consumers**: hit two builder paths and (post-0.6) one consistent `base_url` semantic across REST + WS.

Constraints:
- 0.6.0 is a minor — breaking changes allowed but must come with a migration guide.
- FFI bindings (py/js/uniffi) must still compile after the `WebSocketError` reshape — they construct `ConnectionConfig` at the boundary and don't pattern-match on this variant.
- `pub mod testing` must be feature-gated so production builds don't pull in `tokio-tungstenite` for non-test code paths.

## Goals / Non-Goals

**Goals:**
- Replace `WebSocketError { msg: String }` with `WebSocketError { kind: WebSocketErrorKind, msg: String }`.
- Map `tungstenite::Error` variants to a structured kind on construction, not at error-handling time.
- Rewire `is_retryable()` to honour the new kind — protocol violations are never retryable.
- Ship `core::testing::MockWsServer` behind `features = ["test-utils"]` with enough surface (subscribe ACK, inject frame, close-on-cue) for monitor + FFI binding tests.
- Align `WebSocketFactory::base_url(...)` semantics with `RestClient::base_url(...)` and industry SDKs — full URL prefix including the API version segment.

**Non-Goals:**
- Renaming `RestClient::base_url(&str)`. It already follows the right convention.
- Renaming `WebSocketFactory::base_url(...)` to `host_root` or anything else. Keeping the OpenAI-canonical name is the whole point.
- Building a full mock REST server. Out of scope for this change.
- Reshaping `MarketDataError` beyond the WebSocket variant.

## Decisions

### D1: `WebSocketErrorKind` is a non-exhaustive enum, not a bitfield

**Choice**:

```rust
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WebSocketErrorKind {
    /// Protocol-level violation: malformed frame, illegal state transition,
    /// reserved bits set. Never retryable — indicates an SDK / version
    /// mismatch.
    Protocol,
    /// Frame too large for the configured `max_message_size` / `max_frame_size`.
    /// Never retryable; the producer is misbehaving.
    Capacity,
    /// UTF-8 decoding failed on a text frame. Never retryable.
    Utf8,
    /// TLS / certificate failure during handshake. Authentication-adjacent.
    Tls,
    /// Transport IO failure: connection reset, EOF, write error. Retryable.
    Io,
    /// HTTP error during handshake. `u16` is the status code.
    Http(u16),
    /// Anything `tungstenite` adds in the future, or an error we can't classify.
    Other,
}
```

**Mapping** from `tungstenite::Error`:

| `tungstenite::Error` variant | `WebSocketErrorKind` |
|---|---|
| `Protocol(_)` | `Protocol` |
| `Capacity(_)` | `Capacity` |
| `Utf8(_)` | `Utf8` |
| `Tls(_)` | `Tls` |
| `Io(_)`, `ConnectionClosed`, `AlreadyClosed` | `Io` |
| `Http(resp)` | `Http(resp.status().as_u16())` |
| any other / future | `Other` |

**Rationale**: Mirrors `tungstenite::Error`'s own categorization without leaking the dependency type. `Http(u16)` is enum-not-struct because callers want to `match err.kind() { WebSocketErrorKind::Http(401 | 403) => ... }`.

**Alternatives considered**:
- *Flat enum with `Http401`, `Http403`, `Http5xx`*: rejected — the SDK doesn't know which codes the server will produce; carrying the raw `u16` is more honest.
- *Re-export `tungstenite::Error` directly*: rejected — leaks an internal dep across a stability boundary; complicates feature gating.

### D2: `is_retryable()` honours `WebSocketErrorKind`

**Choice**: `WebSocketError { kind, .. }`:

| Kind | Retryable |
|---|---|
| `Protocol`, `Capacity`, `Utf8` | **no** |
| `Tls` | **no** (cert issues need human attention) |
| `Io` | **yes** |
| `Http(429)`, `Http(5xx)` | **yes** |
| `Http(401\|403)` | **no** |
| `Http(other)` | **no** |
| `Other` | **yes** (conservative) |

`MarketDataError::source_kind()` (added in 0.5.1) updates its mapping:

- `WebSocketError { kind: Protocol | Capacity | Utf8 }` → `ErrorKind::Protocol`
- `WebSocketError { kind: Tls }` → `ErrorKind::Auth`
- `WebSocketError { kind: Io }` → `ErrorKind::Network`
- `WebSocketError { kind: Http(..) }` → mirror the HTTP-status mapping for `ApiError`

### D3: `core::testing::MockWsServer` shape

**Choice**:

```rust
#[cfg(all(feature = "test-utils", feature = "tokio-comp"))]
pub mod testing {
    pub struct MockWsServer { /* private */ }

    impl MockWsServer {
        /// Bind to an ephemeral 127.0.0.1 port and start accepting one client.
        pub async fn start() -> Self { ... }

        /// `ws://127.0.0.1:<port>/marketdata/v1.0/stock/streaming` (auth
        /// is accepted as anything non-empty — the mock validates shape,
        /// not content).
        pub fn url(&self) -> String { ... }

        /// Force the next subscribe ACK to assign `server_id`. Useful for
        /// asserting `SubscriptionManager` records the right key.
        pub fn next_subscribe_id(&self, id: impl Into<String>) { ... }

        /// Push a frame the client receives on its next `messages().recv()`.
        pub async fn inject_frame(&self, frame: StreamMessage) { ... }

        /// Server closes the connection with `code` / `reason`.
        pub async fn close(&self, code: u16, reason: impl Into<String>) { ... }

        /// Address consumers can construct a config against (sync sugar).
        pub fn address(&self) -> std::net::SocketAddr { ... }
    }

    /// Convenience: spin up server + async `aio::WebSocketClient`.
    pub async fn aio_pair() -> (MockWsServer, crate::aio::WebSocketClient) { ... }
}
```

**Feature gate**: `test-utils = ["tokio-comp", "dep:tokio-tungstenite", "dep:futures-util"]`. The mock is async-only because building a synchronous tokio-tungstenite server inside the sync sibling tests would force tokio runtime construction either way.

**Lives in**: `core/src/testing/mod.rs`. Smoke test at `tests/mock_server_smoke.rs`.

### D4: `base_url(...)` semantic alignment with OpenAI convention

**Choice**: `WebSocketFactory::base_url(impl Into<String>)` now stores the full URL prefix that endpoint paths are appended to. The factory's `endpoint_for(kind)` helper appends only `/{kind}/streaming`; the `{API_VERSION}` segment is no longer injected.

**Before / after**:

| Call | Pre-0.6.0 result | 0.6.0+ result |
|---|---|---|
| `.base_url("wss://api.fugle.tw/marketdata")` then `.stock().build().url` | `wss://api.fugle.tw/marketdata/v1.0/stock/streaming` | `wss://api.fugle.tw/marketdata/stock/streaming` (404) |
| `.base_url("wss://api.fugle.tw/marketdata/v1.0")` then `.stock().build().url` | `wss://api.fugle.tw/marketdata/v1.0/v1.0/stock/streaming` (404) | `wss://api.fugle.tw/marketdata/v1.0/stock/streaming` ✓ |
| `WebSocketFactory::new().auth(a).stock().build().url` (no `.base_url(...)`) | `wss://api.fugle.tw/marketdata/v1.0/stock/streaming` | `wss://api.fugle.tw/marketdata/v1.0/stock/streaming` (unchanged) |

**Default-URL construction** in `WebSocketFactory::new()` continues to use `urls::WS_BASE_ROOT` + `/{API_VERSION}` internally so the no-override path stays byte-identical. The `API_VERSION` const is retained but is no longer concatenated onto user-supplied base URLs.

**Rationale**: aligns with `RestClient::base_url` (already OpenAI-style), and with the OpenAI Python SDK (`base_url="https://api.openai.com/v1"`), Stripe SDK, AWS SDK endpoint URLs, Anthropic SDK. Users who learn one transferable rule — "`base_url` is the everything-before-the-endpoint-path prefix" — apply it everywhere.

**Risk: silent breaking change**. 0.5.0 consumers with custom `base_url(...)` call sites compile fine in 0.6.0 but produce a 404 on first WebSocket connect because their URL lacks the `/v1.0` segment. Mitigations:

1. Section header in `MIGRATION-0.6.md` is **`### SILENT BREAKING: WebSocketFactory::base_url`** — first content the migration doc surfaces.
2. CHANGELOG entry under `### Breaking` carries the `[SILENT]` tag with explicit before/after example.
3. Optional: emit `tracing::warn!` on `WebSocketFactory::base_url(...)` when the supplied string does not contain `/v1` or `/v2` (heuristic) — see Open Questions.

**Alternatives considered**:
- *Rename to `host_root(...)`*: rejected — keeps the host-root semantic but loses the transferable OpenAI mental model. Worse onboarding for users who know other SDKs.
- *Add a new method `endpoint_base(...)` with the new semantic, deprecate `base_url(...)`*: rejected — introduces a third name in 0.6.0, and users still hit the wrong default if they grab `base_url` from autocomplete.
- *Keep current semantic, fix docs only*: rejected — the name collision with `RestClient::base_url` is a real footgun.

### D5: `MIGRATION-0.6.md` covers two breaks

**Choice**: ship a migration guide listing:
1. **`base_url` semantic change** (SILENT) — with sed recipe and before/after URL table.
2. `WebSocketError` pattern matches now need `{ kind, msg }` or `{ kind, .. }`.

Plus an additive note: `core::testing::MockWsServer` is available for downstream test crates that previously rolled their own server.

## Risks / Trade-offs

- **[Risk] Silent `base_url` semantic shift produces production 404 with no compile warning** → Mitigation: prominent MIGRATION-0.6.md entry, CHANGELOG `[SILENT BREAKING]` tag, and an optional `tracing::warn!` heuristic (off by default, lights up under the `tracing` feature when supplied URL doesn't contain `/v1` or `/v2`). Encourage staging deploys before prod cutover in release notes.
- **[Risk] Refining `is_retryable()` could change observable behaviour mid-session** → Mitigation: explicit MIGRATION-0.6.md table comparing 0.5 vs 0.6 retry verdicts.
- **[Risk] `MockWsServer` API drifts from real server** → Mitigation: smoke test asserts the mock implements the same subscribe/ACK protocol as `protocol.rs`. Drift caught by CI.
- **[Risk] `test-utils` feature pulls heavy deps for end users who flip it on by accident** → Mitigation: feature is **off by default** and gated on `tokio-comp`. Production builds never see it.

## Migration Plan

1. Implement `WebSocketErrorKind` + reshape `MarketDataError::WebSocketError`. Rewire `From<tungstenite::Error>`.
2. Update `is_retryable()` and `source_kind()` mappings to the new kind. Update tests.
3. Build `core::testing::MockWsServer` behind `features = ["test-utils"]`. Smoke test.
4. Shift `WebSocketFactory::base_url(...)` to OpenAI-style semantic: store the full prefix; `endpoint_for(kind)` appends only `/{kind}/streaming`. Update internal default-URL construction to bypass the user override path.
5. (Optional) Add `tracing::warn!` heuristic detecting URLs without `/v[0-9]` segment under the `tracing` feature.
6. Write `MIGRATION-0.6.md`. Update CHANGELOG with `[SILENT BREAKING]` tag.
7. Bump versions to 0.6.0 across `core/Cargo.toml`, `rust/Cargo.toml`, and workspace dep `marketdata-core`.
8. Validate: `cargo test --all-features`, `cargo build --features test-utils`, FFI binding builds, `cargo publish --dry-run`.

**Rollback**: revert in three commits matching steps 4→3→1. Each step is self-contained.

## Open Questions

- Should the optional `tracing::warn!` URL-shape heuristic be implemented in 0.6.0, or deferred? Lean **defer to 0.6.1** — fewer moving parts in the headline release, and we can survey early consumers' reaction first.
- Should `MockWsServer` accept multiple clients in a single instance, or one-shot per `start()`? Lean **one-shot** — easier semantics for tests; can iterate.
