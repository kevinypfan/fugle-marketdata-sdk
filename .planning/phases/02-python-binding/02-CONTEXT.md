# Phase 2: Python Binding Enhancement - Context

**Gathered:** 2026-01-31
**Status:** Ready for planning

<domain>
## Phase Boundary

Modernize Python binding to PyO3 0.27+ with native asyncio support and full API compatibility with fugle-marketdata-python. Python users can use async/await syntax, see full IDE autocomplete, and replace `import fugle_marketdata` without changing method calls.

</domain>

<decisions>
## Implementation Decisions

### Async API design
- Async-only API (no sync wrappers) — market data is inherently real-time, matches official SDK patterns
- Rate limiting: Configurable auto-retry with exponential backoff (default enabled, can be disabled for manual handling)

### Module & import structure
- Match official SDK import paths exactly: `from fugle_marketdata import RestClient, WebSocketClient`
- Drop-in replacement goal: existing code should work by changing only the pip package name

### WebSocket streaming pattern
- Support BOTH async iterator (`async for msg in ws.subscribe(...)`) AND callback patterns (`ws.on_message(handler)`)
- Configurable auto-reconnect with exponential backoff (default enabled)
- Queue subscription changes during disconnection, replay after reconnect
- Parse errors: Skip malformed messages and continue streaming (resilient), but also emit error events for users who want visibility

### Error handling & types
- Full exception hierarchy: `FugleError` → `ApiError`, `AuthError`, `RateLimitError`, `ConnectionError`, `TimeoutError`, etc.
- Response model fields use `Optional[T] = None` for explicit nullability where API can return null

### Claude's Discretion
- Connection lifecycle management (context manager vs explicit connect/close — likely both)
- Timeout behavior (client-level default + per-call override — likely both)
- Types submodule organization and `__all__` exports (match official SDK structure)
- py.typed marker and .pyi stub generation (follow PyO3 best practices)
- API error response structure (structured object with code/message + raw response for debugging)
- Response model mutability (frozen dataclasses preferred for safety)

</decisions>

<specifics>
## Specific Ideas

- "Match official SDK exactly" — import paths and method signatures should allow drop-in replacement
- Both iterator and callback patterns for WebSocket to support different use cases
- Resilient streaming that doesn't crash on malformed messages but still exposes errors

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 02-python-binding*
*Context gathered: 2026-01-31*
