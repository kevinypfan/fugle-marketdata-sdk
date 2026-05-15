## Why

Three structural gaps survive 0.5.0 that require breaking changes or a meaningful new module — bundle for the 0.6.0 minor so consumers absorb the migration once:

- **`MarketDataError::WebSocketError { msg: String }`** swallows every `tungstenite::Error` variant. `is_retryable()` flags the whole class as retryable, but a protocol violation should never retry (it signals a version mismatch or SDK bug) whereas an IO/timeout should. Monitor incident classification needs structured access without `match`ing on substrings. The 0.5.1 `source_kind()` helper papers over the hot path but the right fix is an enum split.
- **No `pub mod testing` / `MockWsServer`.** Every consumer — monitor's `tests/integration_counter.rs`, the Python pytest suite, the JS Jest suite — re-implements a `tokio-tungstenite` echo server with subscribe ACK injection. One feature-gated module would unblock all three languages.
- **`base_url(...)` semantic divergence vs industry norm.** `RestClient::base_url(&str)` follows the OpenAI / Stripe / AWS convention (`"https://api.fugle.tw/marketdata/v1.0"` — full URL prefix before the endpoint path), but `WebSocketFactory::base_url(impl Into<String>)` is a **host root** (`"wss://api.fugle.tw/marketdata"` — SDK silently appends `/v1.0/{stock|futopt}/streaming`). Same method name, conflicting mental model. Aligning the WS factory with the rest of the SDK ecosystem makes "set `base_url` once, SDK appends the endpoint suffix" a single, transferable mental model.

## What Changes

- **BREAKING — `MarketDataError::WebSocketError` split**:
  - Replace `WebSocketError { msg: String }` with `WebSocketError { kind: WebSocketErrorKind, msg: String }`.
  - `WebSocketErrorKind` is a non-exhaustive enum: `Protocol`, `Capacity`, `Utf8`, `Tls`, `Io`, `Http(u16)`, `Other`.
  - `From<tungstenite::Error>` now routes each variant to the right kind instead of collapsing to one string.
  - `is_retryable()` updated: `Protocol`/`Capacity`/`Utf8`/`Http(401|403)` non-retryable; `Io`/`Tls`/`Http(5xx|429)`/`Other` retryable.
  - The 0.5.1 `source_kind() -> ErrorKind` helper is kept for cross-variant classification; the new `WebSocketErrorKind` is its WebSocket-specific refinement.
- **BREAKING (silent semantic) — `WebSocketFactory::base_url(...)` now expects the full base URL including the API version segment**:
  - Pre-0.6.0: `.base_url("wss://api.fugle.tw/marketdata")` → SDK appends `/v1.0/stock/streaming`.
  - 0.6.0+: `.base_url("wss://api.fugle.tw/marketdata/v1.0")` → SDK appends `/stock/streaming`.
  - Method name and signature are unchanged; **the meaning of the string the caller passes is changed**. This is a deliberate silent breaking change to align with OpenAI / Stripe / AWS / Anthropic SDK conventions. Callers who relied on the 0.5 host-root semantic produce a 404-shaped failure on first connect because the URL lacks the `/v1.0` segment.
  - `RestClient::base_url(&str)` is unchanged — it already follows the OpenAI convention.
  - `MIGRATION-0.6.md` covers the recipe (`sed -i '' 's|/marketdata"|/marketdata/v1.0"|g'` on call sites that use a custom base).
  - The `crate::urls::API_VERSION` constant is retained for default URL construction inside `WebSocketFactory::new()`, but is no longer concatenated onto user-supplied base URLs.
- **New `pub mod testing` behind `features = ["test-utils"]`**:
  - `pub struct MockWsServer` — bind to ephemeral localhost port, await client, echo subscribe ACKs with server-assigned IDs, support inject-frame / inject-delay / inject-close fixtures.
  - `pub fn url(&self) -> String` — exposes the `ws://127.0.0.1:port/...` URL the test points the SDK at.
  - Lives in `core/src/testing/mod.rs`. Compiled only when `test-utils` is enabled (default off; dev-dependency of FFI test crates).
  - Async-only — gated `cfg(all(feature = "test-utils", feature = "tokio-comp"))` so the sync-default story doesn't force an unwanted tokio dep on consumers who never test.

## Capabilities

### New Capabilities
- `testing-utilities`: defines the `pub mod testing` contract — what `MockWsServer` MUST expose, the wire protocol it implements, and the feature-flag gate.

### Modified Capabilities
- `websocket-config`: `WebSocketFactory::base_url(...)` semantic shifts from host-root (SDK appends version) to full-base-URL-including-version (SDK appends only endpoint suffix). Aligns with `RestClient::base_url`.
- `websocket-events`: `MarketDataError::WebSocketError` carries a structured `kind` field; new requirement covers the `WebSocketErrorKind` mapping from `tungstenite::Error` variants.
- `error-classification`: `source_kind()` mapping refined to honour `WebSocketErrorKind` variants (Protocol/Capacity/Utf8 → Protocol; Io → Network; Tls → Auth; Http(status) routed by status code).

## Impact

- **Affected files**:
  - `core/src/errors.rs` — new `WebSocketErrorKind` enum; reshape `WebSocketError` variant; rewire `From<tungstenite::Error>`; update `is_retryable()` and `source_kind()` mapping
  - `core/src/websocket/factory.rs` — `base_url(...)` now stores the full URL prefix; `endpoint_for(kind)` appends only `/{kind}/streaming` (no `/v1.0`)
  - `core/src/testing/mod.rs` — **new** `MockWsServer` + helpers (feature-gated)
  - `core/Cargo.toml` — new feature `test-utils = ["tokio-comp", "dep:tokio-tungstenite", "dep:futures-util"]`
  - `core/src/lib.rs` — re-export `pub mod testing` behind the feature gate
  - `tests/mock_server_smoke.rs` — new smoke test exercising the mock server
  - `MIGRATION-0.6.md` — new file documenting the silent base_url semantic change with sed-recipe
  - FFI binding crates (py/js/uniffi): switch their dev-dependencies to `features = ["test-utils"]`; delete hand-rolled echo servers (follow-up, not blocking)
- **APIs**:
  - Rust callers who pattern-match on `WebSocketError { msg }` must add `kind` (or use `..`). One mechanical fix.
  - Anyone who called `WebSocketFactory::base_url("wss://staging.example.com/marketdata")` must append `/v1.0`. **Silent break**: code still compiles, but produces a 404 on first connect.
  - FFI bindings: zero call-site impact (they construct `ConnectionConfig` directly).
- **Dependencies**: `test-utils` feature pulls existing tokio-tungstenite + futures-util crates; no new transitive deps.
- **Release**: `0.5.0 → 0.6.0` minor. `MIGRATION-0.6.md` MUST be prominent in the CHANGELOG entry because the `base_url` change is silent at compile time.
