# documentation-policy Specification

## Purpose
Codifies the publish-readiness layer for the `fugle-marketdata-core` crate: declared MSRV, strict doc-coverage lints, docs.rs feature-badge rendering, and README-as-crate-docs. Adopted from databento-rs conventions in 0.4.1 to prevent doc rot and improve discoverability for crates.io consumers before 0.5.0.

## Requirements

### Requirement: Declared MSRV

`core/Cargo.toml` SHALL declare `rust-version = "1.82"` under `[package]`. CI SHALL include a job that builds `fugle-marketdata-core` with `--all-features` using the Rust 1.82 toolchain and fails the pipeline on compile error.

#### Scenario: Manifest declares MSRV
- **WHEN** `core/Cargo.toml` is read
- **THEN** the `[package]` table MUST contain `rust-version = "1.82"`

#### Scenario: CI verifies MSRV builds
- **WHEN** the CI workflow runs
- **THEN** at least one job MUST install Rust 1.82 (no newer) and execute `cargo build --all-features -p fugle-marketdata-core` successfully

#### Scenario: Older toolchain produces clear error
- **WHEN** a consumer attempts `cargo build` with a Rust toolchain older than 1.82
- **THEN** Cargo MUST fail at dependency resolution with a message citing the MSRV declared in `core/Cargo.toml` rather than producing a downstream compile error

### Requirement: Strict documentation lints on core crate

`core/src/lib.rs` SHALL declare `#![deny(missing_docs, rustdoc::broken_intra_doc_links, clippy::missing_errors_doc)]`. Every public item exported from `fugle-marketdata-core` MUST carry a doc comment such that `cargo doc --all-features -p fugle-marketdata-core` produces zero warnings.

#### Scenario: Crate root denies missing docs
- **WHEN** `core/src/lib.rs` is read
- **THEN** the file MUST contain `#![deny(missing_docs, rustdoc::broken_intra_doc_links, clippy::missing_errors_doc)]` at crate scope

#### Scenario: cargo doc is clean
- **WHEN** `cargo doc --all-features -p fugle-marketdata-core` is run against the workspace
- **THEN** the command MUST exit with status 0 and emit no doc warnings

#### Scenario: New undocumented public item breaks build
- **WHEN** a contributor adds a `pub fn` to `core/` without a doc comment
- **THEN** `cargo build -p fugle-marketdata-core` MUST fail with a `missing_docs` lint error pointing at the new item

### Requirement: docs.rs renders all features with feature badges

`core/Cargo.toml` SHALL configure `[package.metadata.docs.rs]` with `all-features = true` and `rustdoc-args = ["--cfg", "docsrs"]`. `core/src/lib.rs` SHALL declare `#![cfg_attr(docsrs, feature(doc_cfg))]`. Every public item gated by `#[cfg(feature = "tokio-comp")]` or `#[cfg(feature = "tracing")]` SHALL also carry `#[cfg_attr(docsrs, doc(cfg(feature = "<name>")))]` so docs.rs renders a feature badge.

#### Scenario: docs.rs metadata enables all features
- **WHEN** `core/Cargo.toml` is read
- **THEN** `[package.metadata.docs.rs]` MUST contain both `all-features = true` and `rustdoc-args = ["--cfg", "docsrs"]`

#### Scenario: aio module carries feature badge
- **WHEN** the `core/src/websocket/aio/mod.rs` module declaration is rendered on docs.rs
- **THEN** docs.rs MUST display the badge `Available on crate feature tokio-comp only`

#### Scenario: tracing-gated items carry feature badge
- **WHEN** a public item annotated with `#[cfg(feature = "tracing")]` is rendered on docs.rs
- **THEN** docs.rs MUST display the badge `Available on crate feature tracing only`

### Requirement: README is the crate-level rustdoc

`core/src/lib.rs` SHALL include `#![doc = include_str!("../README.md")]` so the rendered crate-level documentation on docs.rs matches `core/README.md` exactly. README code blocks that cannot compile as doctests (placeholder snippets, partial examples) MUST be tagged with `ignore` or `no_run` so `cargo test --doc -p fugle-marketdata-core` passes.

#### Scenario: lib.rs embeds README as crate docs
- **WHEN** `core/src/lib.rs` is read
- **THEN** the file MUST contain `#![doc = include_str!("../README.md")]` (no separate hand-maintained crate-level doc block)

#### Scenario: cargo test --doc passes
- **WHEN** `cargo test --doc -p fugle-marketdata-core` is run
- **THEN** the command MUST exit with status 0 with all README-derived doctests either compiling, running, or correctly tagged `ignore` / `no_run`
