# Phase 6: Testing & Production Readiness - Context

**Gathered:** 2026-01-31
**Status:** Ready for planning

<domain>
## Phase Boundary

Validate API compatibility, integration correctness, and production reliability through comprehensive test coverage across all language bindings (Python, Node.js, C#, Go, Java). This phase creates the test infrastructure and validates the SDK against official Fugle SDKs — it does not add new features or endpoints.

</domain>

<decisions>
## Implementation Decisions

### Compatibility Testing Scope
- Full drop-in replacement compatibility — both method signatures AND response structures must match official SDKs
- Reference SDKs: fugle-marketdata-python and fugle-marketdata-node (both official SDKs)
- Unimplemented features: Skip with clear marker in test file (e.g., `@pytest.mark.skip(reason="not yet implemented: historical endpoints")`)
- Use recorded fixtures from official SDKs — capture responses once, compare this SDK against snapshots
- No live comparison needed; fixtures provide stable baseline

### Integration Test Strategy
- Credentials via environment variable: `FUGLE_API_KEY` required, tests skip gracefully if missing
- Coverage: All implemented endpoints — REST intraday, WebSocket streaming, whatever the SDK supports
- Test symbols: Configurable via environment variable (e.g., `FUGLE_TEST_SYMBOLS=2330.TW,2317.TW`)
- Default to well-known symbols (2330.TW TSMC) when not configured

### Cross-Language Test Consistency
- Coverage standard: Critical path coverage — core workflows tested, edge cases based on risk
- Structural tests: Use reflection-based validation to verify API surface without native library
- Tests should pass structurally even when native library unavailable

### Performance Benchmarks
- Metrics: Latency, throughput, and memory usage — comprehensive profile
- Baseline: Compare against official Fugle SDKs (fugle-marketdata-python, fugle-marketdata-node)
- Benchmark cadence: CI on every PR with regression detection

### Claude's Discretion
- Rate limit handling strategy (sequential with delays, retry with backoff, or quota-aware batching)
- Cross-language test consistency approach (shared spec vs language-idiomatic)
- FFI boundary test strategy (language-specific vs shared test cases)
- Performance threshold definition (what's acceptable vs official SDKs)

</decisions>

<specifics>
## Specific Ideas

- Recorded fixtures approach aligns with existing test patterns in Phase 2-4 (pytest-asyncio, Jest, xUnit already use fixture-based testing)
- Reflection-based structural tests already exist in Java binding (04.2-03) — extend pattern to all bindings
- Environment variable pattern for credentials matches existing CI workflow patterns from Phase 5

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 06-testing-production-readiness*
*Context gathered: 2026-01-31*
