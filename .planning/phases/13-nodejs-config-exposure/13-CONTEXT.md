# Phase 13: Node.js Config Exposure - Context

**Gathered:** 2026-02-05
**Status:** Ready for planning

<domain>
## Phase Boundary

Add options-based constructor and config exposure to Node.js binding. This includes `WebSocketOptions`, `ReconnectOptions`, and `HealthCheckOptions` napi objects, modified `RestClient` and `WebSocketClient` to accept options objects, runtime validation despite TypeScript types, and updated `index.d.ts` with config option types.

</domain>

<decisions>
## Implementation Decisions

### Constructor Pattern
- Match @fugle/marketdata official SDK structure exactly
- Options object with flat properties: `{ apiKey, bearerToken, sdkToken, baseUrl, healthCheck, reconnect }`
- `reconnect` config exposed at top-level (same level as `healthCheck`) — our SDK enhancement beyond official API
- Use camelCase property names to match official SDK: `apiKey`, `bearerToken`, `sdkToken`, `baseUrl`, `healthCheck`, `reconnect`
- Official SDK reference: `ClientOptions`, `WebSocketClientOptions`, `HealthCheckConfig` interfaces

### TypeScript Experience
- Use union types to enforce exactly-one-auth at compile time: `{ apiKey: string } | { bearerToken: string } | { sdkToken: string }`
- Explicit optional fields for config objects (not `Partial<T>`): `{ maxAttempts?: number, initialDelayMs?: number }`
- Minimal JSDoc documentation — types only, no comments
- Export all types for user convenience: `RestClientOptions`, `WebSocketClientOptions`, `ReconnectOptions`, `HealthCheckOptions`

### Validation Behavior
- Fail-fast at constructor time — `new RestClient({...})` throws immediately if invalid
- Use standard `Error` type for validation failures (not custom error classes)
- Match Python error message format from Phase 12: "field must be >= value, got actual"
- Always validate at runtime even when TypeScript types are correct — protects JavaScript users

### Migration from v0.2.x
- Remove string constructor immediately — only options object accepted (breaking change)
- No deprecation warnings — clean break for v0.3.0
- API reference documentation only — no before/after migration examples
- Version bump deferred to Phase 15 — all languages release 0.3.0 together

### Claude's Discretion
- napi-rs object structure implementation details
- Internal FFI conversion patterns
- Test file organization

</decisions>

<specifics>
## Specific Ideas

- Official SDK exports: `ClientOptions`, `HealthCheckConfig`, `WebSocketClientOptions`
- Official SDK `HealthCheckConfig` shape: `{ enabled: boolean, pingInterval?: number, maxMissedPongs?: number }`
- Our `ReconnectOptions` is an enhancement — not in official SDK

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 13-nodejs-config-exposure*
*Context gathered: 2026-02-05*
