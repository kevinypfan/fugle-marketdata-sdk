# Feature Landscape: Multi-Language SDK

**Domain:** Financial Market Data SDK (Rust core with Python/Node.js/C# bindings)
**Researched:** 2026-01-30

## Executive Summary

This research analyzes the feature landscape for professional multi-language SDKs, with specific focus on maintaining API compatibility with existing Fugle MarketData SDKs across Python, Node.js, and C#. Features are categorized into table stakes (must-have for user retention), differentiators (competitive advantages), and anti-features (deliberate exclusions).

**Key Finding:** Modern SDKs in 2026 require language-idiomatic design over one-size-fits-all approaches. Type safety, async patterns, and error handling must feel native to each language while maintaining functional parity across all bindings.

## Table Stakes

Features users expect. Missing any = users leave for alternatives.

### 1. Language-Idiomatic API Design
**Why Expected:** Developers expect SDKs to feel native to their language, not like a foreign library ported awkwardly.

**Complexity:** High

**Requirements:**
- **Python:** snake_case naming, property decorators, context managers, async/await with asyncio
- **Node.js:** camelCase naming, Promise-based APIs, EventEmitter patterns, async/await
- **C#:** PascalCase naming, async Task<T> methods, IDisposable pattern, LINQ support
- **All:** Follow language-specific error handling conventions (exceptions vs Result types)

**Current State in Official SDKs:**
- Python: Uses snake_case, context managers not present
- Node.js: Uses camelCase, Promise-based
- C#: Uses PascalCase, async patterns

**Source:** [Azure SDK Design Guidelines](https://azure.github.io/azure-sdk/python_design.html) - HIGH confidence

### 2. Comprehensive Error Handling
**Why Expected:** Developers need actionable error information for debugging and production monitoring.

**Complexity:** Medium

**Requirements:**
- Custom exception types with debugging context
- HTTP status codes, URLs, request parameters
- Idiomatic error patterns per language:
  - **Python:** Exceptions with rich context (FugleAPIError ✓)
  - **Node.js:** Rejected promises with error objects
  - **C#:** Typed exceptions with InnerException
- Network error vs API error differentiation
- Retry-able vs non-retry-able error classification

**Current State:** Python SDK has excellent FugleAPIError with debug context. Node.js needs equivalent.

**Source:** [Standard Error Handling Patterns in SDKs](https://www.stainless.com/sdk-api-best-practices/standard-error-handling-patterns-in-sdks-across-languages) - HIGH confidence

### 3. Async/Await Support (Language-Native)
**Why Expected:** Modern applications are async-first. Blocking I/O is unacceptable in 2026.

**Complexity:** High

**Requirements:**
- **Python:** asyncio-based with async/await syntax
- **Node.js:** Promise-based with async/await syntax
- **C#:** Task-based async pattern (TAP) with async/await
- Non-blocking I/O for all network operations
- Proper cancellation token support (C#), AbortSignal (Node.js), asyncio.Task.cancel() (Python)

**Current State:**
- Node.js: Promises via then() chains, needs async/await examples
- Python: Limited async support, uses synchronous requests library
- C#: Unknown async support

**Source:** [WebSocket Async Patterns](https://websockets.readthedocs.io/en/stable/howto/patterns.html) - HIGH confidence

### 4. Type Safety and Completion
**Why Expected:** IDEs with autocomplete are standard. Developers expect IntelliSense/type hints.

**Complexity:** Medium

**Requirements:**
- **Python:** Type hints (PEP 484) with typed return values, mypy compatibility
- **Node.js:** TypeScript definitions (.d.ts files) with complete coverage
- **C#:** Strong typing with generics
- Typed request/response models
- Enum types for constants (channels, market types)

**Current State:**
- Node.js: Has TypeScript source (✓)
- Python: Needs type hints added
- C#: Strongly typed (✓)

**Source:** [Building Great SDKs](https://newsletter.pragmaticengineer.com/p/building-great-sdks) - HIGH confidence

### 5. Dual Client Pattern (REST + WebSocket)
**Why Expected:** Market data requires both historical queries (REST) and real-time streams (WebSocket).

**Complexity:** High

**Requirements:**
- Separate clients with consistent initialization patterns
- Shared authentication mechanism
- Factory pattern for client creation
- Consistent method naming across both clients
- WebSocket reconnection logic with exponential backoff
- Health check/ping-pong for connection monitoring

**Current State:** Official SDKs have this (✓)

**Source:** Existing official SDK implementation - HIGH confidence

### 6. Authentication Flexibility
**Why Expected:** Different deployment scenarios need different auth methods.

**Complexity:** Low

**Requirements:**
- API Key authentication (X-API-KEY header)
- Bearer token authentication (Authorization header)
- SDK token authentication (X-SDK-TOKEN header)
- Mutual exclusion validation (only one auth method at a time)
- Environment variable support for keys

**Current State:** Official SDKs support all three methods (✓)

**Source:** Existing official SDK implementation - HIGH confidence

### 7. Connection Management
**Why Expected:** Production applications need reliable, efficient network connections.

**Complexity:** Medium

**Requirements:**
- HTTP connection pooling (reuse connections)
- Configurable timeouts (connect, read, total)
- Automatic retry with exponential backoff
- Maximum retry attempts configuration
- WebSocket automatic reconnection
- Graceful degradation on connection loss

**Current State:**
- Python: Uses requests (has connection pooling)
- Node.js: Uses isomorphic-fetch (basic)
- Needs configurable retry logic

**Source:** [AWS SDK Best Practices](https://docs.aws.amazon.com/sdk-for-java/latest/developer-guide/best-practices.html) - HIGH confidence

### 8. Comprehensive Testing
**Why Expected:** SDKs are infrastructure. Broken SDKs break user applications.

**Complexity:** Medium

**Requirements:**
- Unit tests for all public APIs
- Mock server for REST endpoints
- Mock WebSocket server for stream testing
- Test coverage >80%
- Language-specific test frameworks:
  - Python: pytest
  - Node.js: Jest
  - C#: xUnit or MSTest
- CI/CD integration

**Current State:** Official SDKs have test infrastructure (✓)

**Source:** [SDK Testing Best Practices](https://www.speakeasy.com/blog/sdk-best-practices) - HIGH confidence

### 9. Clear Documentation
**Why Expected:** Developers evaluate SDKs by documentation quality before adoption.

**Complexity:** Medium

**Requirements:**
- Getting started guide with installation
- Authentication setup examples
- Code examples for common use cases
- API reference (auto-generated from code)
- Error handling examples
- Migration guides between versions
- Language-specific idioms documented

**Current State:** Official SDKs have basic README, need expansion

**Source:** [Code Documentation Best Practices](https://www.qodo.ai/blog/code-documentation-best-practices-2026/) - MEDIUM confidence

### 10. Semantic Versioning
**Why Expected:** Breaking changes without version signals destroy user trust.

**Complexity:** Low

**Requirements:**
- Follow SemVer (MAJOR.MINOR.PATCH)
- MAJOR: Breaking API changes
- MINOR: Backward-compatible features
- PATCH: Backward-compatible bug fixes
- Changelog with migration instructions
- Deprecation warnings before removal
- Maintain N-1 version compatibility

**Current State:** Official SDKs use semantic versioning (✓)

**Source:** [LaunchDarkly SDK Versioning](https://docs.launchdarkly.com/sdk/concepts/versioning) - HIGH confidence

## Differentiators

Features that set excellent SDKs apart. Not expected, but highly valued.

### 1. Zero-Copy Performance (Rust Core)
**Value Proposition:** 10-100x faster parsing and serialization compared to pure Python/Node.js implementations.

**Complexity:** High

**Implementation:**
- Rust core handles JSON parsing with serde
- Zero-copy message passing to language bindings
- Shared memory buffers where possible
- Benchmark against pure-language implementations

**Competitive Advantage:** Most market data SDKs use pure-language implementations. Rust core = significant performance edge for high-frequency use cases.

**Source:** Rust performance characteristics - HIGH confidence

### 2. Smart Retry with Circuit Breaker
**Value Proposition:** Prevent cascading failures and excessive retries during API outages.

**Complexity:** Medium

**Implementation:**
- Exponential backoff with jitter
- Circuit breaker pattern (closed → open → half-open)
- Configurable failure thresholds
- Health check integration
- Per-endpoint circuit breakers

**Competitive Advantage:** Most SDKs do basic retries. Circuit breakers prevent retry storms during outages.

**Source:** [AWS SDK Retry Behavior](https://docs.aws.amazon.com/sdkref/latest/guide/feature-retry-behavior.html) - HIGH confidence

### 3. Type-Safe Request Builders
**Value Proposition:** Prevent invalid API requests at compile time, not runtime.

**Complexity:** Medium

**Implementation:**
- Builder pattern for complex requests
- Compile-time validation of required parameters
- Fluent API design
- IntelliSense-friendly method chaining

**Example (Python):**
```python
client.stock.intraday.candles(
    symbol="2330",
    timeframe="1m"  # Enum validates at type-check time
).from_date("2024-01-01")
```

**Competitive Advantage:** Reduces API errors and improves developer experience.

**Source:** Industry pattern - MEDIUM confidence

### 4. Streaming Data Aggregation
**Value Proposition:** Built-in helpers for common streaming patterns (windowing, aggregation).

**Complexity:** High

**Implementation:**
- Rolling window aggregation (e.g., 5-minute VWAP from tick data)
- Time-based buffering
- Backpressure handling
- Observable/reactive patterns where idiomatic

**Competitive Advantage:** Users often need to aggregate tick data. Built-in support saves implementation time.

**Source:** Domain knowledge - MEDIUM confidence

### 5. Comprehensive Logging and Tracing
**Value Proposition:** Production debugging and monitoring.

**Complexity:** Medium

**Implementation:**
- Structured logging with log levels
- Optional request/response logging (with PII filtering)
- Distributed tracing integration (OpenTelemetry)
- Performance metrics (latency, throughput)
- Language-specific logging conventions:
  - Python: logging module
  - Node.js: console/winston/pino
  - C#: ILogger interface

**Competitive Advantage:** Enterprise users need observability. Few SDKs provide it out-of-box.

**Source:** [OpenTelemetry SDKs](https://opentelemetry.io/docs/languages/) - HIGH confidence

### 6. Rate Limit Handling
**Value Proposition:** Automatic handling of API rate limits.

**Complexity:** Medium

**Implementation:**
- Detect 429 status codes
- Parse Retry-After headers
- Automatic queuing and pacing
- Expose rate limit status to users
- Optional: Token bucket algorithm for proactive limiting

**Competitive Advantage:** Prevents user applications from hitting rate limits unknowingly.

**Source:** API best practices - HIGH confidence

### 7. Offline Mode / Request Recording
**Value Proposition:** Development and testing without live API access.

**Complexity:** Medium

**Implementation:**
- Record API responses to disk
- Replay mode for testing
- Mock data generation
- Environment-based toggle (dev vs production)

**Competitive Advantage:** Accelerates development and reduces API call costs during testing.

**Source:** Development workflow patterns - MEDIUM confidence

### 8. Migration Utilities
**Value Proposition:** Smooth migration from official SDKs to Rust-backed SDK.

**Complexity:** Low

**Implementation:**
- Compatibility layer for old API patterns
- Migration checker script
- Side-by-side usage support
- Deprecation warnings with alternatives

**Competitive Advantage:** Reduces friction for existing users to adopt the new SDK.

**Source:** Migration strategy - MEDIUM confidence

## Anti-Features

Things to deliberately NOT build. Common mistakes in this domain.

### 1. Embedded Data Storage
**What:** Built-in database for caching market data
**Why Avoid:** Scope creep. Storage is orthogonal to data fetching. Users have their own storage solutions.
**Instead:** Provide clear examples of integration with common databases (PostgreSQL, Redis, TimescaleDB).

**Source:** SDK scope discipline - HIGH confidence

### 2. Complex Configuration Files
**What:** YAML/TOML/JSON configuration files for SDK settings
**Why Avoid:** Configuration as code is superior. Files add deployment complexity and make testing harder.
**Instead:** Programmatic configuration via constructor options. Support environment variables for deployment.

**Source:** [Modern C# Error Handling Patterns](https://medium.com/@tejaswini.nareshit/modern-c-error-handling-patterns-you-should-be-using-in-2026-57eacd495123) - MEDIUM confidence

### 3. GUI/Web Dashboard
**What:** Built-in web interface for monitoring or testing
**Why Avoid:** Not the SDK's job. Adds massive scope and maintenance burden.
**Instead:** Focus on excellent logging and metrics that integrate with existing monitoring tools.

**Source:** SDK focus principle - HIGH confidence

### 4. Built-in Backtesting Framework
**What:** Historical data backtesting engine
**Why Avoid:** Backtesting has unique requirements (slippage, commissions, etc.). Separate library or user responsibility.
**Instead:** Ensure historical data APIs are fast and complete. Document integration with backtesting libraries.

**Source:** Domain separation - HIGH confidence

### 5. Custom Async Runtime
**What:** Custom event loop or async runtime instead of language-native
**Why Avoid:** Incompatible with user applications' existing async code. Integration nightmare.
**Instead:** Use standard async patterns: asyncio (Python), Tokio/async-std for Rust internals, native Promises (Node.js), Task (C#).

**Source:** [WebSocket Async Patterns](https://websockets.readthedocs.io/en/stable/howto/patterns.html) - HIGH confidence

### 6. Synchronous-Only APIs
**What:** Only blocking/synchronous API methods
**Why Avoid:** Modern applications are async-first. Synchronous-only SDKs force users into threading complexity.
**Instead:** Async-first design with optional synchronous wrappers where language conventions support it (Python can have sync + async APIs).

**Source:** Modern async patterns - HIGH confidence

### 7. Global Singletons
**What:** Single global client instance (e.g., `fugle.client`)
**Why Avoid:** Prevents multiple configurations in same application (e.g., different API keys for different accounts).
**Instead:** Explicit client instantiation. Users can create singletons if desired.

**Source:** API design patterns - HIGH confidence

### 8. Automatic Retry on User Errors (4xx)
**What:** Retrying requests that fail with 400-level errors
**Why Avoid:** User errors (bad parameters, auth failures) won't succeed on retry. Wastes resources and delays error feedback.
**Instead:** Only retry transient errors (5xx, network errors, timeouts). Fail fast on client errors.

**Source:** [AWS SDK Retry Behavior](https://docs.aws.amazon.com/sdkref/latest/guide/feature-retry-behavior.html) - HIGH confidence

### 9. Implicit Magic Behavior
**What:** Automatic behavior users don't explicitly request (e.g., auto-reconnect without notification)
**Why Avoid:** "Principle of least surprise" - magic behavior causes debugging confusion.
**Instead:** Explicit opt-in for conveniences. Emit events/logs when automatic actions occur.

**Source:** API design philosophy - HIGH confidence

### 10. Breaking Changes Without Major Version
**What:** Changing method signatures or removing features in minor/patch versions
**Why Avoid:** Destroys user trust and breaks production applications.
**Instead:** Strict semantic versioning. Deprecate → warn → remove across major versions.

**Source:** [API Versioning Best Practices](https://blog.xapihub.io/2024/06/19/API-Design-Anti-patterns.html) - HIGH confidence

## Feature Dependencies

```
Authentication Configuration
  ↓
Connection Management (uses auth)
  ↓
REST Client ←→ WebSocket Client
  ↓                    ↓
Error Handling  ←  Type Safety
  ↓                    ↓
Logging & Tracing    Testing
  ↓
Rate Limiting & Retry Logic
  ↓
Circuit Breaker (optional)
```

**Critical Path:**
1. Authentication → Connection → REST/WebSocket → Error Handling
2. Type Safety enables: Request Builders, IDE Support, Compile-time Validation
3. Testing depends on: All core features being stable

## Feature Complexity Analysis

| Feature | Python | Node.js | C# | Notes |
|---------|--------|---------|----|----|
| Idiomatic API Design | High | High | High | Different conventions per language |
| Error Handling | Medium | Medium | Medium | FugleAPIError pattern established |
| Async/Await | High | Low | Medium | Python needs asyncio rewrite |
| Type Safety | Medium | Low | Low | Python needs hints, Node has TS |
| Dual Client | Medium | Medium | Medium | Pattern exists in official SDKs |
| Authentication | Low | Low | Low | Already implemented |
| Connection Mgmt | Medium | Medium | Medium | Retry logic needs addition |
| Testing | Medium | Medium | Medium | Frameworks in place |
| Documentation | Low | Low | Low | Expand existing docs |
| Versioning | Low | Low | Low | Already following SemVer |

## Language-Specific Feature Matrix

### Async Patterns

| Language | Pattern | Example |
|----------|---------|---------|
| Python | asyncio + async/await | `await client.stock.intraday.quote(symbol="2330")` |
| Node.js | Promise + async/await | `await client.stock.intraday.quote({ symbol: '2330' })` |
| C# | Task<T> + async/await | `await client.Stock.Intraday.Quote("2330")` |

### Error Handling

| Language | Pattern | Example |
|----------|---------|---------|
| Python | Exception | `try: ... except FugleAPIError as e: print(e.status_code)` |
| Node.js | Rejected Promise | `try { await ... } catch (error) { console.error(error.statusCode) }` |
| C# | Exception | `try { await ... } catch (FugleAPIException ex) { Console.WriteLine(ex.StatusCode); }` |

### Type Safety

| Language | Pattern | Example |
|----------|---------|---------|
| Python | Type hints | `def quote(symbol: str) -> Quote: ...` |
| Node.js | TypeScript | `function quote(symbol: string): Promise<Quote>` |
| C# | Generics | `Task<Quote> Quote(string symbol)` |

### Resource Management

| Language | Pattern | Example |
|----------|---------|---------|
| Python | Context manager | `async with WebSocketClient(api_key=key) as client: ...` |
| Node.js | Manual cleanup | `await client.disconnect()` |
| C# | IDisposable | `using var client = new WebSocketClient(apiKey); ...` |

## MVP Recommendation

For initial Rust-backed SDK release, prioritize:

### Phase 1: Core Table Stakes (Must-Have)
1. ✅ **Idiomatic API Design** - Matches existing SDK conventions
2. ✅ **Error Handling** - FugleAPIError with debug context
3. ✅ **Basic Async Support** - Language-native patterns
4. ✅ **Type Safety** - Type hints/definitions where missing
5. ✅ **Dual Client Pattern** - REST + WebSocket
6. ✅ **Authentication** - All three methods (already done)
7. ⚠️ **Connection Management** - Add retry logic with exponential backoff
8. ✅ **Basic Testing** - Unit tests for core functionality
9. ⚠️ **Documentation** - Expand README with examples
10. ✅ **Versioning** - SemVer with changelog

### Phase 2: Enhanced Reliability (High Value)
1. **Smart Retry with Circuit Breaker** - Production reliability
2. **Comprehensive Logging** - Debugging and monitoring
3. **Rate Limit Handling** - Automatic 429 handling
4. **WebSocket Reconnection** - Automatic with backoff
5. **Health Checks** - Connection monitoring

### Phase 3: Developer Experience (Differentiators)
1. **Type-Safe Request Builders** - Fluent API
2. **Migration Utilities** - Smooth transition from official SDKs
3. **Offline Mode** - Development without live API
4. **Performance Benchmarks** - Demonstrate Rust advantage

### Defer to Post-MVP
- **Streaming Data Aggregation** - Complex, niche use case
- **Distributed Tracing** - Enterprise feature, small user base
- **Advanced Observability** - Logging sufficient for MVP

## Testing Strategy

### Unit Testing
- **Python:** pytest with pytest-asyncio, pytest-mock
- **Node.js:** Jest with jest-websocket-mock
- **C#:** xUnit with Moq

### Integration Testing
- Mock REST server (WireMock, nock, or built-in mocks)
- Mock WebSocket server (jest-websocket-mock, pytest-websocket)
- Test against recorded API responses

### Compatibility Testing
- Test matrix: Python 3.8-3.12, Node.js 18-22 LTS, .NET 6-8
- Platform matrix: Windows, Linux, macOS
- Architecture matrix: x64, ARM64

### Performance Testing
- Benchmark parsing speed (Rust vs pure-language)
- Memory usage profiling
- Connection pool efficiency
- Throughput testing (messages/second)

## Migration Path from Official SDKs

### Compatibility Targets
1. **API-level compatibility:** Same method names and signatures
2. **Behavioral compatibility:** Same defaults and error messages
3. **Drop-in replacement:** Minimal code changes required

### Migration Checklist
- [ ] All official SDK public APIs implemented
- [ ] Error messages match official SDK format
- [ ] Default behavior matches (timeouts, retry counts)
- [ ] Environment variable names match
- [ ] Response object structure identical
- [ ] WebSocket event names identical

## Open Questions

1. **Async Python:** Should we provide both sync and async APIs, or async-only with sync wrapper?
   - **Recommendation:** Async-only for cleaner codebase. Users who need sync can use `asyncio.run()`.

2. **Node.js Streams:** Should WebSocket data use Node.js Streams API or EventEmitter?
   - **Recommendation:** Keep EventEmitter (matches official SDK), consider Streams as optional future feature.

3. **C# Naming:** Should we match Python naming (lowercase methods) or C# conventions (PascalCase)?
   - **Recommendation:** C# conventions (PascalCase) for language idiomaticity.

4. **Type Definitions:** Auto-generate from OpenAPI spec or hand-write?
   - **Recommendation:** Auto-generate where possible, manual for WebSocket types.

5. **Rust Exposure:** Should we expose Rust core as separate package for power users?
   - **Defer:** Start with bindings only. Consider Rust crate later if demand exists.

## Confidence Assessment

| Category | Confidence | Rationale |
|----------|-----------|-----------|
| Table Stakes | HIGH | Based on official SDK analysis + industry standards |
| Async Patterns | HIGH | Official docs + established language conventions |
| Error Handling | HIGH | Existing FugleAPIError pattern + Stainless research |
| Type Safety | MEDIUM | Language-specific, needs per-language verification |
| Differentiators | MEDIUM | Based on AWS/Azure SDK patterns, some speculative |
| Anti-Features | HIGH | Well-documented SDK mistakes in industry |
| Testing Strategy | HIGH | Standard tooling per language |
| Migration Path | HIGH | Direct analysis of official SDK codebases |

## Sources

### High Confidence
- [Building Great SDKs - Pragmatic Engineer](https://newsletter.pragmaticengineer.com/p/building-great-sdks)
- [Azure SDK Design Guidelines](https://azure.github.io/azure-sdk/python_design.html)
- [Standard Error Handling Patterns in SDKs](https://www.stainless.com/sdk-api-best-practices/standard-error-handling-patterns-in-sdks-across-languages)
- [AWS SDK Best Practices](https://docs.aws.amazon.com/sdk-for-java/latest/developer-guide/best-practices.html)
- [AWS SDK Retry Behavior](https://docs.aws.amazon.com/sdkref/latest/guide/feature-retry-behavior.html)
- [LaunchDarkly SDK Versioning](https://docs.launchdarkly.com/sdk/concepts/versioning)
- [WebSocket Async Patterns (Python)](https://websockets.readthedocs.io/en/stable/howto/patterns.html)
- [OpenTelemetry Language SDKs](https://opentelemetry.io/docs/languages/)
- Official SDK codebases: fugle-marketdata-python, fugle-marketdata-node, FubonNeo

### Medium Confidence
- [Modern API Design Best Practices 2026](https://www.xano.com/blog/modern-api-design-best-practices/)
- [Modern C# Error Handling Patterns 2026](https://medium.com/@tejaswini.nareshit/modern-c-error-handling-patterns-you-should-be-using-in-2026-57eacd495123)
- [SDK Testing Best Practices](https://www.speakeasy.com/blog/sdk-best-practices)
- [Code Documentation Best Practices 2026](https://www.qodo.ai/blog/code-documentation-best-practices-2026/)
- [API Design Anti-patterns](https://blog.xapihub.io/2024/06/19/API-Design-Anti-patterns.html)
- [Best SDK Generation Tools 2025](https://buildwithfern.com/post/best-sdk-generation-tools-for-multi-language-api-development-november-2025)

### Analysis Sources
- Direct code analysis of three official SDK implementations
- Industry patterns from AWS, Azure, Google Cloud SDKs
- Language-specific documentation (Python PEPs, TypeScript handbook, C# guidelines)
