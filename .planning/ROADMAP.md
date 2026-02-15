# Milestone v0.3.0: API Compatibility & Configuration

**Status:** In Progress
**Target:** Align constructor APIs with official Fugle SDKs, expose WebSocket configuration options
**Phases:** 8-15 (continuing from v0.2.0)

## Overview

Transform the SDK constructor APIs from positional string arguments to options object patterns matching the official Fugle Python and Node.js SDKs. Expose WebSocket reconnection and health check configuration through all 5 language bindings (Python, Node.js, C#, Java, Go).

**Note:** Phases 9-11 delivered foundational binding upgrades (async patterns, type safety, new FFI). Phases 12-14 deliver the v0.3.0 configuration exposure work.

## Phases

### Phase 8: Core Config Validation & Defaults ✓

**Goal**: Establish canonical defaults, add comprehensive validation, align with official SDKs
**Status**: Complete (2026-02-01)
**Depends on**: Nothing (first phase of v0.3.0)
**Plans:** 2 plans

**Delivers:**
- Comprehensive validation for `ReconnectionConfig` (max_attempts >= 1, delays > 0)
- Comprehensive validation for `HealthCheckConfig` (timeout < interval)
- Align `health_check.enabled` default to `false` (matching official SDKs)
- Export canonical defaults as public constants
- Helpful error messages with field names and valid ranges

**Requirements addressed:** VAL-01, VAL-02, VAL-03, VAL-04, CON-01

Plans:
- [x] 08-01-PLAN.md — ReconnectionConfig validation & default constants
- [x] 08-02-PLAN.md — HealthCheckConfig validation, default alignment & exports

### Phase 9: Python Async Foundation ✓

**Goal**: Modernize Python binding with PyO3 0.27+ and native asyncio support
**Status**: Complete (2026-01-31)
**Depends on**: Nothing (foundational)
**Plans:** 5 plans

**Actually Delivered:**
- PyO3 0.27 with `future_into_py` for async REST methods
- Async WebSocket with `connect_async`, `subscribe_async`, `disconnect_async`
- Async iterator (`__aiter__/__anext__`) for WebSocket messages
- GIL-safe polling with `spawn_blocking`
- Type stubs (739-line .pyi) with PEP 561 compliance
- `ReconnectConfig` class (partial - not connected to core)
- Static auth methods: `with_bearer_token()`, `with_sdk_token()`

**NOT delivered (deferred to Phase 12):**
- Options-based constructor with kwargs
- `HealthCheckConfig` exposure
- WebSocket config parameter acceptance

### Phase 10: Node.js TypeScript Foundation ✓

**Goal**: Upgrade Node.js binding with napi-rs 3.x and comprehensive TypeScript definitions
**Status**: Complete (2026-01-31)
**Depends on**: Nothing (foundational)
**Plans:** 4 plans

**Actually Delivered:**
- napi-rs 3.x with `Arc<ThreadsafeFunction>` pattern
- Promise-based async for all 11 REST methods
- EventEmitter pattern for WebSocket callbacks
- TypeScript definitions (813 lines, no `any` types)
- API-compatible with @fugle/marketdata structure

**NOT delivered (deferred to Phase 13):**
- `WebSocketOptions`, `ReconnectOptions`, `HealthCheckOptions` types
- Options-based constructor
- Runtime validation for config

### Phase 11: C# csbindgen Foundation ✓

**Goal**: Replace UniFFI with csbindgen for idiomatic .NET interop
**Status**: Complete (2026-01-31)
**Depends on**: Nothing (foundational)
**Plans:** 5 plans

**Actually Delivered:**
- csbindgen replacing UniFFI for C# binding
- `Task<T>` async pattern for REST methods
- `EventHandler<T>` pattern for WebSocket
- FFI panic handling with `catch_unwind`
- PascalCase naming matching .NET conventions

**NOT delivered (deferred to Phase 14):**
- Java binding (needs separate implementation)
- Go binding (needs separate implementation)
- Config exposure for any language

### Phase 12: Python Config Exposure ✓

**Goal**: Add options-based constructor and config exposure to Python binding
**Status**: Complete (2026-02-05)
**Depends on**: Phase 8 (core validation), Phase 9 (async foundation)
**Plans:** 3 plans

**Delivers:**
- `HealthCheckConfig` PyClass with constructor
- Modified `RestClient` to accept kwargs: `api_key`, `bearer_token`, `sdk_token`, `base_url`
- Modified `WebSocketClient` to accept optional `reconnect` and `health_check` configs
- Wire `ReconnectConfig` to core's validated config
- Authentication validation (exactly one method required)
- Unit tests for all constructor patterns (32 tests, 100% pass)

**Requirements addressed:** API-01 to API-05, WS-01 to WS-06, TEST-01

Plans:
- [x] 12-01-PLAN.md — Config classes (HealthCheckConfig, updated ReconnectConfig)
- [x] 12-02-PLAN.md — Client constructors (RestClient/WebSocketClient kwargs)
- [x] 12-03-PLAN.md — Tests and type stubs

### Phase 13: Node.js Config Exposure ✓

**Goal**: Add options-based constructor and config exposure to Node.js binding
**Status**: Complete (2026-02-06)
**Depends on**: Phase 8 (core validation), Phase 10 (TypeScript foundation)
**Plans:** 3 plans

**Delivers:**
- `WebSocketClientOptions` napi object with nested config types
- `ReconnectOptions` and `HealthCheckOptions` napi objects
- Modified `RestClient` to accept options object
- Modified `WebSocketClient` to accept options object
- Runtime validation despite TypeScript types (exactly-one-auth + config constraints)
- Updated `index.d.ts` with union types for compile-time auth enforcement
- Comprehensive test suite (188 lines)

**Requirements addressed:** API-01 to API-05, WS-01 to WS-06, CON-02, TEST-01

Plans:
- [x] 13-01-PLAN.md — Config option structs (ReconnectOptions, HealthCheckOptions, client options)
- [x] 13-02-PLAN.md — Client constructors (RestClient/WebSocketClient options acceptance)
- [x] 13-03-PLAN.md — TypeScript definitions and tests

### Phase 14: Java & Go Bindings ✓

**Goal**: Add config exposure to Java, Go, and C# bindings with idiomatic patterns
**Status**: Complete (2026-02-15)
**Depends on**: Phase 8 (core validation), Phase 11 (C# foundation)
**Plans:** 3 plans

**Delivers:**
- Java binding with builder pattern for configs
- Go binding with functional options pattern
- C# config exposure (extend Phase 11 work)
- Tests for all three languages (41 total: 14 Java + 13 Go + 14 C#)

**Requirements addressed:** API-01 to API-05, WS-01 to WS-06, CON-01, TEST-01, TEST-02

Plans:
- [x] 14-01-PLAN.md — Java builder pattern with config classes and tests
- [x] 14-02-PLAN.md — Go functional options with config structs and tests
- [x] 14-03-PLAN.md — C# options classes with config and tests

### Phase 15: Documentation & Migration

**Goal**: Update all documentation and provide migration tooling
**Depends on**: Phases 12, 13, 14
**Plans:** 3 plans

**Delivers:**
- Updated README examples with options object pattern
- Configuration reference documentation
- Migration guide from v0.2.x to v0.3.0 (before/after examples)
- Migration scripts (Python codemod, JavaScript jscodeshift)
- CI check for outdated patterns in examples

**Requirements addressed:** DOC-01, DOC-02, DOC-03

Plans:
- [ ] 15-01-PLAN.md — Update all language READMEs and create CHANGELOG.md
- [ ] 15-02-PLAN.md — Migration guide and configuration reference docs
- [ ] 15-03-PLAN.md — Migration scripts (Python codemod, JS codemod) and CI validation

---

## Phase Dependencies

```
Phase 8 (Core) ─────┬─→ Phase 12 (Py Config) ────┐
                    ├─→ Phase 13 (JS Config) ────┼─→ Phase 15 (Docs)
                    └─→ Phase 14 (Java/Go) ──────┘

Phase 9 (Py Async) ──→ Phase 12 (Py Config)
Phase 10 (JS Types) ─→ Phase 13 (JS Config)
Phase 11 (C# FFI) ───→ Phase 14 (Java/Go + C# Config)
```

Phases 12, 13, and 14 can run in parallel after their dependencies complete.

## Key Decisions

| Decision | Rationale | Status |
|----------|-----------|--------|
| Options object constructor | Match official SDK patterns for painless migration | Confirmed |
| Health check default: `false` | Align with official Python/Node.js SDKs | Confirmed |
| Deprecation-first approach | Avoid breaking existing Python/Node.js users | Confirmed |
| Defer `subscribe(dict)` to v0.4.0 | Deeper API change, separate scope | Confirmed |
| Milliseconds at FFI boundary | Avoid Duration serialization complexity | Confirmed |
| csbindgen over UniFFI for C# | Better .NET idioms, Task-based async | Confirmed (Phase 11) |

## Out of Scope

- WebSocket `subscribe()` signature change (dict vs positional) -- v0.4.0
- REST client timeout configuration -- v0.3.1
- Removal of deprecated string constructors -- v0.4.0+

---
*Created: 2026-02-01*
*Updated: 2026-02-16 -- Phase 15 planned (3 plans, 2 waves)*
*Research basis: .planning/research/v0.3.0-SUMMARY.md*
