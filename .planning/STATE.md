# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-01)

**Core value:** API-compatible drop-in replacement for official Fugle SDKs
**Current focus:** v0.3.0 API Compatibility & Configuration

## Current Position

Phase: 13 of 15 — Node.js Config Exposure
Plan: 3/3 complete (13-01, 13-02, 13-03 complete ✓)
Status: Phases 8-12 complete ✓, Phase 13 complete ✓
Last activity: 2026-02-06 - Completed 13-03-PLAN.md (TypeScript types and config tests)

Progress: [███████░░░] 70% (Phases 8-13 complete, 7/8 v0.3.0 phases complete)

**Note:** Phase 13 complete - Node.js bindings expose config with TypeScript union types for compile-time auth enforcement.

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

From Phase 12-02 execution:
- Kwargs-only constructors (signature with *) matching official SDK pattern
- Exactly-one-auth validation: ValueError if zero or multiple auth methods provided
- Config objects passed as &Bound<'_, ConfigType> and cloned for storage
- Core uses base_url() method (not with_base_url())
- WebSocketClient stores configs for future propagation to child clients

From Phase 12-03 execution:
- Comprehensive test coverage: 32 unit tests for all config classes and constructors
- Test pattern: construction, validation, field access, error paths
- Type stub accuracy: field names and signatures match implementation exactly
- Validation constraints documented in type stub docstrings for IDE hints

From Phase 13-01 execution:
- Use f64 (not u64) for millisecond fields - napi-rs doesn't support u64, f64 is JavaScript's number type
- Use #[napi(object)] for plain TypeScript interfaces (not classes with constructors)
- All config fields are Option<T> - TypeScript optional fields, runtime validation in constructors
- RestClientOptions defined in websocket.rs for re-export (shared by REST and WebSocket clients)

From Phase 13-02 execution:
- Breaking change: String constructor removed, only options object accepted per CONTEXT.md
- Error message format matches Python: "Provide exactly one of: apiKey, bearerToken, sdkToken"
- WebSocketClient configs validated but stored as _ (ConnectionConfig doesn't accept them yet)
- RestClient uses base_url() method (not with_base_url) per Phase 12-02 learning

From Phase 13-03 execution:
- TypeScript union types enforce exactly-one-auth at compile time using 'never' type
- Union pattern: `{ apiKey: string; bearerToken?: never; ... }` prevents multiple auth
- Config interfaces (ReconnectOptions, HealthCheckOptions) match Rust structs in camelCase
- Comprehensive test suite validates both TypeScript types and JavaScript runtime behavior
- Tests use @ts-expect-error to validate error messages for JavaScript users

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

From Phase 13-01 execution:
- napi struct pattern: #[napi(object)] + #[derive(Debug, Clone, Default)] + Option<T> fields
- snake_case Rust fields auto-convert to camelCase in TypeScript
- f64 for millisecond fields (JavaScript number type compatibility)
- Nested config types (reconnect, health_check) in WebSocketClientOptions

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

Last session: 2026-02-06
Stopped at: Completed 13-03-PLAN.md (TypeScript types and tests)
Resume file: N/A
Next: Phase 13-UAT (User Acceptance Testing for Node.js config exposure)
