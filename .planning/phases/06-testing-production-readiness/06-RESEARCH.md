# Phase 6: Testing & Production Readiness - Research

**Researched:** 2026-01-31
**Domain:** Multi-language FFI testing, API compatibility validation, performance benchmarking
**Confidence:** HIGH

## Summary

Phase 6 focuses on comprehensive testing infrastructure for multi-language SDK bindings (Python, Node.js, C#, Java) built on a Rust core with FFI boundaries. The standard approach combines three testing layers: (1) structural/API compatibility tests that verify drop-in replacement capability without network calls, (2) integration tests with real API endpoints using environment-based credential management, and (3) FFI boundary tests covering panic recovery, memory safety, and cross-platform reliability.

Key insight: Testing FFI-based SDKs requires separating structural validation (which can run without native libraries) from integration/boundary tests (which require both native libraries and API credentials). This separation enables CI workflows where build artifacts from one job are used by test jobs in another, supporting cross-platform testing without rebuilding on every platform.

**Primary recommendation:** Use fixture-based testing (VCR/cassette pattern) for API compatibility tests to compare against official SDK responses, enabling fast, deterministic tests without live API dependencies or manual response recording.

## Standard Stack

The established libraries/tools for multi-language FFI SDK testing:

### Core Testing Frameworks
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| pytest | 8.x+ | Python testing | De facto standard for Python with excellent plugin ecosystem |
| pytest-asyncio | 0.25+ | Async test support | Required for testing Python async/await patterns |
| pytest-benchmark | 5.x+ | Performance testing | Industry standard for Python performance regression detection |
| pytest-timeout | 2.x+ | Deadlock detection | Critical for FFI GIL safety testing (10-15s timeouts) |
| Jest | 29.x+ | Node.js testing | Standard for Node.js with built-in async, mocking, snapshot support |
| MSTest | Latest | C# testing | Microsoft standard, integrated with Visual Studio |
| JUnit 5 | 5.x+ | Java testing | Current Java standard with modern features |

### API Compatibility & Fixtures
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| vcrpy | 6.x+ | HTTP fixture recording | Python integration tests requiring deterministic API responses |
| nock | 13.x+ | HTTP mocking | Node.js integration tests, request/response recording |
| pytest-httpx | 0.34+ | Modern async HTTP mocking | Python async client testing |
| jest-snapshot | Built-in | Response snapshots | Structural comparison of API responses |

### Performance & Benchmarking
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| github-action-benchmark | Latest | CI benchmark tracking | Automatic regression detection in GitHub Actions |
| bencher | Latest | Continuous benchmarking | Advanced statistical regression detection (200% threshold default) |
| perf_hooks | Node.js built-in | Native addon benchmarks | Node.js performance.now(), timerify for FFI overhead measurement |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| pytest-benchmark | timeit module | Benchmark has statistical analysis, CI integration; timeit is manual |
| VCR.py fixtures | Manual JSON files | VCR provides automatic recording, replay, filtering; manual is brittle |
| github-action-benchmark | Manual comparison | Action automates detection, historical tracking; manual requires custom logic |
| MSTest | xUnit | Both work, MSTest is Microsoft standard with better VS integration |

**Installation:**
```bash
# Python
pip install pytest pytest-asyncio pytest-benchmark pytest-timeout vcrpy pytest-httpx

# Node.js
npm install --save-dev jest @types/jest nock

# C# (via NuGet)
dotnet add package MSTest.TestFramework
dotnet add package MSTest.TestAdapter

# Java (via Gradle)
testImplementation 'org.junit.jupiter:junit-jupiter:5.11+'
```

## Architecture Patterns

### Recommended Test Structure
```
tests/
├── unit/                    # No network, no native library required
│   ├── api_compatibility/   # Structural tests (reflection-based)
│   └── error_handling/      # Exception hierarchy validation
├── integration/             # Require FUGLE_API_KEY, native library
│   ├── rest/                # Live REST API tests
│   ├── websocket/           # Live WebSocket streaming tests
│   └── fixtures/            # VCR cassettes or recorded responses
├── ffi/                     # FFI boundary-specific tests
│   ├── panic_recovery/      # Rust panic doesn't crash process
│   ├── memory_safety/       # No leaks, proper cleanup
│   └── gil_safety/          # Python GIL release (pytest-timeout)
├── performance/             # Benchmark tests
│   ├── baseline/            # Comparison vs official SDKs
│   └── regression/          # Track performance over time
└── conftest.py / jest.config.js  # Shared fixtures, config
```

### Pattern 1: Three-Layer Testing Strategy
**What:** Separate structural, integration, and FFI boundary tests
**When to use:** FFI-based SDKs with multiple language bindings
**Benefits:**
- Structural tests run in CI without native libraries (fast feedback)
- Integration tests gated by environment variables (skip gracefully)
- FFI tests catch boundary issues (panics, memory leaks, deadlocks)

**Example:**
```python
# Source: Existing py/tests/test_api_compatibility.py pattern
class TestRestClientAPICompatibility:
    """Structural tests - no network, no native library required."""

    def test_client_has_stock_property(self):
        """Verify API surface without execution."""
        client = RestClient("test-key")
        assert hasattr(client, 'stock')
        assert client.stock is not None

@pytest.mark.integration
class TestIntegrationRest:
    """Integration tests - require FUGLE_API_KEY."""

    @pytest.mark.asyncio
    async def test_quote_returns_dict(self, rest_client):
        result = await rest_client.stock.intraday.quote("2330")
        assert isinstance(result, dict)
        assert result['symbol'] == '2330'
```

### Pattern 2: Fixture-Based API Compatibility Testing
**What:** Record responses from official SDK, compare this SDK against snapshots
**When to use:** Validating drop-in replacement compatibility
**Benefits:**
- Fast, deterministic tests (no live API calls)
- Detects breaking changes in response structure
- Version-controlled expected behavior

**Example:**
```python
# Source: VCR.py pattern (https://vcrpy.readthedocs.io/)
import vcr

@vcr.use_cassette('fixtures/vcr_cassettes/quote_2330.yaml')
def test_quote_matches_official_sdk_structure():
    """Compare response structure to official SDK recording."""
    client = RestClient(api_key='test-key')
    response = await client.stock.intraday.quote('2330')

    # Cassette contains response from official fugle-marketdata-python
    # Structural validation
    assert 'symbol' in response
    assert 'date' in response
    assert 'bids' in response and isinstance(response['bids'], list)
    assert 'asks' in response and isinstance(response['asks'], list)
```

### Pattern 3: Environment-Based Test Skipping
**What:** Automatically skip tests when prerequisites unavailable
**When to use:** Integration tests requiring API keys or native libraries
**Benefits:**
- CI-friendly (no failures from missing credentials)
- Developer-friendly (tests pass locally without setup)
- Clear skip messages for debugging

**Example:**
```python
# Source: Existing py/tests/conftest.py pattern
def pytest_collection_modifyitems(config, items):
    """Skip integration tests if no API key is set."""
    if not os.environ.get("FUGLE_API_KEY"):
        skip_integration = pytest.mark.skip(reason="FUGLE_API_KEY not set")
        for item in items:
            if "integration" in item.keywords:
                item.add_marker(skip_integration)
```

```javascript
// Source: Existing js/tests/rest-integration.test.js pattern
const API_KEY = process.env.FUGLE_API_KEY;
const describeWithApiKey = API_KEY ? describe : describe.skip;

describeWithApiKey('REST Integration Tests', () => {
  // Tests only run when FUGLE_API_KEY is set
});
```

### Pattern 4: Native Library Availability Detection
**What:** Check if native library loads before running tests
**When to use:** FFI boundary tests, cross-platform CI with artifact sharing
**Benefits:**
- Tests pass structurally without native library
- Clear skip messages when library unavailable
- Enables test-first development before native build

**Example:**
```csharp
// Source: Existing bindings/csharp/MarketdataUniffi.Tests/RestClientTests.cs
[ClassInitialize]
public static void ClassInit(TestContext context)
{
    try {
        using var client = new RestClient("test-api-key");
        _nativeLibraryAvailable = true;
    }
    catch (DllNotFoundException) {
        _nativeLibraryAvailable = false;
    }
}

private void SkipIfNativeLibraryUnavailable()
{
    if (!_nativeLibraryAvailable) {
        Assert.Inconclusive("Native library not available. Build with: cargo build -p marketdata-uniffi --release");
    }
}
```

### Pattern 5: Performance Benchmarking with Baseline Comparison
**What:** Measure performance relative to official SDKs with regression detection
**When to use:** Performance-sensitive libraries, CI performance gates
**Benefits:**
- Catches performance regressions before merge
- Historical tracking of performance trends
- Relative comparison cancels out CI noise

**Example:**
```python
# Source: pytest-benchmark pattern (https://pytest-benchmark.readthedocs.io/)
def test_quote_performance_baseline(benchmark, rest_client):
    """Benchmark quote() against performance threshold."""
    result = benchmark(lambda: rest_client.stock.intraday.quote('2330'))

    # Assert performance within 2x of official SDK (from success criteria)
    # github-action-benchmark tracks this over time
    assert result is not None
```

### Anti-Patterns to Avoid
- **Manual response recording:** Don't manually write JSON fixtures — use VCR/cassette recording to capture from official SDK
- **Mixing structural and integration tests:** Separate concerns — structural tests shouldn't require API keys or network
- **Ignoring FFI boundary failures:** Don't assume native calls are safe — test panic recovery, memory leaks, deadlocks explicitly
- **Performance tests without baselines:** Don't measure absolute performance — compare to official SDK or historical data
- **Live API calls in CI:** Don't make real API calls for compatibility tests — use recorded fixtures for determinism

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| HTTP response recording | Manual JSON files | VCR.py (Python), nock (Node.js) | Handles headers, request matching, filtering, auto-record/replay |
| Performance regression detection | Simple time comparison | pytest-benchmark, github-action-benchmark | Statistical analysis, 200% threshold detection, CI integration |
| Async test execution | Custom event loop management | pytest-asyncio auto mode, Jest built-in | Handles cleanup, fixture integration, error propagation |
| Memory leak detection | Manual heap snapshots | napi-rs memory workflow, sanitizers | Automated detection, napi_add_finalizer, ASAN/TSAN |
| API compatibility validation | String comparison | Structural inspection + VCR fixtures | Validates both signature AND response shape |
| Cross-platform test skipping | Manual platform checks | pytest.mark, JUnit @Tag, MSTest [TestCategory] | Framework-native, conditional execution, clear reporting |

**Key insight:** FFI testing requires specialized tools that understand boundary semantics (panic recovery, GIL release, memory ownership). Don't implement custom boundary safety checks — use language-specific testing patterns (pytest-timeout for GIL, catch_unwind in Rust, Assert.Inconclusive in C#).

## Common Pitfalls

### Pitfall 1: Not Separating Structural from Integration Tests
**What goes wrong:** CI fails because tests require API keys or native libraries that aren't available in all build stages
**Why it happens:** Mixing test types in same file/suite without proper conditional execution
**How to avoid:**
- Use markers/tags: `@pytest.mark.integration`, `@Tag("integration")`, `[TestCategory("Integration")]`
- Implement collection hooks to auto-skip when prerequisites missing
- Separate test directories: `tests/unit/` vs `tests/integration/`
**Warning signs:**
- Tests fail with "DllNotFoundException" or "FUGLE_API_KEY not set" in CI
- Need to build native library before running any tests
- Can't run tests locally without API credentials

### Pitfall 2: Testing Against Live API Without Fixtures
**What goes wrong:** Tests are slow, flaky (network issues), consume API quota, fail outside trading hours
**Why it happens:** Direct API calls without recording/mocking layer
**How to avoid:**
- Record fixtures from official SDK once (VCR cassettes)
- Use recorded fixtures for compatibility tests (fast, deterministic)
- Reserve live API tests for minimal integration validation
- Use environment variable to toggle between fixture and live modes
**Warning signs:**
- Tests take >30 seconds to run
- Random test failures due to network timeouts
- Tests fail on weekends/holidays (market closed)
- High API quota consumption during test runs

### Pitfall 3: Ignoring FFI Memory Management
**What goes wrong:** Memory leaks in long-running applications, crashes in production, test suite memory bloat
**Why it happens:** Not testing resource cleanup, object lifetime, GIL release patterns
**How to avoid:**
- Use pytest-timeout for GIL safety (10-15s timeouts)
- Run memory leak detection: napi-rs has dedicated workflow (https://github.com/napi-rs/napi-rs/actions/workflows/memory-test.yml)
- Test cleanup paths: `with` statements (Python), `using` (C#), `try-finally` (Java)
- Use sanitizers in CI: ASAN/UBSAN/TSAN on native builds
**Warning signs:**
- Tests hang indefinitely (GIL deadlock)
- Memory usage increases 100MB+ per test suite (Jest + napi issue)
- Process crashes with segfault in production
- Resource cleanup requires manual intervention

### Pitfall 4: Not Testing Error Propagation Across FFI Boundary
**What goes wrong:** Native errors become process crashes instead of catchable exceptions
**Why it happens:** Missing panic recovery, unhandled error codes, NULL pointer returns
**How to avoid:**
- Wrap all FFI calls in catch_unwind (Rust side)
- Test error scenarios explicitly: invalid symbols, auth failures, network errors
- Validate exception hierarchy: ApiError, AuthError, NetworkError
- Check error messages are descriptive (not just "error code -1")
**Warning signs:**
- Process terminates instead of raising exception
- Generic error messages ("unknown error")
- Missing stack traces for native errors
- Different error types for same failure across languages

### Pitfall 5: Performance Testing Without Statistical Rigor
**What goes wrong:** False positives from CI noise, missed real regressions, arbitrary thresholds
**Why it happens:** Single-run benchmarks, absolute time comparison, no baseline
**How to avoid:**
- Use relative benchmarking: run old and new code in same CI job
- Statistical analysis: pytest-benchmark runs multiple iterations, calculates stddev
- Set thresholds based on data: github-action-benchmark defaults to 200% regression detection
- Compare to baseline: official SDK performance, not arbitrary targets
**Warning signs:**
- Benchmark results vary wildly between runs
- Performance "regressions" that aren't real
- No historical tracking of performance trends
- Thresholds like "must be < 100ms" without justification

### Pitfall 6: Incomplete API Coverage in Compatibility Tests
**What goes wrong:** Breaking changes ship because not all API surface tested
**Why it happens:** Only testing happy path, missing edge cases, optional parameters untested
**How to avoid:**
- Test all public methods: use reflection to enumerate and validate
- Test all parameters: positional, keyword, optional, default values
- Test all response fields: use snapshot/VCR fixtures to validate structure
- Test error cases: invalid symbols, auth failures, malformed requests
**Warning signs:**
- Structural tests only check method existence, not signatures
- No tests for optional parameters or default values
- Response validation only checks status code, not body structure
- Users report "breaking changes" in patch releases

## Code Examples

Verified patterns from official sources and existing implementation:

### Cross-Platform Native Library Detection (C#)
```csharp
// Source: bindings/csharp/MarketdataUniffi.Tests/RestClientTests.cs
[ClassInitialize]
public static void ClassInit(TestContext context)
{
    try {
        using var client = new FugleMarketData.RestClient("test-api-key");
        _nativeLibraryAvailable = true;
    }
    catch (DllNotFoundException) {
        _nativeLibraryAvailable = false;
    }
    catch (TypeInitializationException ex) when (ex.InnerException is DllNotFoundException) {
        _nativeLibraryAvailable = false;
    }
}

[TestMethod]
public void CreateRestClient_WithApiKey_Succeeds()
{
    if (!_nativeLibraryAvailable) {
        Assert.Inconclusive("Native library not available");
        return;
    }
    // Test proceeds only with native library
    using var client = new FugleMarketData.RestClient("test-api-key");
    Assert.IsNotNull(client);
}
```

### Environment-Based Test Skipping (Python)
```python
# Source: py/tests/conftest.py
import os
import pytest

def pytest_collection_modifyitems(config, items):
    """Skip integration tests if no API key is set."""
    if not os.environ.get("FUGLE_API_KEY"):
        skip_integration = pytest.mark.skip(reason="FUGLE_API_KEY not set")
        for item in items:
            if "integration" in item.keywords:
                item.add_marker(skip_integration)

# Usage in tests:
@pytest.mark.integration
class TestIntegrationRest:
    @pytest.mark.asyncio
    async def test_quote_returns_dict(self, rest_client):
        # Only runs when FUGLE_API_KEY is set
        result = await rest_client.stock.intraday.quote("2330")
        assert isinstance(result, dict)
```

### Structural API Compatibility Testing (JavaScript)
```javascript
// Source: js/tests/api-compatibility.test.js
function isPromiseLike(value) {
  return value && typeof value.then === 'function' && typeof value.catch === 'function';
}

describe('API Compatibility', () => {
  describe('RestClient structure', () => {
    let client;

    beforeAll(() => {
      client = new RestClient('test-api-key');
    });

    test('quote returns a Promise-like object', async () => {
      const result = client.stock.intraday.quote('2330');
      expect(isPromiseLike(result)).toBe(true);
      // Wait for rejection and suppress it (no API key)
      await expect(result).rejects.toThrow();
    });
  });
});
```

### VCR Fixture Recording Pattern
```python
# Source: VCR.py documentation (https://vcrpy.readthedocs.io/)
import vcr

# One-time recording from official SDK
@vcr.use_cassette('fixtures/official_sdk_quote_2330.yaml', record_mode='once')
def test_record_official_sdk_response():
    """Record response from official fugle-marketdata SDK."""
    from fugle_marketdata import RestClient as OfficialClient
    official = OfficialClient(api_key=os.environ['FUGLE_API_KEY'])
    response = official.stock.intraday.quote(symbol='2330')
    # Cassette recorded, now compare our SDK

@vcr.use_cassette('fixtures/official_sdk_quote_2330.yaml', record_mode='none')
@pytest.mark.asyncio
async def test_our_sdk_matches_official_structure():
    """Validate our SDK response matches official SDK structure."""
    from marketdata_py import RestClient as OurClient
    client = OurClient(api_key='test-key')  # Doesn't matter, using cassette
    response = await client.stock.intraday.quote('2330')

    # Structural validation against recorded official response
    assert 'symbol' in response
    assert response['symbol'] == '2330'
    assert 'date' in response
    assert 'bids' in response and isinstance(response['bids'], list)
    assert 'asks' in response and isinstance(response['asks'], list)
```

### Performance Benchmarking with Baseline
```python
# Source: pytest-benchmark pattern (https://pytest-benchmark.readthedocs.io/)
import pytest

@pytest.mark.benchmark(group="quote-latency")
def test_quote_performance(benchmark, rest_client):
    """Benchmark quote() latency."""
    # benchmark fixture automatically runs multiple iterations
    result = benchmark(lambda: rest_client.stock.intraday.quote('2330'))

    # Results tracked by github-action-benchmark in CI
    # Regression threshold: 200% default (can be customized)
    assert result is not None

# Integration with GitHub Actions:
# https://github.com/benchmark-action/github-action-benchmark
# - Auto-detects pytest-benchmark output
# - Compares to previous runs
# - Alerts on >200% regression
# - Publishes results to GitHub Pages
```

### Java Reflection-Based Structural Testing
```java
// Source: bindings/java/src/test/java/tw/com/fugle/marketdata/RestClientTest.java
import org.junit.jupiter.api.*;
import static org.junit.jupiter.api.Assertions.*;
import java.lang.reflect.Method;
import java.util.concurrent.CompletableFuture;

@Test
@DisplayName("IntradayStockClientWrapper has async methods")
void intradayStockHasAsyncMethods() throws NoSuchMethodException {
    Method getQuoteAsync = IntradayStockClientWrapper.class.getMethod("getQuoteAsync", String.class);
    assertNotNull(getQuoteAsync);
    assertEquals(CompletableFuture.class, getQuoteAsync.getReturnType());

    Method getTickerAsync = IntradayStockClientWrapper.class.getMethod("getTickerAsync", String.class);
    assertNotNull(getTickerAsync);
    assertEquals(CompletableFuture.class, getTickerAsync.getReturnType());
}

// Tests pass without native library (reflection only)
// Integration tests use @Tag("integration") and Assumptions.assumeTrue()
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Manual JSON fixtures | VCR/cassette recording | 2015+ (VCR.py stable) | Automated recording, request matching, filtering |
| Single-run benchmarks | Statistical benchmarking | pytest-benchmark 3.0 (2018) | Reliable regression detection, stddev analysis |
| Absolute performance targets | Relative baselines | Bencher launch (2024) | Cancels CI noise, detects real regressions |
| pytest-asyncio strict mode | Auto mode | 0.21.0 (2022) | Automatic async test discovery, less boilerplate |
| Manual test skipping | Collection hooks | pytest 3.0+ (2016) | Automatic skip logic, clear reporting |
| napi-rs 2.x | napi-rs 3.x | 2024 | Better memory management, Promise handling |

**Deprecated/outdated:**
- **pyo3-asyncio**: Deprecated in favor of pyo3-async-runtimes (better asyncio integration)
- **Jest --runInBand for napi memory**: Still an issue as of 2026 (https://github.com/prisma/prisma/issues/8989), use --detectLeaks to identify
- **UDL-based UniFFI**: Deprecated in favor of proc-macro approach (less duplication)
- **Manual VCR cassette format**: VCR.py now supports multiple serializers (YAML, JSON, custom)

## Open Questions

Things that couldn't be fully resolved:

1. **Performance Baseline Definition**
   - What we know: Success criteria states "within 2x for Python, 1.5x for Node.js" vs official SDKs
   - What's unclear: Need to benchmark official SDKs first to establish baseline measurements
   - Recommendation: Create baseline benchmark suite that measures official fugle-marketdata-python and @fugle/marketdata, record results as reference points, then measure our SDK against those baselines

2. **Fixture Recording Strategy for Official SDK Comparison**
   - What we know: User decided on recorded fixtures from official SDKs for compatibility tests
   - What's unclear: Best approach to record fixtures (one-time script, manual process, or automated workflow)
   - Recommendation: Create one-time recording script that uses official SDKs with real API key to capture responses, commit cassettes to version control, re-record only when API changes

3. **Rate Limit Handling in Integration Tests**
   - What we know: Integration tests will make real API calls when FUGLE_API_KEY is set
   - What's unclear: API rate limits could cause test failures if too many tests run concurrently
   - Recommendation: Mark integration tests as sequential (pytest-xdist with `--dist loadfile`), add retry logic with backoff, or use quota-aware batching based on API documentation

4. **Memory Leak Testing Automation**
   - What we know: napi-rs has memory leak detection workflow, Jest + napi has known memory issues
   - What's unclear: How to integrate napi_add_finalizer testing into our CI workflow
   - Recommendation: Add dedicated memory leak CI job similar to napi-rs pattern (https://github.com/napi-rs/napi-rs/actions/workflows/memory-test.yml), run with --detectLeaks flag, investigate heap snapshots if failures occur

5. **Cross-Language Test Specification Sharing**
   - What we know: Want consistent coverage across all 4 language bindings (Python, Node.js, C#, Java)
   - What's unclear: Whether to use shared test specification (YAML/JSON) or language-idiomatic separate test suites
   - Recommendation: Use language-idiomatic tests (better IDE support, debuggability) with documented coverage matrix to ensure parity, rather than complex test generation from shared spec

## Sources

### Primary (HIGH confidence)
- pytest-benchmark official docs: https://pytest-benchmark.readthedocs.io/
- VCR.py documentation: https://vcrpy.readthedocs.io/
- github-action-benchmark repository: https://github.com/benchmark-action/github-action-benchmark
- Bencher continuous benchmarking: https://bencher.dev/
- napi-rs memory leak workflow: https://github.com/napi-rs/napi-rs/actions/workflows/memory-test.yml
- Official SDK repositories:
  - https://github.com/fugle-dev/fugle-marketdata-python
  - https://www.npmjs.com/package/@fugle/marketdata
- Existing test implementations in this project:
  - py/tests/test_api_compatibility.py (structural tests)
  - js/tests/api-compatibility.test.js (Promise detection pattern)
  - bindings/csharp/MarketdataUniffi.Tests/RestClientTests.cs (native library detection)
  - bindings/java/src/test/java/tw/com/fugle/marketdata/RestClientTest.java (reflection-based tests)

### Secondary (MEDIUM confidence)
- [GitHub - python-cffi/cffi](https://github.com/python-cffi/cffi) - FFI testing patterns
- [Catching API regressions with snapshot testing](https://kreya.app/blog/api-snapshot-testing/) - API snapshot testing best practices
- [How to benchmark Python code with pytest-benchmark](https://bencher.dev/learn/benchmarking/python/pytest-benchmark/) - Benchmarking guide
- [State of Node.js Performance 2023](https://blog.rafaelgss.dev/state-of-nodejs-performance-2023) - Node.js performance context
- [Memory leaks in Jest when running tests with nApi](https://github.com/prisma/prisma/issues/8989) - Known Jest + napi issue
- [Snapshot Testing with Playwright in 2026](https://www.browserstack.com/guide/playwright-snapshot-testing) - Modern snapshot testing patterns

### Tertiary (LOW confidence)
- Web search results on FFI testing strategies (general patterns, not SDK-specific)
- Cross-language test consistency discussions (theoretical approaches, no standard found)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - pytest, Jest, MSTest, JUnit are de facto standards with official documentation
- Architecture: HIGH - Patterns verified in existing project tests, official documentation for VCR/pytest-benchmark
- Performance benchmarking: MEDIUM - Tools exist (github-action-benchmark, bencher) but need baseline establishment
- Fixture recording: HIGH - VCR.py and nock are mature, well-documented solutions
- FFI boundary testing: MEDIUM - General patterns documented, but SDK-specific implementation details require validation

**Research date:** 2026-01-31
**Valid until:** 60 days (testing tools are stable; API may change requiring fixture updates)
