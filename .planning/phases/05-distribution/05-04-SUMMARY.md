---
phase: 05-distribution
plan: 04
subsystem: infra
tags: [github-actions, pypi, npm, publishing, ci-cd, trusted-publishing, provenance]

# Dependency graph
requires:
  - phase: 05-01
    provides: Python wheel build workflow artifacts
  - phase: 05-02
    provides: Node.js binding build workflow artifacts
provides:
  - PyPI trusted publishing workflow (OIDC-based, no token)
  - npm publishing workflow with provenance attestation
  - Foundation for release automation pipeline
affects: [05-05, 05-06, release-pipeline]

# Tech tracking
tech-stack:
  added: ["pypa/gh-action-pypi-publish@v1.12.2"]
  patterns:
    - "Reusable workflows for package publishing"
    - "OIDC-based trusted publishing (PyPI)"
    - "Artifact download and package consolidation"

key-files:
  created:
    - ".github/workflows/publish-python.yml"
    - ".github/workflows/publish-nodejs.yml"
  modified: []

key-decisions:
  - "Use OIDC-based trusted publishing for PyPI (no token needed)"
  - "Use npm provenance attestation for supply chain security"
  - "Defer external service configuration (PyPI/npm setup) as checkpoint"

patterns-established:
  - "Reusable workflow pattern for Python package publishing"
  - "Reusable workflow pattern for Node.js package publishing"
  - "Artifact consolidation via download-artifact with merge-multiple"

# Metrics
duration: 1min
completed: 2026-01-31
---

# Phase 5: Distribution Plan 4 Summary

**PyPI trusted publishing with OIDC authentication and npm publishing with provenance attestation for automated package releases**

## Performance

- **Duration:** 1 min
- **Started:** 2026-01-31T18:05:15+08:00
- **Completed:** 2026-01-31T18:05:41+08:00
- **Tasks:** 2 of 3 (checkpoint skipped)
- **Files created:** 2

## Accomplishments

- PyPI publish workflow with OIDC-based trusted publishing (no token exposure)
- npm publish workflow with build provenance attestation for supply chain security
- Both workflows consume artifacts from Wave 1 build jobs (05-01, 05-02)
- Consolidated artifact handling with merge-multiple for multiple wheel/binding targets

## Task Commits

1. **Task 1: Create PyPI publish workflow with trusted publishing** - `cfd290c` (feat)
2. **Task 2: Create npm publish workflow** - `e2f4267` (feat)

_Note: Task 3 (Configure external services) was a checkpoint and was skipped by user decision._

## Files Created/Modified

- `.github/workflows/publish-python.yml` - Reusable workflow for PyPI publishing via OIDC trusted publishing
- `.github/workflows/publish-nodejs.yml` - Reusable workflow for npm publishing with provenance

## Decisions Made

1. **OIDC Trusted Publishing for PyPI**: Eliminates need to store PyPI tokens in GitHub secrets. Requires PyPI trusted publisher configuration matching workflow_name and environment name.

2. **npm Provenance Attestation**: Uses `npm publish --provenance` with `id-token: write` permission to add verifiable build attestation to npm packages (supply chain security).

3. **Artifact Consolidation**: Both workflows use `pattern` and `merge-multiple` to handle multiple build artifacts (e.g., multiple wheel files for different Python versions, multiple bindings for different platforms).

4. **Deferred External Configuration**: User chose to skip the checkpoint, deferring PyPI and npm external service setup. This is expected for early-stage workflow development (workflows created, external auth configured later when publishing).

## Deviations from Plan

None - plan executed exactly as specified for the two auto tasks. Task 3 checkpoint was skipped per user request (expected for this phase).

## External Configuration Pending

**These tasks were skipped at checkpoint and should be completed before releasing:**

1. **PyPI Trusted Publishing Configuration**
   - Go to https://pypi.org/manage/project/fugle-marketdata/settings/publishing/
   - Add new pending publisher with Repository: `fugle/fugle-marketdata-sdk`, Workflow: `publish-python.yml`, Environment: `release`
   - Verification: `pypa/gh-action-pypi-publish` will authenticate via OIDC token

2. **npm Token Configuration**
   - Generate automation token at https://www.npmjs.com/settings/YOUR_USERNAME/tokens
   - Add `NPM_TOKEN` secret to GitHub repository settings
   - Verification: `npm publish` will authenticate with NODE_AUTH_TOKEN

3. **GitHub Release Environment**
   - Create environment named `release` in repository settings
   - Optional: Add approval rules or other protection rules
   - Verification: PyPI workflow can deploy to this environment

## Issues Encountered

None - workflows created and committed successfully.

## User Setup Required

**External services require manual configuration before release:**

See detailed steps in "External Configuration Pending" section above. This is typical workflow development pattern:
- Phase 5 creates CI/CD workflows (automated)
- User configures external services (PyPI, npm, etc.) as needed before releases
- Phase 6 (Testing/Release) validates integration

## Next Phase Readiness

- Workflows are ready for testing and integration
- 05-05 (NuGet/Java publish workflows) continues same pattern
- Phase 6 can integrate these workflows into release pipeline once external configuration is complete
- No blockers for continuing to next distribution workflows

---
*Phase: 05-distribution*
*Plan: 04*
*Completed: 2026-01-31*
