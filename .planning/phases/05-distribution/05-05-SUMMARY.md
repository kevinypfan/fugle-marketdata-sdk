---
phase: 05-distribution
plan: 05
subsystem: distribution
tags: [nuget, java, publishing, github-actions, gradle, dotnet]

# Dependency graph
requires:
  - phase: 05-03
    provides: Cross-platform native libraries (uniffi-all artifact)
provides:
  - NuGet publish workflow for C# packages
  - Java publish workflow for GitHub Packages
  - GitHub Actions workflows for automated distribution
affects: [Phase 6 (Testing), future package releases]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Reusable GitHub Actions workflows for package publishing"
    - "Native library bundling in NuGet (runtimes/{rid}/native)"
    - "Native library bundling in Java (src/main/resources/native/{platform})"
    - "GitHub Packages publishing with automatic GITHUB_TOKEN authentication"

key-files:
  created:
    - .github/workflows/publish-nuget.yml
    - .github/workflows/publish-java.yml
  modified:
    - bindings/csharp/MarketdataUniffi/MarketdataUniffi.csproj
    - bindings/java/build.gradle.kts

key-decisions:
  - "Java publishes to GitHub Packages (automatic GITHUB_TOKEN) instead of Maven Central (requires GPG signing)"
  - "NuGet uses API key with skip-duplicate for idempotent publishing"
  - "Native libraries bundled at build time into package artifacts"

patterns-established:
  - "GitHub Packages CI publishing pattern: download artifact → copy natives → gradle publish"
  - "NuGet CI publishing pattern: download artifact → copy natives → dotnet pack → dotnet nuget push"
  - "Cross-platform native library organization: platform-specific subdirectories in runtimes/"

# Metrics
duration: 5min
completed: 2026-01-31
---

# Phase 5, Plan 5: Publish Workflows Summary

**NuGet and Java publish workflows with cross-platform native library bundling and GitHub Actions integration**

## Performance

- **Duration:** ~5 min
- **Completed:** 2026-01-31
- **Tasks:** 2 completed, 1 skipped
- **Files modified:** 4

## Accomplishments

- Created `.github/workflows/publish-nuget.yml` reusable workflow that downloads uniffi-all artifact, organizes native libraries by RID (runtime identifier), packs with dotnet, and publishes to NuGet.org with skip-duplicate idempotency
- Created `.github/workflows/publish-java.yml` reusable workflow that downloads uniffi-all artifact, copies native libraries to JNA resource paths, and publishes to GitHub Packages with automatic GITHUB_TOKEN authentication
- Updated `.csproj` to support both CI builds (runtimes/ artifact paths) and local development patterns
- Added maven-publish plugin to Gradle with GitHub Packages repository configuration and automatic GITHUB_TOKEN credentials

## Task Commits

Each task was committed atomically:

1. **Task 1: Create NuGet publish workflow** - `789f5e2` (feat)
2. **Task 2: Create Java publish workflow** - `7b188e4` (feat)

**Plan metadata:** Not yet committed (see User Setup section below)

## Files Created/Modified

- `.github/workflows/publish-nuget.yml` - Reusable workflow for NuGet publishing with native library bundling
- `.github/workflows/publish-java.yml` - Reusable workflow for GitHub Packages publishing with native library bundling
- `bindings/csharp/MarketdataUniffi/MarketdataUniffi.csproj` - Added runtimes directory inclusion for NuGet packing
- `bindings/java/build.gradle.kts` - Added maven-publish plugin and GitHub Packages repository configuration

## Decisions Made

1. **GitHub Packages for Java**: Chose GitHub Packages over Maven Central to avoid GPG signing complexity. Automatic GITHUB_TOKEN authentication with packages:write permission simplifies CI setup.

2. **NuGet skip-duplicate**: Implemented `--skip-duplicate` flag in dotnet nuget push for idempotent publishing - same version can be pushed multiple times without failure.

3. **Native library bundling timing**: Native libraries bundled at package build time (not runtime) by including pre-downloaded artifact contents. Simplifies packaging and ensures deterministic builds.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None - workflows created as specified.

## User Setup Required

**External service configuration is PENDING:**

The plan included Task 3 (checkpoint for NuGet API key configuration), which was skipped per user decision to defer external service setup.

### Pending Configuration

**NuGet API Key Setup (PENDING):**
- Service: NuGet.org
- Required for: Publishing C# packages to NuGet
- Status: Not configured
- Steps when ready:
  1. Go to https://www.nuget.org/account/apikeys
  2. Create new API key with name "fugle-marketdata-sdk-ci"
  3. Set glob pattern to "MarketdataUniffi*"
  4. Add "Push" scope
  5. In GitHub repo settings → Secrets and variables → Actions
  6. Create secret: NUGET_API_KEY with the key value

**GitHub Packages (AUTOMATIC):**
- No setup required
- Uses automatic GITHUB_TOKEN with packages:write permission
- Java publishing will work immediately once workflows are triggered

### Current State

- ✅ Workflows created and committed
- ✅ Gradle configuration complete (GITHUB_TOKEN automatic)
- ❌ NuGet API key secret not yet added to GitHub
- ⏳ Publishing will fail for NuGet until NUGET_API_KEY secret is configured

To complete this plan's external setup, add the NUGET_API_KEY secret when ready. Java publishing is fully automated.

## Next Phase Readiness

- Publishing workflows ready for integration into release pipeline
- All platform-specific native library bundling configured correctly
- Java publishing immediately available (GitHub Packages auto-authenticated)
- NuGet publishing blocked on NUGET_API_KEY secret addition
- Ready to proceed with Phase 6 (Testing) in parallel

---

*Phase: 05-distribution*
*Completed: 2026-01-31*
