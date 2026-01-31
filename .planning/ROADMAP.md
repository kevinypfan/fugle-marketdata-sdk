# Milestone v0.3.0: API Compatibility & Configuration

**Status:** In Progress
**Target:** Align constructor APIs with official Fugle SDKs, expose WebSocket configuration options
**Phases:** 5

## Overview

Transform the SDK constructor APIs from positional string arguments to options object patterns matching the official Fugle Python and Node.js SDKs. Expose WebSocket reconnection and health check configuration through all 5 language bindings (Python, Node.js, C#, Java, Go).

## Phases

### Phase 1: Core Config Validation & Defaults

**Goal**: Establish canonical defaults, add comprehensive validation, align with official SDKs
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
- [ ] 01-01-PLAN.md — ReconnectionConfig validation & default constants
- [ ] 01-02-PLAN.md — HealthCheckConfig validation, default alignment & exports

### Phase 2: Python Binding Enhancement

**Goal**: Add options-based constructor and config exposure to Python binding
**Depends on**: Phase 1
**Plans:** (created by /gsd:plan-phase)

**Delivers:**
- `HealthCheckConfig` PyClass with constructor
- Modified `RestClient` to accept kwargs: `api_key`, `bearer_token`, `sdk_token`, `base_url`
- Modified `WebSocketClient` to accept optional `reconnect` and `health_check` configs
- Authentication validation (exactly one method required)
- Deprecation warnings for old string constructor (keep working)
- Cross-language default validation tests

**Requirements addressed:** API-01, API-02, API-03, API-04, API-05, WS-01 to WS-06, TEST-01, TEST-02

### Phase 3: Node.js Binding Enhancement

**Goal**: Add options-based constructor and config exposure to Node.js binding
**Depends on**: Phase 1
**Plans:** (created by /gsd:plan-phase)

**Delivers:**
- `WebSocketOptions` napi object with nested config types
- `ReconnectOptions` and `HealthCheckOptions` napi objects
- Modified `RestClient` to accept options object
- Modified `WebSocketClient` to accept options object
- Strict TypeScript definitions (no `any` types)
- Runtime validation despite TypeScript types
- Updated `index.d.ts` with comprehensive types

**Requirements addressed:** API-01, API-02, API-03, API-04, API-05, WS-01 to WS-06, CON-02, TEST-01, TEST-02

### Phase 4: UniFFI Binding Enhancement

**Goal**: Add config exposure to C#, Java, and Go bindings via UniFFI
**Depends on**: Phase 1
**Plans:** (created by /gsd:plan-phase)

**Delivers:**
- `ReconnectConfig` and `HealthCheckConfig` UniFFI Records
- `ClientOptions` Record for authentication options
- `new_with_config()` constructor alongside existing `new()`
- Updated C# wrapper with config support
- Updated Java wrapper with builder pattern for configs
- Updated Go wrapper with config parameters
- Tests for all three languages

**Requirements addressed:** API-01 to API-05, WS-01 to WS-06, CON-01, TEST-01, TEST-02

### Phase 5: Documentation & Migration

**Goal**: Update all documentation and provide migration tooling
**Depends on**: Phases 2, 3, 4
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
Phase 1 (Core) ─────┬─→ Phase 2 (Python) ────┐
                    ├─→ Phase 3 (Node.js) ───┼─→ Phase 5 (Docs)
                    └─→ Phase 4 (UniFFI) ────┘
```

Phases 2, 3, and 4 can run in parallel after Phase 1 completes.

## Key Decisions

| Decision | Rationale | Status |
|----------|-----------|--------|
| Options object constructor | Match official SDK patterns for painless migration | Confirmed |
| Health check default: `false` | Align with official Python/Node.js SDKs | Confirmed |
| Deprecation-first approach | Avoid breaking existing Python/Node.js users | Confirmed |
| Defer `subscribe(dict)` to v0.4.0 | Deeper API change, separate scope | Confirmed |
| Milliseconds at FFI boundary | Avoid Duration serialization complexity | Confirmed |

## Out of Scope

- WebSocket `subscribe()` signature change (dict vs positional) — v0.4.0
- REST client timeout configuration — v0.3.1
- Removal of deprecated string constructors — v0.4.0+

---
*Created: 2026-02-01*
*Research basis: .planning/research/v0.3.0-SUMMARY.md*
