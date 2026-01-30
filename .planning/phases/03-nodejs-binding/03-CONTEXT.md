# Phase 3: Node.js Binding Enhancement - Context

**Gathered:** 2026-01-31
**Status:** Ready for planning

<domain>
## Phase Boundary

Upgrade Node.js binding to napi-rs 3.6+ with improved TypeScript definitions and API compatibility with fugle-marketdata-node. Users can replace `require('@fugle/marketdata')` with this SDK without changing method signatures or response structures. WebSocket streaming emits events through EventEmitter pattern without memory leaks.

</domain>

<decisions>
## Implementation Decisions

### TypeScript API Surface
- Fully typed responses — every field has explicit type, nested objects typed, no any/unknown
- Hybrid type generation — napi-rs auto-generates base types, hand-curate public API surface for documentation and accuracy
- Generic vs concrete types: Claude's discretion based on what matches official SDK patterns
- Optional fields: Claude's discretion based on official SDK patterns (likely `field?: Type`)

### Async/Promise Patterns
- Typed error classes — ApiError, AuthError, RateLimitError for catch-specific handling
- Timeout configuration — both client-level default AND per-request override option
- No AbortController/cancellation support for now — keep REST simple, defer to future phase when core has async HTTP
- All REST methods return Promises with proper rejection typing

### EventEmitter Design
- Event names match official SDK exactly — inspect @fugle/marketdata and replicate naming
- Strongly typed EventEmitter — `on('message', (data: QuoteData) => {})` with TypeScript-aware payloads
- Buffer and emit all messages — queue if JS event loop is slow, don't drop messages
- Auto-reconnect built-in with configurable options — retry delay, max attempts, exponential backoff

### API Compatibility
- Method signatures: exact match with official @fugle/marketdata — same names, parameter order, optional params
- Response shapes: exact field names, casing, nesting — existing code works unchanged
- Error codes: Claude investigates official patterns and matches where defined
- May note optional improvements where official API is awkward, but as additions not replacements
- Structural API compatibility tests verify method names, signatures, response types match official SDK

</decisions>

<specifics>
## Specific Ideas

- "希望是 Exact match 但如果有更好的建議可以提" — compatibility is baseline, Claude can suggest improvements as opt-in additions
- Drop-in replacement goal — user changes only the import statement, everything else works

</specifics>

<deferred>
## Deferred Ideas

- AbortController/request cancellation — requires async HTTP in core, defer to future phase
- True request cancellation (not fake drop) — needs core architecture change

</deferred>

---

*Phase: 03-nodejs-binding*
*Context gathered: 2026-01-31*
