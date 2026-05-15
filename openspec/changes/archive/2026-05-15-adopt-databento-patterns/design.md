## Context

databento-rs uses three patterns we currently lack:

1. **Typestate builder for `LiveClient::builder()`** (`databento-rs/src/live.rs:82-112`) — generic phantom parameters track which required fields are set; `build()` only compiles when every required slot is populated.
2. **`bon` v3 derive macro** — declarative builder generation across the SDK (e.g. `Subscription`, `SubmitJobParams`), including `maybe_*` setters for `Option<T>` fields.
3. **Top-level `Symbols` type** (`databento-rs/src/lib.rs:42-212`) — flexible enum with rich `From` impls, normalization helpers, and chunking support — sitting in front of the wire serialization.

Our parallels:

- `WebSocketFactory` (`core/src/websocket/factory.rs`) does runtime auth validation. Calling `.stock().build()` without a fully-formed `AuthRequest` is a runtime hazard the type system could prevent.
- `ConnectionConfigBuilder`, `SubscribeRequest` (chainable modifiers), `RetryPolicy`, and `ReconnectionConfig` are hand-rolled with non-trivial repetition. `bon = "3"` would replace ~300 LOC of boilerplate.
- `SymbolSpec` (`core/src/websocket/models/subscription.rs:36-112`) already has 9 `From` impls covering `&str` through `&[String]`. What it lacks: a normalization method, deduplication on subscription dispatch, and a chunking helper for future per-frame limits.

Constraints:

- FFI bindings (`py`, `js`, `uniffi`) must compile unchanged. They construct the final `ConnectionConfig` / `SubscribeRequest` at the boundary, not the builder — so builder shape changes don't cross the FFI line.
- Existing call sites that rely on `Into<SymbolSpec>` (e.g. `StockSubscription::new(channel, vec!["2330", "2454"])`) must still work after rename, just with a `use ... as SymbolSpec` shim or — preferred — by aliasing on first sight (`pub use Symbols as SymbolSpec` for one release, then deletion).
- 0.5.0 will be the next minor; this change must align with FutOpt expansion so all `SymbolSpec` references in new FutOpt code adopt `Symbols` from day one.

## Goals / Non-Goals

**Goals:**

- Make `WebSocketFactory` compile-checked: `.stock()` / `.futopt()` MUST be unreachable until every required field is set.
- Replace four hand-rolled builders (`ConnectionConfig`, `SubscribeRequest`, `RetryPolicy`, `ReconnectionConfig`) with `bon::Builder` derives while preserving today's public method signatures.
- Rename `SymbolSpec` → `Symbols`, move into a dedicated module, add `.normalized()` / `.iter()` / `.len()` / `.is_empty()` / `.chunked(n)` methods.
- Make subscription dispatch deduplicate symbols silently and deterministically (preserve insertion order).
- Reserve `SUBSCRIPTION_BATCH_LIMIT: Option<usize> = None` as a forward-compatible knob.

**Non-Goals:**

- `Symbols::All` and `Symbols::Ids(Vec<u32>)` variants. TWSE has no "subscribe to entire market" model and no numeric symbol IDs.
- REST API redesign. All REST endpoints take a single `&str` symbol; converting them to `Symbols` would be over-engineering.
- Implementing actual chunking. We add the helper and the const, but `SUBSCRIPTION_BATCH_LIMIT = None` means the dispatch path still emits one frame regardless of symbol count.
- Auto-generating bindings from `bon` builders. Bindings construct final config structs; they don't use Rust builder chains.

## Decisions

### D1: Typestate via marker types, not const generics

**Choice**: introduce zero-sized marker types `Unset` and `Set` (or `WithAuth`), parameterize `WebSocketFactory<State>` over them, gate `.stock()` / `.futopt()` on `WebSocketFactory<Set>`.

**Rationale**: marker types are the idiomatic Rust typestate pattern. databento-rs uses them; standard library uses them (`std::process::Command` builders elsewhere). Const generics over `bool` would also work but are less ergonomic in error messages.

**Sketch**:

```rust
pub struct Unset;
pub struct WithAuth(AuthRequest);

pub struct WebSocketFactory<S = Unset> {
    state: S,
    base: String,
}

impl WebSocketFactory<Unset> {
    pub fn new() -> Self { ... }
    pub fn auth(self, auth: AuthRequest) -> WebSocketFactory<WithAuth> { ... }
}

impl WebSocketFactory<WithAuth> {
    pub fn stock(&self) -> ConnectionConfigBuilder { ... }
    pub fn futopt(&self) -> ConnectionConfigBuilder { ... }
}
```

**Migration**: existing `WebSocketFactory::new(auth)` becomes `WebSocketFactory::new().auth(auth)`. This is a **two-call breaking change** at the Rust call site. Acceptable for 0.5.0 minor with migration note.

**Alternatives considered**:

- *Const generic over `bool`*: `WebSocketFactory<const AUTH_SET: bool>`. Works but error messages are worse ("the trait `_` is not implemented for `WebSocketFactory<false>`"). Rejected.
- *Single-call `WebSocketFactory::new(auth)` keeping current shape*: doesn't actually solve anything — auth is already required at the runtime API. Rejected because the whole point is compile-time enforcement of *future* required fields (e.g. when we add reconnect strategy as a required slot).

### D2: `bon = "3"` for parameter struct builders

**Choice**: add `bon = "3"` as a runtime dep. Derive `bon::Builder` on `ConnectionConfig`, `SubscribeRequest`, `RetryPolicy`, `ReconnectionConfig`. Delete corresponding hand-rolled builder modules.

**Rationale**: `bon` v3 generates idiomatic builders with `maybe_*` setters for `Option<T>` fields (databento-rs uses this). It supports default values via `#[builder(default = ...)]` attribute, matching today's `ConnectionConfigBuilder::message_buffer` default of 4096.

**Public API preservation**:

- `ConnectionConfig::builder(url, auth)` style is preserved by `bon`'s `#[builder(start_fn = builder)]` attribute and positional `#[builder(field)]` markers — but bon's default `builder()` signature is method-less and chains required fields. **Decision**: use `bon`'s standard form (`ConnectionConfig::builder().url(...).auth(...).build()`) and add a thin `pub fn builder(url, auth)` wrapper for backward compat only if it's needed at call sites. Most call sites already chain.
- `SubscribeRequest::with_symbols(channel, symbols)` is kept as a hand-written constructor that calls the bon builder internally (because it has the dedup contract; see D3).

**Compile-time cost**: adds one proc-macro crate. `bon` is small (~10kB compiled); estimated +5% incremental build time on `core`. Acceptable.

**Alternatives considered**:

- *`typed-builder`*: older, no `maybe_*` setters, less active development. Rejected.
- *Hand-rolled builders forever*: 300 LOC of boilerplate + the same amount in tests. Rejected for boilerplate cost and divergence risk between builders.
- *No builder at all, direct struct construction*: loses default-value ergonomics and forces every consumer to spell out every field. Rejected for ergonomics.

### D3: `Symbols` module with normalization + dedup contract

**Choice**: move the enum from `core/src/websocket/models/subscription.rs:36-112` into `core/src/websocket/models/symbols.rs`. Rename to `Symbols`. Add:

```rust
impl Symbols {
    pub fn normalized(self) -> Self;           // trim each + dedup, preserve first-seen order
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    pub fn iter(&self) -> impl Iterator<Item = &str>;
    pub fn chunked(self, max_per_chunk: usize) -> Vec<Symbols>;
}

pub const SUBSCRIPTION_BATCH_LIMIT: Option<usize> = None;
```

Plus all existing `From` impls retargeted to `Symbols`. `SubscribeRequest::with_symbols(channel, symbols)` MUST call `.normalized()` before building the `SubscribeRequest`.

**Dedup spec**:

- Whitespace trim on each entry first (`"  2330  "` → `"2330"`).
- After trimming, identical entries collapse to the first occurrence.
- Empty strings (post-trim) are dropped silently — they were never valid subscription targets.
- `Single("2330")` and `Many(vec!["2330"])` are equivalent post-normalize; we choose to normalize `Many` of length 1 down to `Single` for canonical form, so equality (`PartialEq`) is meaningful.

**Why dedup is non-breaking-in-practice**: today, `SubscribeRequest::with_symbols(Channel::Trades, vec!["2330", "2330"])` calls `expand()` to produce two `SubscribeRequest`s, each tracked separately in `SubscriptionManager`. The server returns two ACKs, and we record two entries with different local keys. This is almost certainly unintentional in all known call sites; making it deterministically one subscription matches user intent.

**Chunking is unused for now**: `SUBSCRIPTION_BATCH_LIMIT = None` means `chunked(n)` is a public helper for downstream code that may know about server limits we don't, but the SDK itself doesn't chunk on dispatch. If/when the Fugle gateway documents a per-frame limit, the dispatch path adds one line: `for chunk in symbols.chunked(LIMIT) { send(chunk) }`.

**Alternatives considered**:

- *Keep `SymbolSpec` name*: rejected — `Symbols` is shorter, matches databento-rs, and signals the type is the canonical answer to "how do I express one or many symbols", not an alternate spec.
- *Make dedup opt-in*: rejected — silent duplicate subscriptions are a footgun. Default-on dedup is the safer behavior.
- *Add `Symbols::All`*: rejected per Non-Goals (no TWSE semantics).

### D4: Backward-compat alias?

**Choice**: do NOT add `pub use Symbols as SymbolSpec;` re-export.

**Rationale**: the 0.5.0 minor already breaks the factory API (D1). Bundling `SymbolSpec` rename into the same minor is one clear migration note, not two. Aliases would just delay the rename without saving meaningful migration cost.

**Migration note**: the `MIGRATION-0.5.md` doc (created by tasks.md step) MUST include a `sed`-style search/replace recipe:

```text
# In your crate:
sed -i '' 's/SymbolSpec/Symbols/g' src/**/*.rs
```

## Risks / Trade-offs

- **[Risk] Typestate API surface change breaks downstream Rust callers** → Mitigation: callers use `WebSocketFactory::new(auth)` today (one-call); the migration is mechanical (`new(auth)` → `new().auth(auth)`). Documented in MIGRATION-0.5.md.
- **[Risk] `bon` macro errors are harder to debug than hand-rolled builders** → Mitigation: keep `bon` derives small per struct, default-values explicit; `cargo expand` is the debugging tool of last resort. `bon` has excellent error messages compared to most builder macros (it's a 2023+ codebase).
- **[Risk] Dedup semantically changes "send two subscribe requests for the same symbol"** → Mitigation: not a real use case; document the dedup contract explicitly in `Symbols::normalized` rustdoc; one CHANGELOG line.
- **[Risk] Chunking helper without a real limit is dead code** → Mitigation: it's six lines and `#[doc(hidden)]` is an option if rustdoc clutter becomes a concern. Current decision: keep it public; it's a stable forward-compat surface.
- **[Risk] `bon` adds a proc-macro to the dep tree** → Mitigation: `bon` is well-maintained and small; one of the cleanest builder crates in the ecosystem.

## Migration Plan

1. **`bon` first** (lowest risk): add dep, derive on `RetryPolicy` and `ReconnectionConfig` (smallest blast radius), update their tests. Land as a single commit; no public API change visible.
2. **`Symbols` rename + module split**: new file `core/src/websocket/models/symbols.rs`, move enum + From impls, add helpers. Delete the old declaration. Update `models/mod.rs` re-export.
3. **Dedup wiring**: `SubscribeRequest::with_symbols` calls `.normalized()`. Add tests covering duplicate-collapse and whitespace-trim. Verify `SubscriptionManager` count matches normalized length.
4. **`ConnectionConfig` + `SubscribeRequest` to `bon::Builder`**: keep `with_symbols` hand-written (it's the dedup gate). Replace `ConnectionConfigBuilder` with `bon`.
5. **Typestate `WebSocketFactory`**: introduce `Unset` / `WithAuth` markers, rewire `new` to `new().auth(...)`. Update existing tests.
6. **MIGRATION-0.5.md**: write the migration doc with `sed` recipes for both the factory change and the `SymbolSpec → Symbols` rename.
7. **CHANGELOG.md**: add 0.5.0 section with `[BREAKING]` markers.
8. **FFI binding smoke check**: `cargo build -p fugle-marketdata-py -p fugle-marketdata-js -p fugle-marketdata-uniffi` must still pass without source changes in those crates.

**Rollback**: revert in three commits matching steps 5→4→1. Each step is a complete, self-contained refactor.

## Open Questions

- Do we want to expose `Symbols::chunked` from the top-level crate re-export? Probably yes (it's user-facing forward compat), but verify no name collisions with `chunks` from std.
- Should `SubscribeRequest::with_symbols` log (at `tracing::warn!`) when dedup collapses entries? Lean **no** — silent normalization is the contract — but discuss during implementation.
