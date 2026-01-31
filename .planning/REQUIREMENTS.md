# Requirements: v0.3.0 API Compatibility & Configuration

**Milestone:** v0.3.0
**Status:** Defining
**Last Updated:** 2026-02-01

## Overview

Align SDK constructor APIs with official Fugle marketdata SDKs (Python and Node.js) and expose WebSocket configuration options (reconnection, health check) through all language bindings.

## Functional Requirements

### Constructor API Alignment

- [x] **API-01**: RestClient accepts options object constructor `RestClient({ apiKey: '...' })` matching official SDK patterns
- [x] **API-02**: WebSocketClient accepts options object constructor matching official SDK patterns
- [x] **API-03**: Support all three authentication methods via options: `api_key`, `bearer_token`, `sdk_token`
- [x] **API-04**: Validate exactly one authentication method provided (error if zero or multiple)
- [x] **API-05**: Support `base_url` override for REST and WebSocket clients (optional)

### WebSocket Configuration

- [x] **WS-01**: Expose reconnection config: `max_attempts` (default: 5)
- [x] **WS-02**: Expose reconnection config: `initial_delay_ms` (default: 1000)
- [x] **WS-03**: Expose reconnection config: `max_delay_ms` (default: 60000)
- [x] **WS-04**: Expose health check config: `enabled` (default: false, aligned with official SDKs)
- [x] **WS-05**: Expose health check config: `interval_ms` (default: 30000)
- [x] **WS-06**: Expose health check config: `max_missed_pongs` (default: 2)

### Configuration Validation

- [x] **VAL-01**: Configuration validation at construction time (fail-fast)
- [x] **VAL-02**: Validate reconnection config constraints (max_attempts >= 1, delays > 0)
- [x] **VAL-03**: Validate health check constraints (timeout < interval)
- [x] **VAL-04**: Error messages include field names and valid ranges

## Non-Functional Requirements

### Consistency

- [x] **CON-01**: Configuration defaults consistent across all 5 language bindings (Python, Node.js, C#, Java, Go)
- [x] **CON-02**: TypeScript definitions with strict types (no `any`)
- [x] **CON-03**: Duration values as milliseconds at FFI boundaries

### Documentation

- [x] **DOC-01**: Configuration reference documentation for all options
- [x] **DOC-02**: Migration guide from v0.2.x to v0.3.0
- [x] **DOC-03**: Updated README examples with options object pattern

## Testing Requirements

- [x] **TEST-01**: Unit tests for all constructor patterns (default, custom, invalid configs)
- [x] **TEST-02**: Cross-language configuration consistency tests
- [x] **TEST-03**: Integration tests for reconnection behavior with custom config
- [x] **TEST-04**: Integration tests for health check behavior with custom config

## Out of Scope (Deferred)

- **WebSocket `subscribe()` signature change** — Research shows official SDK uses `subscribe(dict)` while we use `subscribe(channel, symbol)`. This is a deeper API change deferred to v0.4.0.
- **REST client timeout config** — Core hardcodes 30s timeouts. Optional feature, consider for v0.3.1.
- **Breaking removal of string constructors** — Deprecation only in v0.3.0, removal in v0.4.0+.

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| API-01 to API-05 | Phase 2-4 | Phase 2 (Python): Complete |
| WS-01 to WS-06 | Phase 1-4 | Phase 1 (Core): Complete, Phase 2 (Python): Complete |
| VAL-01 to VAL-04 | Phase 1 | Complete |
| CON-01 to CON-03 | Phase 1-4 | Phase 1 (Core): Complete, Phase 2 (Python): Complete |
| DOC-01 to DOC-03 | Phase 5 | Pending |
| TEST-01 to TEST-04 | Phase 1-4 | Phase 1 (Core): Complete, Phase 2 (Python): Complete |

---
*Created: 2026-02-01 from v0.3.0 research synthesis*
