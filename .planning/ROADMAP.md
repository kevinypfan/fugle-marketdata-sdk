# Milestone v0.3.0: API Compatibility & Configuration

**Status:** In Progress
**Target:** Align constructor APIs with official Fugle SDKs, expose WebSocket configuration options
**Phases:** 8

## Overview

Transform the SDK constructor APIs from positional string arguments to options object patterns matching the official Fugle Python and Node.js SDKs. Expose WebSocket reconnection and health check configuration through all 5 language bindings (Python, Node.js, C#, Java, Go).

**Note:** Phases 2-4 delivered foundational binding upgrades (async patterns, type safety, new FFI). Phases 5-7 deliver the v0.3.0 configuration exposure work.

## Phases

### Phase 1: Core Config Validation & Defaults ✓

**Goal**: Establish canonical defaults, add comprehensive validation, align with official SDKs
**Status**: Complete (2026-02-01)
**Depends on**: Nothing (first phase)
**Plans:** 2 plans

**Delivers:**
- Comprehensive validation for `ReconnectionConfig` (max_attempts >= 1, delays > 0)
- Comprehensive validation for `HealthCheckConfig` (timeout < interval)
- Align `health_check.enabled` default to `false` (matching official SDKs)
- Export canonical defaults as public constants
- Helpful error messages with field names and valid ranges

**Requirements addressed:** VAL-01, VAL-02, VAL-03, VAL-04, CON-01

Plans:
- [x] 01-01-PLAN.md — ReconnectionConfig validation & default constants
- [x] 01-02-PLAN.md — HealthCheckConfig validation, default alignment & exports

### Phase 2: Python Async Foundation ✓

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

**NOT delivered (deferred to Phase 5):**
- Options-based constructor with kwargs
- `HealthCheckConfig` exposure
- WebSocket config parameter acceptance

### Phase 3: Node.js TypeScript Foundation ✓

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

**NOT delivered (deferred to Phase 6):**
- `WebSocketOptions`, `ReconnectOptions`, `HealthCheckOptions` types
- Options-based constructor
- Runtime validation for config

### Phase 4: C# csbindgen Foundation ✓

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

**NOT delivered (deferred to Phase 7):**
- Java binding (needs separate implementation)
- Go binding (needs separate implementation)
- Config exposure for any language

### Phase 5: Python Config Exposure

**Goal**: Add options-based constructor and config exposure to Python binding
**Depends on**: Phase 1 (core validation), Phase 2 (async foundation)
**Plans:** (created by /gsd:plan-phase)

**Delivers:**
- `HealthCheckConfig` PyClass with constructor
- Modified `RestClient` to accept kwargs: `api_key`, `bearer_token`, `sdk_token`, `base_url`
- Modified `WebSocketClient` to accept optional `reconnect` and `health_check` configs
- Wire `ReconnectConfig` to core's validated config
- Authentication validation (exactly one method required)
- Deprecation warnings for old string constructor

**Requirements addressed:** API-01 to API-05, WS-01 to WS-06, TEST-01

### Phase 6: Node.js Config Exposure

**Goal**: Add options-based constructor and config exposure to Node.js binding
**Depends on**: Phase 1 (core validation), Phase 3 (TypeScript foundation)
**Plans:** (created by /gsd:plan-phase)

**Delivers:**
- `WebSocketOptions` napi object with nested config types
- `ReconnectOptions` and `HealthCheckOptions` napi objects
- Modified `RestClient` to accept options object
- Modified `WebSocketClient` to accept options object
- Runtime validation despite TypeScript types
- Updated `index.d.ts` with config option types

**Requirements addressed:** API-01 to API-05, WS-01 to WS-06, CON-02, TEST-01

### Phase 7: Java & Go Bindings

**Goal**: Add Java and Go bindings with config support
**Depends on**: Phase 1 (core validation)
**Plans:** (created by /gsd:plan-phase)

**Delivers:**
- Java binding with builder pattern for configs
- Go binding with functional options pattern
- Config exposure for both languages
- C# config exposure (extend Phase 4 work)
- Tests for all three languages

**Requirements addressed:** API-01 to API-05, WS-01 to WS-06, CON-01, TEST-01, TEST-02

### Phase 8: Documentation & Migration

**Goal**: Update all documentation and provide migration tooling
**Depends on**: Phases 5, 6, 7
**Plans:** (created by /gsd:plan-phase)

**Delivers:**
- Updated README examples with options object pattern
- Configuration reference documentation
- Migration guide from v0.2.x to v0.3.0 (before/after examples)
- Migration scripts (Python codemod, JavaScript jscodeshift)
- CI check for outdated patterns in examples

**Requirements addressed:** DOC-01, DOC-02, DOC-03

---

## Phase Dependencies

```
Phase 1 (Core) ─────┬─→ Phase 5 (Py Config) ────┐
                    ├─→ Phase 6 (JS Config) ────┼─→ Phase 8 (Docs)
                    └─→ Phase 7 (Java/Go) ──────┘

Phase 2 (Py Async) ──→ Phase 5 (Py Config)
Phase 3 (JS Types) ──→ Phase 6 (JS Config)
Phase 4 (C# FFI) ────→ Phase 7 (Java/Go + C# Config)
```

Phases 5, 6, and 7 can run in parallel after their dependencies complete.

## Key Decisions

| Decision | Rationale | Status |
|----------|-----------|--------|
| Options object constructor | Match official SDK patterns for painless migration | Confirmed |
| Health check default: `false` | Align with official Python/Node.js SDKs | Confirmed |
| Deprecation-first approach | Avoid breaking existing Python/Node.js users | Confirmed |
| Defer `subscribe(dict)` to v0.4.0 | Deeper API change, separate scope | Confirmed |
| Milliseconds at FFI boundary | Avoid Duration serialization complexity | Confirmed |
| csbindgen over UniFFI for C# | Better .NET idioms, Task-based async | Confirmed (Phase 4) |

## Out of Scope

- WebSocket `subscribe()` signature change (dict vs positional) — v0.4.0
- REST client timeout configuration — v0.3.1
- Removal of deprecated string constructors — v0.4.0+

---
*Created: 2026-02-01*
*Updated: 2026-02-01 — Restructured to reflect actual Phase 2-4 deliverables*
*Research basis: .planning/research/v0.3.0-SUMMARY.md*
