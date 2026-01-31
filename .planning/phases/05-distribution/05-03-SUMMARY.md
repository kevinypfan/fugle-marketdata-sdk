---
phase: 05-distribution
plan: 03
subsystem: infra
tags: [github-actions, uniffi, cross-compile, native-libraries, ci-cd]

requires:
  - phase: 04.1-uniffi-migration
    provides: marketdata-uniffi crate with cdylib target

provides:
  - Reusable workflow for UniFFI native library builds
  - Cross-platform native libraries (Linux, macOS, Windows)
  - RID-organized consolidated artifact for downstream packaging

affects: [05-04-csharp-nuget, 05-05-go-module, 05-06-java-maven]

tech-stack:
  added: []
  patterns:
    - Reusable workflow with workflow_call
    - Cross-compilation with target-specific toolchains
    - RID-organized artifact structure for multi-platform packages

key-files:
  created:
    - .github/workflows/build-uniffi.yml

key-decisions:
  - "Separate macOS ARM64 and x64 builds (no universal2 for cdylib)"
  - "RID naming convention (linux-x64, osx-arm64, osx-x64, win-x64) matches NuGet runtimes"
  - "Consolidation job downloads all artifacts into single uniffi-all structure"

patterns-established:
  - "UniFFI build workflow: workflow_call for reuse, workflow_dispatch for manual testing"
  - "Artifact naming: uniffi-{target} for individual, uniffi-all for consolidated"

duration: 2min
completed: 2026-01-31
---

# Phase 05 Plan 03: UniFFI Native Library Build Workflow Summary

**Reusable GitHub Actions workflow building UniFFI cdylib for Linux x64, macOS ARM64/x64, and Windows x64 with RID-organized consolidated artifact**

## Performance

- **Duration:** 2 min
- **Started:** 2026-01-31T09:59:22Z
- **Completed:** 2026-01-31T10:01:11Z
- **Tasks:** 2
- **Files created:** 1

## Accomplishments

- Created reusable workflow (workflow_call) for UniFFI native library builds
- Matrix builds for 4 platform targets with correct library naming (.so/.dylib/.dll)
- Consolidation job creates uniffi-all artifact with RID-organized directory structure
- Version output from Cargo.toml workspace for downstream publish workflows

## Task Commits

Each task was committed atomically:

1. **Task 1: Create UniFFI native library build workflow** - `c804038` (feat)
   - Includes Task 2 requirements (workflow_dispatch and consolidation job were implemented together)

## Files Created/Modified

- `.github/workflows/build-uniffi.yml` - Reusable workflow for cross-platform UniFFI native library builds

## Decisions Made

1. **Separate macOS builds instead of universal2**: UniFFI cdylib requires separate ARM64 and x64 builds since we're not creating fat binaries. C#/Go/Java consumers select the correct library at runtime based on platform detection.

2. **RID naming convention**: Used NuGet runtime identifier (RID) naming pattern (linux-x64, osx-arm64, osx-x64, win-x64) for consistency with .NET packaging and easy mapping to NuGet's `runtimes/{rid}/native/` structure.

3. **Consolidation approach**: Single consolidation job downloads all 4 artifacts and organizes them into unified structure, simplifying downstream workflows that only need to download one artifact.

## Deviations from Plan

None - plan executed exactly as written. Task 1 and Task 2 were implemented together since the consolidation job and workflow_dispatch trigger were naturally part of the complete workflow design.

## Issues Encountered

None

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- build-uniffi.yml ready to be called by publish workflows
- uniffi-all artifact structure matches expected RID layout for NuGet packaging
- Version output available for downstream version consistency

---
*Phase: 05-distribution*
*Completed: 2026-01-31*
