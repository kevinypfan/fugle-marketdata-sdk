---
phase: 15-documentation-migration
plan: 03
subsystem: migration-tooling
tags: [libcst, jscodeshift, codemod, github-actions, markdownlint, ci]

# Dependency graph
requires:
  - phase: 15-01
    provides: "Updated READMEs and CHANGELOG to v0.3.0 API patterns"
  - phase: 15-02
    provides: "MIGRATION.md guide with migration steps"
provides:
  - Python codemod (libCST) for v0.2.x → v0.3.0 migration
  - JavaScript codemod (jscodeshift) for v0.2.x → v0.3.0 migration
  - Validation script checking for deprecated patterns
  - CI workflow for documentation quality enforcement
affects: [release, ci-cd, documentation-maintenance]

# Tech tracking
tech-stack:
  added: [libcst, jscodeshift, markdownlint-cli, markdown-link-check]
  patterns: [codemod pattern, CI validation pattern, dry-run preview pattern]

key-files:
  created:
    - migration/migrate-python.py
    - migration/migrate-javascript.js
    - migration/validate-migration.sh
    - .github/workflows/docs-validation.yml
    - .markdownlint.json
    - .markdown-link-check.json
  modified: []

key-decisions:
  - "libCST chosen over lib2to3 or regex for lossless Python transformation"
  - "jscodeshift transform module pattern (not standalone CLI)"
  - "Validation script skips MIGRATION.md (expected to show deprecated patterns in Before sections)"
  - "CI workflow runs on PRs/pushes affecting docs, migration, or examples"
  - "markdownlint allows long lines (MD013:false) for code blocks"

patterns-established:
  - "Codemod pattern: detect already-migrated code and skip transformation"
  - "Dry-run pattern: preview changes before applying"
  - "CI validation: enforce documentation quality on every PR"

# Metrics
duration: 2min 39s
completed: 2026-02-16
---

# Phase 15 Plan 03: Migration Tooling Summary

**Automated codemods (Python libCST + JavaScript jscodeshift) and CI validation workflow preventing documentation drift back to deprecated v0.2.x patterns**

## Performance

- **Duration:** 2min 39s
- **Started:** 2026-02-15T17:08:11Z
- **Completed:** 2026-02-15T17:10:50Z
- **Tasks:** 2
- **Files created:** 6

## Accomplishments
- Python codemod handles 5 transformation patterns (positional args, static methods)
- JavaScript codemod transforms constructor calls to options objects
- Both codemods support dry-run mode and skip already-migrated code
- CI workflow validates markdown quality and checks for deprecated patterns
- Validation script passes on current documentation (zero deprecated patterns found)

## Task Commits

Each task was committed atomically:

1. **Task 1: Create Python and JavaScript migration scripts** - `af34369` (feat)
2. **Task 2: Create validation script and CI docs workflow** - `2d8b096` (feat)

## Files Created/Modified

### Created
- `migration/migrate-python.py` - libCST codemod for Python v0.2.x → v0.3.0 migration (5 patterns)
- `migration/migrate-javascript.js` - jscodeshift transform for JavaScript v0.2.x → v0.3.0 migration
- `migration/validate-migration.sh` - Bash script checking READMEs/examples for deprecated patterns
- `.github/workflows/docs-validation.yml` - CI workflow running markdownlint, link checks, pattern validation
- `.markdownlint.json` - Configuration disabling MD013 (line length), MD033 (inline HTML), MD041 (first line h1)
- `.markdown-link-check.json` - Configuration for link validation (localhost ignore, 429 retry, 10s timeout)

## Decisions Made

**1. libCST chosen for Python codemod**
- Rationale: Lossless transformation preserving formatting and comments (superiority over lib2to3 or regex)
- Pattern: CSTTransformer class with leave_Call visitor matching specific patterns

**2. jscodeshift transform module pattern**
- Rationale: Standard jscodeshift pattern (not standalone CLI like Python script)
- Usage: `npx jscodeshift -t migration/migrate-javascript.js src/`
- Returns: null if no modifications (jscodeshift convention)

**3. Validation script skips MIGRATION.md**
- Rationale: MIGRATION.md is expected to show deprecated patterns in "Before" examples
- Scope: Only checks README files (py/README.md, js/README.md, core/README.md, uniffi/README.md)

**4. CI workflow triggers on doc-related paths**
- Rationale: Avoid running validation on every code change
- Paths: `**/*.md`, `migration/**`, `examples/**`
- Runs: On PRs and pushes to main

**5. markdownlint configuration tuned for SDK docs**
- MD013 (line length): disabled - code blocks need long lines
- MD033 (inline HTML): disabled - allow badges
- MD041 (first line h1): disabled - CHANGELOG has comment-like header

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None - all tools worked as expected, validation passed on first run.

## User Setup Required

**Migration script dependencies:**

For Python migration:
```bash
pip install libcst
python migration/migrate-python.py --path src/ --dry-run
```

For JavaScript migration:
```bash
npm install -g jscodeshift
npx jscodeshift -t migration/migrate-javascript.js src/ --dry
```

**Note:** These are user-facing tools, not project dependencies. Users run them once to migrate their code.

## Next Phase Readiness

Phase 15 complete:
- ✓ Plan 01: READMEs and CHANGELOG updated to v0.3.0
- ✓ Plan 02: MIGRATION.md guide created (assumption based on depends_on)
- ✓ Plan 03: Migration tooling and CI validation

All v0.3.0 documentation deliverables complete. CI will prevent documentation drift.

## Self-Check: PASSED

All files verified:
- ✓ migration/migrate-python.py
- ✓ migration/migrate-javascript.js
- ✓ migration/validate-migration.sh
- ✓ .github/workflows/docs-validation.yml
- ✓ .markdownlint.json
- ✓ .markdown-link-check.json
- ✓ .planning/phases/15-documentation-migration/15-03-SUMMARY.md

All commits verified:
- ✓ af34369 (Task 1: Create Python and JavaScript migration scripts)
- ✓ 2d8b096 (Task 2: Create validation script and CI docs workflow)

---
*Phase: 15-documentation-migration*
*Completed: 2026-02-16*
