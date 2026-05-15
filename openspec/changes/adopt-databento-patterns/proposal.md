## Why

Comparing fugle-marketdata-sdk against databento-rs surfaced three high-value patterns we should adopt before 0.5.0 lands: compile-time enforcement of required factory fields (currently runtime-only), declarative builder generation via the `bon` crate (currently hand-written boilerplate), and a normalized `Symbols` type that deduplicates subscription targets (currently silently sends duplicate frames). Bundling these into 0.5.0 minor alongside FutOpt expansion keeps Rust-side breaking changes to a single release.

## What Changes

- **BREAKING — Typestate `WebSocketFactory` builder**: convert `WebSocketFactory` from runtime-validated config to a phantom-generic typestate builder so `.stock()` / `.futopt()` only become callable once `auth` (and any other required field) is set. Compile-time error replaces today's runtime panic / silent default.
- **`bon` crate adoption for parameter builders**: replace hand-written builders on `ConnectionConfig`, `SubscribeRequest`, `RetryPolicy`, and `ReconnectionConfig` with `#[derive(bon::Builder)]`. Public method signatures stay equivalent; `Option<T>` fields gain `maybe_*` convenience setters. Hand-written builder methods are deleted.
- **BREAKING — `SymbolSpec` → `Symbols` rename, with dedup + helpers**:
  - Rename `SymbolSpec` to `Symbols`, move from `core/src/websocket/models/subscription.rs` into a dedicated `core/src/websocket/models/symbols.rs` module.
  - Add `Symbols::normalized(self) -> Self` (trim whitespace + dedup preserving insertion order), `Symbols::len`, `Symbols::is_empty`, `Symbols::iter`, `Symbols::chunked(n)`.
  - `SubscribeRequest::with_symbols(...)` and channel-specific subscription constructors MUST run `.normalized()` internally so duplicate symbols collapse to one subscription before reaching `SubscriptionManager`.
  - Expose `pub const SUBSCRIPTION_BATCH_LIMIT: Option<usize> = None` in the new module as future-proofing; current behavior remains single-frame batch with no chunking.
  - Existing `Into<SymbolSpec>` impls (`&str`, `String`, `&String`, `Vec<String>`, `Vec<&str>`, `[&str; N]`, `[String; N]`, `&[&str]`, `&[String]`) all retarget the renamed type so call sites compile unchanged after a single `use` rename.
- **Out of scope**: `Symbols::All` / `Symbols::Ids(Vec<u32>)` variants (TWSE has no equivalent concept). REST API surface is unchanged — all REST endpoints accept a single `&str` symbol and stay that way.

## Capabilities

### New Capabilities
(none — all changes refine existing capabilities)

### Modified Capabilities
- `websocket-config`: `WebSocketFactory` gains typestate guarantees; the existing scenarios documenting `WebSocketFactory::new(auth).stock().build()` etc. must continue to compile post-change, but invoking `.stock()` before `auth` is set MUST fail at compile time.
- `subscription-api`: introduces `Symbols` (renamed from `SymbolSpec`) with explicit dedup contract; `SubscribeRequest::with_symbols(...)` MUST collapse duplicates before subscription dispatch.

## Impact

- **Affected files**:
  - `core/src/websocket/factory.rs` — phantom-generic state types, `stock()` / `futopt()` constrained to the populated state
  - `core/src/websocket/connection_config.rs` — `bon::Builder` derive replacing `ConnectionConfigBuilder` hand-roll
  - `core/src/websocket/subscription.rs` — split: keep `SubscribeRequest` here; move symbol type to new module
  - `core/src/websocket/models/symbols.rs` — **new** module containing `Symbols` enum, impls, helpers, constant
  - `core/src/websocket/models/mod.rs` — re-export `Symbols`
  - `core/src/rest/retry.rs` — `RetryPolicy` switches to `bon::Builder`
  - `core/src/websocket/reconnection.rs` — `ReconnectionConfig` switches to `bon::Builder`
  - `core/src/lib.rs` — public re-export `Symbols` (replacing `SymbolSpec`)
  - `core/Cargo.toml` — add `bon = "3"` dependency
- **APIs**:
  - Rust users who imported `SymbolSpec` must rename their `use` to `Symbols`; no other call-site changes if they used `Into` conversions
  - Direct `WebSocketFactory` constructor callers using `auth: None` or partial state must finish initialization before `.stock()` / `.futopt()` (caught at compile time)
  - FFI bindings (`py`, `js`, `uniffi`) unaffected — they construct config structs at the boundary, not the builder
- **Dependencies**: `bon = "3"` (proc-macro, ~adds compile time but no runtime cost)
- **Release**: 0.5.0 minor (paired with FutOpt completion). Migration guide must list `SymbolSpec → Symbols` rename and the new typestate ordering.
