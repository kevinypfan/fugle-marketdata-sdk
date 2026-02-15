# Phase 14: Java & Go Bindings - Context

**Gathered:** 2026-02-15
**Status:** Ready for planning

<domain>
## Phase Boundary

Add config exposure (options-based constructors, reconnect config, health check config) to the three UniFFI-based language bindings: Java, Go, and C#. No official Fugle SDKs exist for these languages, so API design follows each language's idiomatic conventions. The existing binding functionality (REST + WebSocket) from v0.2.0 remains unchanged — this phase adds configuration acceptance to existing client constructors.

</domain>

<decisions>
## Implementation Decisions

### Claude's Discretion

All API design decisions are at Claude's discretion, following each language's standard conventions:

**Java:**
- Builder pattern for client construction (e.g., `RestClient.builder().apiKey("...").build()`)
- Nested builder or separate config classes for ReconnectConfig/HealthCheckConfig
- Validation at build time with descriptive exceptions
- Follow established Java SDK patterns (similar to OkHttp, AWS SDK builders)

**Go:**
- Functional options pattern (e.g., `NewRestClient(WithApiKey("..."))`)
- Config structs with sensible zero-value defaults where possible
- Exactly-one-auth enforcement at construction time
- Follow established Go patterns (similar to grpc-go, aws-sdk-go-v2)

**C#:**
- Options class pattern with properties (e.g., `new RestClientOptions { ApiKey = "..." }`)
- Extend existing csbindgen Phase 11 work
- PascalCase naming matching .NET conventions
- Validation at construction with ArgumentException

**Cross-language:**
- Validation behavior consistent: exactly-one-auth required, same config constraints as Python/Node.js
- Error messages follow same format: include field name, constraint, and actual value
- Config defaults match core constants (DEFAULT_* values from Phase 8)
- Milliseconds at FFI boundary (same pattern as Python/Node.js)

</decisions>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches. Follow existing patterns established in Phase 12 (Python) and Phase 13 (Node.js) for validation logic and config constraint behavior.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 14-java-go-bindings*
*Context gathered: 2026-02-15*
