## Context

`fugle-marketdata-core` 0.4.0 shipped production-readiness (tracing feature, redaction, graceful shutdown, reconnect default) but the **publish-readiness layer** — declared MSRV, strict doc coverage, `docs.rs` feature badges, README-as-crate-docs — is still ad-hoc. databento-rs's `src/lib.rs` and `Cargo.toml` demonstrate the conventional Rust SDK pattern; this change adopts it verbatim where idiomatic.

Stakeholders:
- **Rust crate consumers**: get a stable MSRV contract and complete API docs on docs.rs with correct feature badges.
- **FFI binding crates (`py`, `js`, `uniffi`)**: indirectly benefit — their build matrix is already on newer toolchains.
- **Future contributors**: doc lints prevent doc rot as the surface grows.

Constraints:
- No runtime behavior changes. This is purely metadata, lint, and docs.
- MSRV must be ≤ the minimum that `napi 3.4.0` (Node binding) implicitly requires — that's 1.82 based on current `Cargo.lock`.
- README must compile cleanly as crate-level rustdoc (or have its non-runnable blocks tagged).

## Goals / Non-Goals

**Goals:**
- Declare `rust-version = "1.82"` in `core/Cargo.toml` and verify in CI.
- Enforce `missing_docs` + `rustdoc::broken_intra_doc_links` + `clippy::missing_errors_doc` on every public item in `core/`.
- Render docs.rs with **all features enabled** and **feature badges** on `tokio-comp` / `tracing`-gated items.
- Replace the hand-maintained crate-doc block in `core/src/lib.rs` with `#![doc = include_str!("../README.md")]`.

**Non-Goals:**
- Backfilling docs in `py`, `js`, `uniffi`, or `gui-web/src-tauri` crates (separate concern; only `core` is published to crates.io today).
- Changing public API. Any item that lacks docs gets a doc comment, not a rename or visibility change.
- Lifting MSRV beyond what is implicitly required today; this change *declares* MSRV, it does not *raise* it.
- Adding new doctests beyond what naturally appears in the README.

## Decisions

### D1: MSRV = 1.82

**Choice**: pin `rust-version = "1.82"` in `core/Cargo.toml`.

**Rationale**: `napi 3.4.0` (declared in Node binding) requires 1.82. Pinning lower would be a lie; pinning higher would needlessly cut off consumers. Cargo enforces this at resolve time — older toolchains get a clear error pointing at the manifest, not a cryptic build failure.

**Alternatives considered**:
- *No MSRV declaration* (current state): silent compatibility drift; users hit random build errors as transitive deps bump their MSRVs. Rejected.
- *MSRV = 1.75 (oldest known-working)*: would require a separate compat matrix and forbid features from newer std. Not worth the audit cost for theoretical compat we don't observe demand for.

### D2: `#![deny(missing_docs, rustdoc::broken_intra_doc_links, clippy::missing_errors_doc)]`

**Choice**: enable all three lints at the `core/src/lib.rs` crate root, not just `warn`.

**Rationale**: `deny` is the only setting that fails CI. `warn` rots — warnings get ignored and accumulate. databento-rs uses `deny`; this is the conventional choice for production SDKs.

**Trade-off**: one-shot backfill cost (estimated 100-200 doc comments across `core/src/`). After backfill, the marginal cost per new public item is one line of doc-comment, which is cheap.

**Alternatives considered**:
- *`warn` instead of `deny`*: documented to rot; rejected.
- *`deny` only on `missing_docs`*: rejected — broken intra-doc links and undocumented errors are equally damaging to docs.rs UX.

### D3: `docs.rs` all-features + `docsrs` cfg gate

**Choice**: switch `[package.metadata.docs.rs]` to `all-features = true` plus `rustdoc-args = ["--cfg", "docsrs"]`, and annotate every `#[cfg(feature = "...")]` public item with `#[cfg_attr(docsrs, doc(cfg(feature = "...")))]`.

**Rationale**: today's `features = ["tokio-comp"]` setting omits the `tracing` feature surface from docs.rs and renders no feature badges. databento-rs uses the `docsrs` cfg pattern so users immediately see "this is gated behind `--features tokio-comp`" in the docs.

**Required `lib.rs` addition**: `#![cfg_attr(docsrs, feature(doc_cfg))]` (uses unstable feature `doc_cfg`, gated by `cfg(docsrs)` so stable builds are unaffected — this is the same trick databento-rs uses).

### D4: README-as-crate-docs

**Choice**: replace existing crate-level docs in `core/src/lib.rs` with `#![doc = include_str!("../README.md")]`.

**Rationale**: eliminates duplicate documentation maintenance. The README is the landing page on crates.io *and* on docs.rs, identical text.

**Risks**:
- README code blocks become doctests by default. Existing fenced blocks may not compile (e.g. they use `...` placeholders or skip imports).
- **Mitigation**: audit `core/README.md` once; tag non-runnable blocks with `ignore`, partial-snippet blocks with `no_run`, and import-heavy blocks with `# use ...` hidden imports. Run `cargo test --doc -p fugle-marketdata-core` to verify.

### D5: MSRV CI job

**Choice**: add a `msrv` job in `.github/workflows/*.yml` (whichever workflow runs the Rust matrix) that installs `1.82` via `actions-rs/toolchain` (or `dtolnay/rust-toolchain@1.82`) and runs `cargo build --all-features -p fugle-marketdata-core`.

**Rationale**: declaring MSRV in `Cargo.toml` without CI verification means it can silently break on a dep bump. The MSRV job runs only `build` (not `test`) because tests may use newer toolchain features without affecting downstream consumers.

**Trade-off**: adds one CI matrix entry (~1 min CI time). Acceptable.

## Risks / Trade-offs

- **[Risk] Doc-backfill PR is large and review-heavy** → Mitigation: land in topic order (module-by-module commits) so reviewers can chunk it. Pure additive comments; no functional risk.
- **[Risk] README doctests fail after `include_str!`** → Mitigation: run `cargo test --doc` locally before opening PR; tag blocks as needed.
- **[Risk] `doc_cfg` is unstable, requires `#![cfg_attr(docsrs, feature(doc_cfg))]`** → Mitigation: this is the canonical pattern (used by `tokio`, `databento-rs`, hundreds of others). Only active when the `docsrs` cfg is set, which is only set by docs.rs itself.
- **[Risk] MSRV pin breaks niche consumer** → Mitigation: 1.82 is already implicitly required via `napi 3.4`. No regression vs. status quo.

## Migration Plan

1. Add `rust-version = "1.82"` to `core/Cargo.toml`. Update `[package.metadata.docs.rs]`.
2. Add `#![cfg_attr(docsrs, feature(doc_cfg))]` + `#![deny(...)]` + `#![doc = include_str!("../README.md")]` to `core/src/lib.rs`. Remove the existing hand-maintained crate-doc block.
3. Annotate `aio` module + every `tokio-comp` / `tracing`-gated public item with `#[cfg_attr(docsrs, doc(cfg(feature = "...")))]`.
4. Audit `core/README.md` code blocks; tag non-runnable blocks.
5. Backfill doc comments until `cargo doc --all-features -p fugle-marketdata-core` and `cargo clippy --all-features -p fugle-marketdata-core -- -D warnings` are clean. Land module-by-module.
6. Add MSRV CI job.
7. Verify `cargo +1.82 build --all-features -p fugle-marketdata-core` succeeds locally before merge.
8. Tag 0.4.1 (or roll into 0.5.0 if it lands first; this change is forward-compatible with either).

**Rollback**: revert manifest + `lib.rs` attribute additions in a single commit. No data, no state, no migration.

## Open Questions

- Should `bins` examples (`core/examples/`) also enforce `missing_docs`? Lean **yes** for consistency, but they don't appear on docs.rs so cost/benefit is lower. Defer to implementation.
- Does CI currently run `cargo doc` with `-D warnings`? If not, add it; otherwise lint enforcement is best-effort.
