# Phase 1: Build Infrastructure Modernization - Context

**Gathered:** 2026-01-30
**Status:** Ready for planning

<domain>
## Phase Boundary

Establish unified Cargo workspace with shared dependencies and automated CI/CD pipelines. Developers can build all language bindings (Python, Node.js, C#) from a single workspace root. Language binding implementations, API compatibility, and package distribution are separate phases.

</domain>

<decisions>
## Implementation Decisions

### Workspace Structure
- Claude decides member layout (flat vs grouped)
- Maximum dependency sharing via workspace inheritance (all common deps in root Cargo.toml)
- Minimal feature flags — core has everything enabled, bindings just import it

### Build Commands
- Makefile as primary build interface (universal compatibility, CI-friendly)
- Sequential execution for `make all` (builds Python → Node.js → C# in order)
- Explicit targets for dev/release: `make python-dev`, `make python-release`

### CI/CD Behavior
- Smart detection triggers: path-based (py/ changes → Python CI only, core/ → all bindings)
- Full platform matrix: Linux + macOS + Windows for each binding
- Claude's discretion on caching strategy

### Version Synchronization
- Single source of truth: `workspace.package.version` in root Cargo.toml
- Manual version bumps only (developer edits Cargo.toml)
- All bindings always match core version (Python 0.2.0 = Node.js 0.2.0 = C# 0.2.0)

### Claude's Discretion
- Workspace member layout (flat vs grouped)
- CI caching strategy (aggressive vs minimal)
- Specific Makefile target naming conventions
- GitHub Actions workflow file organization

</decisions>

<specifics>
## Specific Ideas

- Build interface should feel familiar to Rust developers (`make`, not exotic tooling)
- Version matching across bindings supports "drop-in replacement" positioning — users can expect consistent behavior across languages at same version

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 01-build-infrastructure*
*Context gathered: 2026-01-30*
