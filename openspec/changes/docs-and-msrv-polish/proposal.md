## Why

The crate is approaching crates.io maturity (0.4.0 shipped production-readiness), but lacks the documentation discipline and MSRV declaration that downstream Rust consumers and `docs.rs` rendering depend on. Adopting databento-rs's documentation policy (strict lints, all-features rendering, README-as-crate-docs, declared MSRV) is a non-breaking polish pass that improves discoverability and prevents doc rot before 0.5.0.

## What Changes

- **Strict documentation lints**: enable `#![deny(missing_docs, rustdoc::broken_intra_doc_links, clippy::missing_errors_doc)]` in `core/src/lib.rs` and backfill missing doc comments on all public items.
- **`docs.rs` all-features rendering**: switch `[package.metadata.docs.rs]` from `features = ["tokio-comp"]` to `all-features = true` plus `rustdoc-args = ["--cfg", "docsrs"]`, and annotate `tokio-comp` / `tracing`-gated public items with `#[cfg_attr(docsrs, doc(cfg(feature = "...")))]` so feature badges render correctly.
- **README-as-crate-docs**: replace the hand-maintained crate-level docs in `core/src/lib.rs` with `#![doc = include_str!("../README.md")]`, marking README code blocks `ignore` or `no_run` as appropriate so doctests stay green.
- **Declared MSRV**: add `rust-version = "1.82"` to `core/Cargo.toml` (already implicitly required by `napi 3.4`) and add a CI job that verifies the crate builds on the pinned MSRV toolchain.
- **Non-breaking**: no runtime behavior, no public API surface changes.

## Capabilities

### New Capabilities
- `documentation-policy`: declares strict doc-coverage lints, MSRV pin, `docs.rs` rendering config, and README-as-crate-docs as enforced project requirements rather than ad-hoc conventions.

### Modified Capabilities
(none — no existing capability changes its runtime contract)

## Impact

- **Affected files**:
  - `core/Cargo.toml` — `rust-version`, `[package.metadata.docs.rs]` block
  - `core/src/lib.rs` — `#![deny(...)]`, `#![doc = include_str!(...)]`, `#![cfg_attr(docsrs, feature(doc_cfg))]`
  - `core/src/**/*.rs` — backfill doc comments on public items; add `#[cfg_attr(docsrs, doc(cfg(feature = "...")))]` to gated items
  - `core/README.md` — verify code blocks compile or are tagged appropriately for inclusion as doctests
  - `.github/workflows/*.yml` — add MSRV verification job
- **APIs**: no runtime API changes; documentation surface improves
- **Dependencies**: none added
- **Downstream**: Rust users on toolchains older than 1.82 will be blocked by Cargo (with a clear error); FFI bindings (`py`, `js`, `uniffi`) unaffected since they consume `core` via path
- **Release**: 0.4.1 patch (or rolled into 0.5.0 if it lands first)
