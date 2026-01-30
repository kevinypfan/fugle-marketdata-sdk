# Phase 4: C# Binding Replacement - Context

**Gathered:** 2026-01-31
**Status:** Ready for planning

<domain>
## Phase Boundary

Replace UniFFI architecture with csbindgen for idiomatic .NET interop. Create C# API with Task-based async, EventHandler streaming, and graceful panic handling. Must be API-compatible drop-in replacement for FubonNeo SDK style while following .NET conventions.

</domain>

<decisions>
## Implementation Decisions

### API Surface Design
- **Dual API approach during transition**: Primary idiomatic .NET API (PascalCase, async suffixes) with compatibility aliases matching official Fugle SDK method names
- **Target frameworks**: Multi-target netstandard2.0 and net6.0 (matching FubonNeo)
- **Response types**: Use C# 9+ record types (immutable) for all response DTOs
- **Instantiation**: Claude's discretion — choose pattern based on .NET SDK conventions

### Async Pattern Strategy
- **CancellationToken**: Required parameter on all async methods (not optional with default)
- **Async bridging**: Claude's discretion — prototype Task.Run vs callback-based polling, choose based on performance/complexity tradeoff
- **ConfigureAwait(false)**: Always use internally to prevent UI context deadlocks
- **Sync/Async methods**: Provide both synchronous and async versions of all methods
  - Sync versions must avoid deadlocks (use `.GetAwaiter().GetResult()` with proper context)

### Event/Streaming Model
- **Event pattern**: EventHandler<T> for WebSocket events (not IObservable or IAsyncEnumerable)
- **Connection lifecycle**: Separate events — OnConnected, OnDisconnected, OnReconnecting, OnError
- **Auto-reconnect**: Enabled by default with configurable backoff
- **Disposal**: Implement both IDisposable and IAsyncDisposable for proper cleanup

### Error Handling
- **Exception hierarchy**: Custom base exception `FugleException` with derived types:
  - `ApiException` for HTTP errors
  - `AuthException` for authentication failures
  - `RateLimitException` with RetryAfter property
  - `ConnectionException` for WebSocket issues
  - `FugleInternalException` for caught Rust panics
- **Rate limiting**: Throw RateLimitException with retry info (no auto-retry)
- **Panic recovery**: Rust `catch_unwind` → error code → C# FugleInternalException (no process abort)
- **Connection errors**: Immediate exception for explicit ConnectAsync() calls; auto-reconnect only for established connections

### Claude's Discretion
- Client instantiation pattern (builder, options object, or simple constructor)
- Async bridging implementation (Task.Run vs callback polling) — prototype both if needed
- Exact auto-reconnect backoff algorithm
- Internal threading model for event dispatch

</decisions>

<specifics>
## Specific Ideas

- Match FubonNeo's multi-targeting approach (netstandard2.0;net6.0) for maximum compatibility
- Record types for responses align with modern C# immutable data patterns
- EventHandler pattern is the established norm for .NET event-driven APIs
- Both sync and async methods support legacy codebases and modern async/await usage

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 04-csharp-binding*
*Context gathered: 2026-01-31*
