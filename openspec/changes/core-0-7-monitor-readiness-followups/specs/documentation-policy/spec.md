## ADDED Requirements

### Requirement: tracing_compat module documents internal macro export

`core/src/tracing_compat.rs` SHALL include a module-level `//!` doc-comment explicitly stating that the `__tracing_noop` macro is exported at crate root through `#[macro_export]` solely because `macro_rules!` has no other mechanism for cross-module visibility, that the macro is `#[doc(hidden)]`, and that downstream consumers MUST NOT depend on it. The note MUST also warn that the macro shape may change without semver bump on `tracing` crate major-version upgrades.

#### Scenario: Module doc explains the macro export
- **WHEN** `core/src/tracing_compat.rs` is read
- **THEN** the file MUST contain a `//!` block that mentions both `__tracing_noop` AND `#[doc(hidden)]` AND a phrase indicating the macro is not part of the public API

#### Scenario: __tracing_noop carries doc(hidden)
- **WHEN** the `__tracing_noop` macro definition is read
- **THEN** it MUST be preceded by `#[doc(hidden)]`

### Requirement: cargo public-api regression test

The repository SHALL include a CI job (`ci-public-api`) that runs `cargo public-api --diff <baseline-commit>..HEAD --simplified -p fugle-marketdata-core` and fails if the diff is non-empty without an accompanying `PUBLIC-API.md` acknowledgement entry. The job SHALL run on PRs that touch any of:

- `core/src/lib.rs`
- `core/src/tracing_compat.rs`
- `core/Cargo.toml` (when `[features]` or top-level dependencies change)

The baseline commit SHALL be the most recent merge to `main` tagged with a release tag (`vX.Y.Z`). A repository file `PUBLIC-API.md` SHALL list each acknowledged public-API change with the PR number, the new symbol, and the rationale.

#### Scenario: Workflow defines ci-public-api job
- **WHEN** `.github/workflows/` is searched
- **THEN** at least one workflow file MUST define a job named `ci-public-api` running `cargo public-api`

#### Scenario: Job filters on relevant paths
- **WHEN** the `ci-public-api` job's `paths:` filter is read
- **THEN** the filter MUST include `core/src/lib.rs`, `core/src/tracing_compat.rs`, and `core/Cargo.toml`

#### Scenario: Unacknowledged public surface change fails CI
- **WHEN** a PR adds a new `pub fn` to `core/src/lib.rs` without a corresponding entry in `PUBLIC-API.md`
- **THEN** the `ci-public-api` job MUST fail with a message naming the new symbol

#### Scenario: Acknowledged change passes CI
- **WHEN** a PR adds a new `pub fn` to `core/src/lib.rs` AND adds a `PUBLIC-API.md` entry referencing the same symbol name
- **THEN** the `ci-public-api` job MUST pass
