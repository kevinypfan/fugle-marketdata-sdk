## 1. MSRV declaration & CI

- [x] 1.1 Add `rust-version = "1.82"` under `[package]` in `core/Cargo.toml`
- [ ] 1.2 Run `cargo +1.82 build --all-features -p fugle-marketdata-core` locally to confirm pin compiles _(skipped — 1.82 toolchain not installed locally; CI MSRV job (1.3) covers this)_
- [x] 1.3 Add MSRV job to existing GitHub Actions workflow: install Rust 1.82 via `dtolnay/rust-toolchain@1.82`, run `cargo build --all-features -p fugle-marketdata-core`
- [ ] 1.4 Verify MSRV job runs (push to a feature branch, observe green) _(deferred — requires push)_

## 2. docs.rs metadata + docsrs cfg

- [x] 2.1 Replace `[package.metadata.docs.rs]` in `core/Cargo.toml` with `all-features = true` + `rustdoc-args = ["--cfg", "docsrs"]`
- [x] 2.2 Add `#![cfg_attr(docsrs, feature(doc_cfg))]` to `core/src/lib.rs` (top of file, before lints)
- [x] 2.3 Annotate `aio` module declaration with `#[cfg_attr(docsrs, doc(cfg(feature = "tokio-comp")))]` _(pre-existing)_
- [x] 2.4 Grep for every `#[cfg(feature = "tokio-comp")]` in `core/src/` on public items; add matching `#[cfg_attr(docsrs, doc(cfg(...)))]` _(all sites already annotated)_
- [x] 2.5 Grep for every `#[cfg(feature = "tracing")]` on public items; add matching `#[cfg_attr(docsrs, doc(cfg(...)))]` _(no `pub` items gated on `tracing`; `tracing_compat` is `pub(crate)` only)_
- [ ] 2.6 Run `RUSTDOCFLAGS="--cfg docsrs" cargo +nightly doc --all-features -p fugle-marketdata-core` locally to preview feature badges _(deferred — manual visual check)_

## 3. README as crate docs

- [x] 3.1 Audit `core/README.md` code blocks; tagged all 8 `rust` blocks `rust,ignore` (REST calls, async tokio::main, partial snippets, `?` without function context)
- [x] 3.2 Add `#![doc = include_str!("../README.md")]` to `core/src/lib.rs` (after `#![cfg_attr(docsrs, ...)]` line, before lint denials)
- [x] 3.3 Delete the existing hand-maintained crate-doc block at the top of `core/src/lib.rs`
- [x] 3.4 Run `cargo test --doc -p fugle-marketdata-core` — 60 passed; 0 failed; 13 ignored

## 4. Strict doc lints + backfill

- [x] 4.1 Add `#![deny(missing_docs, rustdoc::broken_intra_doc_links, clippy::missing_errors_doc)]` to `core/src/lib.rs`
- [x] 4.2 Capture warning baseline: 45 missing_docs + 46 missing_errors_doc + 5 broken intra-doc links
- [x] 4.3 Backfill `core/src/rest/`: REST builder setters and `send()` `# Errors` sections via scripted injection (43 sites)
- [x] 4.4 Backfill `core/src/websocket/`: error sections on `connect`, `disconnect`, `shutdown_with_timeout`, `subscribe`, `subscribe_futopt`, `unsubscribe`, `health_check::with_timeout`, `protocol::handle_subscribed_event`, `parser::parse_*`, `reconnection`, `message`, `aio::client` (18 sites)
- [x] 4.5 Backfill `core/src/errors.rs` (12 enum-variant field docs), `core/src/models/streaming.rs` (3 struct fields), `core/src/tls.rs` (1 site)
- [x] 4.6 `cargo doc --all-features -p fugle-marketdata-core` — zero rustdoc warnings
- [x] 4.7 `cargo clippy --all-features -p fugle-marketdata-core -- -D warnings` — clean (after fixing pre-existing `check-cfg`, `elided_named_lifetimes`, `type_complexity`, and `dead_code` warnings)

## 5. Validation

- [x] 5.1 Full test suite: `cargo test --all-features -p fugle-marketdata-core --lib` — 422 passed; 0 failed
- [ ] 5.2 `cargo +1.82 build --all-features -p fugle-marketdata-core` _(skipped — CI handles it; see 1.2)_
- [x] 5.3 `cargo test --doc -p fugle-marketdata-core` — 60 passed; 13 ignored
- [x] 5.4 FFI binding compile check: `cargo check -p marketdata-py -p marketdata-js -p marketdata-uniffi` — all clean
- [ ] 5.5 Preview rendered docs via `cargo doc --all-features -p fugle-marketdata-core --open`; visual inspection _(deferred — manual)_

## 6. Release prep

- [x] 6.1 Update `CHANGELOG.md` with `[Rust 0.4.1] - 2026-05-15` section
- [x] 6.2 Bump `core/Cargo.toml` `0.4.0` → `0.4.1`; also bumped `rust/Cargo.toml` and added `rust-version = "1.82"` to wrapper
- [x] 6.3 `cargo publish --dry-run -p fugle-marketdata-core --allow-dirty` — packaged 110 files, 773.3KiB; ready to publish
- [ ] 6.4 Open PR; wait for CI (including new MSRV job) to pass; merge _(user action)_
- [ ] 6.5 Tag `v0.4.1` and publish to crates.io _(user action — irreversible)_
