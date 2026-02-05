# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-01)

**Core value:** API-compatible drop-in replacement for official Fugle SDKs
**Current focus:** v0.3.0 API Compatibility & Configuration

## Current Position

Phase: 12 of 15 — Python Config Exposure
Plan: 1/3 complete (12-01-PLAN.md complete ✓)
Status: Phase 8-11 complete ✓, Phase 12 in progress
Last activity: 2026-02-05 - Completed 12-01-PLAN.md (HealthCheckConfig + ReconnectConfig)

Progress: [████▓░░░░░] 53% (Phases 8-11 complete, Plan 12-01 complete)

**Note:** Phase 12-01 established Python config classes with core validation. Ready for 12-02 (RestClient options) and 12-03 (WebSocketClient options).

## Milestone History

- **v0.2.0** (2026-01-31): Multi-language SDK with Complete REST API Coverage
  - Phases 1-7 (including 4.1, 4.2), 50 plans, 203 commits
  - 5 language bindings (Python, Node.js, C#, Java, Go)
  - See: `.planning/milestones/v0.2.0-ROADMAP.md`

## Accumulated Context

### Decisions

Key decisions logged in v0.3.0 ROADMAP.md:
- Options object constructor pattern (matching official SDKs)
- Health check default: `false` (aligned with official SDKs)
- Deprecation-first approach for Python/Node.js
- Defer `subscribe(dict)` signature change to v0.4.0
- csbindgen over UniFFI for C# (better .NET idioms)

From Phase 8 execution:
- MIN_INITIAL_DELAY_MS = 100ms to prevent connection storms
- max_attempts must be >= 1 (zero attempts is invalid)
- max_delay must be >= initial_delay (logical constraint)
- Config validation returns Result<Self, MarketDataError::ConfigError>
- DEFAULT_HEALTH_CHECK_ENABLED = false (aligned with official SDKs per CON-01)
- MIN_HEALTH_CHECK_INTERVAL_MS = 5000ms (prevent excessive overhead)
- All 8 config constants re-exported from lib.rs for binding layers

From Phase 12-01 execution:
- ReconnectConfig field rename (max_retries→max_attempts, base_delay_ms→initial_delay_ms) is breaking change
- Config fields are immutable after construction (#[pyo3(get)] only, no set)
- Validation happens at construction time (fail-fast Python convention)
- to_core() method uses .expect() since validation already happened in constructor

### Patterns Established

From Phase 8-11 execution:
- Config validation pattern: new() returns Result with ConfigError on invalid input
- Default constants pattern: pub const DEFAULT_* for binding layer reference
- Error message pattern: include field name, constraint, and actual value
- Python async pattern: `future_into_py` + `spawn_blocking` for GIL safety
- Node.js async pattern: `Arc<ThreadsafeFunction>` + `NonBlocking` mode
- C# async pattern: `Task<T>` + `TaskCompletionSource` + `Task.Run`

From Phase 12-01 execution:
- Config PyClass pattern: kwargs-only constructor with #[pyo3(signature = (*, field=default))]
- Core validation delegation: Python layer calls core::Config::new(), maps errors to PyValueError
- FFI conversion: to_core() method converts validated Python config to core config
- Millisecond FFI boundary: store ms as u64 in Python, convert to Duration for core

### Roadmap Evolution

- v0.3.0 milestone: Phases 8-15 (continuing from v0.2.0's Phase 7)
- Phases 9-11 delivered foundational binding work, not config exposure
- Phases 12-14 added for actual v0.3.0 config exposure work
- Research complete: PYTHON-PATTERNS.md, NODEJS-PATTERNS.md, ARCHITECTURE.md, PITFALLS.md
- Summary: .planning/research/v0.3.0-SUMMARY.md

### Pending Todos

1. **WebSocket client shutdown blocking** (uniffi) - timeout workaround documented
2. **macOS code signing** - deferred until Apple Developer account configured

### Blockers/Concerns

None — Phases 8-11 complete. Phases 12, 13, 14 can run in parallel.

## Session Continuity

Last session: 2026-02-05
Stopped at: Completed 12-01-PLAN.md (HealthCheckConfig + ReconnectConfig PyClasses)
Resume file: N/A
Next: `/gsd:execute-phase 12 02` (RestClient options constructor) or continue with other v0.3.0 phases
