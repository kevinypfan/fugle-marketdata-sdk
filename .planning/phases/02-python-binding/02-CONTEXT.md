# Phase 2: Python Binding Enhancement - Context

**Gathered:** 2026-02-01
**Status:** Ready for planning

<domain>
## Phase Boundary

Modify Python binding constructors to accept kwargs (matching official Fugle SDK), expose WebSocket configuration options as flat kwargs, and add authentication validation. This phase does NOT add new API endpoints or change WebSocket subscribe() signatures.

</domain>

<decisions>
## Implementation Decisions

### Constructor Style
- **Kwargs only** — `RestClient(api_key='...')` or `RestClient(bearer_token='...')`
- **No positional arguments** — matches official Fugle Python SDK pattern
- **Remove static methods** — `with_bearer_token()` and `with_sdk_token()` are removed entirely (not deprecated)
- **Pure kwargs, no Options object** — no `ClientOptions` dataclass; just individual kwargs

### Error Messages
- **Detailed with examples** — Include valid usage example in error message
  - e.g., `"Must provide exactly one of: api_key, bearer_token, or sdk_token. Example: RestClient(api_key='...')"`
- **ValueError for auth errors** — Standard Python exception for invalid arguments
- **English only** — No i18n; consistent with most Python libraries
- **Fail at construction time** — Invalid config raises immediately, not at connect()

### Config Exposure Style
- **Flat kwargs** — `WebSocketClient(api_key='...', reconnect_max_attempts=5, health_check_enabled=True)`
- **Prefixed snake_case** — `reconnect_max_attempts`, `reconnect_initial_delay_ms`, `health_check_enabled`, `health_check_interval_ms`
- **Milliseconds for time values** — `_ms` suffix, consistent with Rust core
- **Read-only properties exposed** — Users can inspect config after construction via `ws_client.reconnect_max_attempts` etc.

### Claude's Discretion
- Exact property names (as long as prefixed snake_case with `_ms` for time)
- Type hints approach (runtime checking vs static only)
- Whether to provide a `__repr__` showing config values
- Test organization and fixtures

</decisions>

<specifics>
## Specific Ideas

- Official Fugle Python SDK already uses `RestClient(api_key='YOUR_API_KEY')` kwargs pattern — our change aligns with this
- Static methods (`with_bearer_token`, `with_sdk_token`) are being removed, not deprecated — clean break
- Config inspection via properties is useful for debugging connection issues

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 02-python-binding*
*Context gathered: 2026-02-01*
