# Phase 12: Python Config Exposure - Context

**Gathered:** 2026-02-01
**Status:** Ready for planning

<domain>
## Phase Boundary

Add options-based constructor and config exposure to Python binding. Users can configure WebSocket reconnection and health check behavior through `ReconnectConfig` and `HealthCheckConfig` classes passed to constructors. Must match official `fugle-marketdata-python` SDK patterns for drop-in replacement compatibility.

</domain>

<decisions>
## Implementation Decisions

### Constructor API Design
- Separate kwargs for auth: `api_key`, `bearer_token`, `sdk_token`
- `base_url` included as optional kwarg
- Exactly one auth method required — raise `ValueError` if zero or multiple provided
- Fail-fast validation: check at construction, not at first request
- Error message: "Provide exactly one of: api_key, bearer_token, sdk_token"

### Config Class Interface
- Kwargs-based construction: `ReconnectConfig(max_attempts=5, initial_delay_ms=1000)`
- Time units in milliseconds with explicit suffix: `initial_delay_ms`, `timeout_ms`, `interval_ms`
- Validation at config construction (not deferred to WebSocketClient)
- Immutable after construction (frozen dataclass style)

### Deprecation Strategy
- No deprecation needed — SDK not formally released
- **Breaking change**: Remove old positional string constructor entirely
- New options-based constructor is the only constructor
- Align exactly with official `fugle-marketdata-python` where possible

### Error Handling
- Use `ValueError` for config validation errors (standard Python)
- Detailed error messages with valid range: "max_attempts must be >= 1, got 0"
- Fail on first invalid field (don't collect all errors)

### Claude's Discretion
- Exact param names: check official `fugle-marketdata-python` and match where sensible
- Type error handling: rely on PyO3 type conversion unless explicit validation improves DX
- Constructor signature details not covered above

</decisions>

<specifics>
## Specific Ideas

- "API-compatible drop-in replacement for official Fugle SDKs" is the core value
- Reference implementation: `/Users/zackfan/Project/fugle/fugle-marketdata-python`
- Milliseconds at FFI boundary (key decision from v0.3.0 roadmap)

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 12-python-config*
*Context gathered: 2026-02-01*
