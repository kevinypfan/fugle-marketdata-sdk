# Phase 5: Cross-Platform Distribution - Context

**Gathered:** 2026-01-31
**Status:** Ready for planning

<domain>
## Phase Boundary

Automate package publishing with platform-specific builds for PyPI, npm, NuGet, and Maven registries. Users can install the SDK in their language of choice without requiring a Rust toolchain. Release workflow publishes all packages with synchronized version numbers.

</domain>

<decisions>
## Implementation Decisions

### Claude's Discretion

User delegated all distribution decisions to Claude. Research and planning will determine:

**Platform Matrix:**
- Which OS/architecture combinations to support
- Linux: x86_64, aarch64, musl variants
- macOS: Intel, ARM, universal2 builds
- Windows: x86_64 (and potentially ARM64)

**Release Workflow:**
- Manual vs automated release triggers
- Version tagging and synchronization strategy
- Coordinated multi-registry publishing approach
- CI/CD workflow structure (GitHub Actions)

**Registry Authentication:**
- PyPI, npm, NuGet, Maven Central credentials management
- GitHub Secrets organization
- Publishing token scope and rotation

**Build Artifacts:**
- Pre-built wheels (Python), native addons (Node.js), NuGet packages (C#), JAR files (Java)
- Platform-specific vs fat binary approach
- Artifact naming conventions

**Language-Specific Tooling:**
- Python: maturin for wheel building
- Node.js: napi-rs prebuild with @napi-rs/cli
- C#: dotnet pack with native library bundling
- Go: Pre-compiled shared libraries
- Java: Gradle publish with native library resources

</decisions>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches based on research findings.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 05-distribution*
*Context gathered: 2026-01-31*
