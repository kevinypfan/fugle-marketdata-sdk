# Domain Pitfalls: Multi-Language SDK Development

**Domain:** Rust-core multi-language SDK with Python (PyO3), Node.js (napi-rs), and C# (UniFFI) bindings
**Researched:** 2026-01-30

## Critical Pitfalls

Mistakes that cause rewrites, major runtime failures, or production incidents.

### Pitfall 1: Panic Unwinding Across FFI Boundaries
**What goes wrong:** Rust panics escape across FFI boundaries causing undefined behavior, process crashes, or corrupted state in the host language runtime.

**Why it happens:**
- Rust's default `extern "C"` ABI treats unwinding as undefined behavior
- Many operations (array indexing, unwrap(), arithmetic overflow) can panic
- PyO3/napi-rs catch panics at their boundaries, but custom FFI code may not
- Developers assume panics work like exceptions in Python/JavaScript

**Consequences:**
- Process termination without cleanup (file handles, network connections leak)
- Corrupted Python/Node.js runtime state leading to segfaults in unrelated code
- Production incidents that are impossible to debug (no stack trace survives FFI boundary)

**Prevention:**
```rust
// ❌ Wrong: panic can escape
#[no_mangle]
pub extern "C" fn get_quote(symbol: *const c_char) -> *mut Quote {
    let symbol = unsafe { CStr::from_ptr(symbol) };
    data.get(symbol).unwrap()  // Can panic!
}

// ✅ Right: catch all panics
#[no_mangle]
pub extern "C" fn get_quote(symbol: *const c_char) -> *mut Result<Quote> {
    std::panic::catch_unwind(|| {
        let symbol = unsafe { CStr::from_ptr(symbol) };
        data.get(symbol).ok_or(Error::NotFound)
    }).unwrap_or_else(|_| Err(Error::Panic))
}
```

- Use `std::panic::catch_unwind()` at ALL FFI entry points
- Return Result types and convert panics to error codes
- Enable `panic = "abort"` in release builds for predictable failure modes
- PyO3 and napi-rs handle this automatically for `#[pyfunction]` and `#[napi]` macros

**Detection:**
- Unexplained crashes with no stack trace
- Intermittent segfaults in Python/Node.js code unrelated to SDK calls
- Process exits with status 134 (SIGABRT) or 139 (SIGSEGV)
- Valgrind/AddressSanitizer shows "invalid pointer" after SDK calls

**Phase mapping:** Phase 1 (Python compatibility) - ensure existing PyO3 code has panic handling; Phase 2 (Node.js) - audit napi-rs FFI boundaries; Phase 3 (C#) - critical for UniFFI where automatic panic handling may be limited

**Sources:**
- [Rust FFI unwinding RFC](https://rust-lang.github.io/rfcs/2797-project-ffi-unwind.html)
- [FFI Best Practices 2025](https://medium.com/@QuarkAndCode/ffi-best-practices-for-rust-deno-mojo-5b9950dde5ce)

---

### Pitfall 2: Async Runtime Deadlocks with GIL and Event Loops
**What goes wrong:** Python GIL acquisition deadlocks with Tokio runtime, Node.js event loop blocks waiting for Rust async operations, or multiple async runtimes conflict.

**Why it happens:**
- PyO3: `Python::with_gil()` blocks the thread, but Tokio may hold locks needed by Python
- napi-rs: Node.js event loop is single-threaded; blocking it stalls all JavaScript
- Tokio spawns tasks that try to re-acquire GIL from different threads
- Async Rust expects `await` points for preemption; FFI synchronous boundaries don't provide this

**Consequences:**
- Application hangs indefinitely (hard to debug - no error, just frozen)
- WebSocket connections timeout because event loop is blocked
- Python asyncio and Tokio compete for CPU, causing starvation
- Multi-threaded Tokio tries to acquire GIL from worker thread → deadlock

**Prevention:**

**For PyO3:**
```rust
// ❌ Wrong: GIL held while awaiting Rust future
#[pyfunction]
fn fetch_quote(py: Python, symbol: String) -> PyResult<Quote> {
    py.allow_threads(|| {  // STILL WRONG - blocks thread
        runtime.block_on(async_fetch(symbol))
    })
}

// ✅ Right: Release GIL, return Python future
#[pyfunction]
fn fetch_quote(py: Python, symbol: String) -> PyResult<&PyAny> {
    pyo3_asyncio::tokio::future_into_py(py, async move {
        let result = async_fetch(symbol).await;
        Python::with_gil(|py| result.into_py(py))
    })
}
```

**For napi-rs:**
```rust
// ❌ Wrong: blocking Node.js event loop
#[napi]
fn fetch_quote(symbol: String) -> Result<Quote> {
    RUNTIME.block_on(async_fetch(symbol))  // Blocks Node!
}

// ✅ Right: return promise to Node.js
#[napi]
async fn fetch_quote(symbol: String) -> Result<Quote> {
    async_fetch(symbol).await  // napi-rs handles conversion
}
```

**Key strategies:**
- Use `pyo3-async-runtimes` for Python - bridges Python asyncio and Tokio
- Always use `async fn` in napi-rs for async operations
- NEVER call `block_on()` from FFI code
- Release GIL before ANY blocking operation with `py.allow_threads()`
- Use `#[pyo3(get)]` and `#[pyo3(set)]` for sync properties only

**Detection:**
- `top` shows 100% CPU but no progress
- `py-spy` or Node.js profiler shows single function consuming all time
- Python threads show "waiting for GIL" in `py-spy dump`
- strace shows futex wait operations indefinitely
- Application becomes unresponsive to signals (except SIGKILL)

**Phase mapping:** Phase 1 (Python) - critical for WebSocket streaming which uses Tokio heavily; Phase 2 (Node.js) - critical, Node.js is single-threaded; Phase 3 (C#) - less critical, .NET has better multi-threading

**Sources:**
- [PyO3 GIL deadlock discussions](https://github.com/PyO3/pyo3/discussions/3045)
- [napi-rs async fn documentation](https://napi.rs/docs/concepts/async-fn)
- [pyo3-async-runtimes](https://github.com/PyO3/pyo3-async-runtimes)

---

### Pitfall 3: Breaking API Compatibility in Subtle Ways
**What goes wrong:** SDK claims API compatibility but breaks it through type mismatches, changed error behaviors, different default values, or inconsistent async/sync signatures.

**Why it happens:**
- Type mapping across languages loses precision (JavaScript number → Rust i64 → overflow)
- Error handling differs (Python exceptions vs Rust Results vs JavaScript thrown errors)
- Defaults not explicitly specified in one language
- Async/sync mismatches: official SDK is sync, Rust binding exposes async
- Serialization format differences (JSON field names, null vs undefined, ISO date formats)

**Consequences:**
- Users' existing code breaks when switching from official SDK
- Silent data corruption (price 1234567890123 becomes NaN in JavaScript)
- Error handling code doesn't catch expected exceptions
- Tests pass but production breaks due to edge cases
- Loss of trust in "drop-in replacement" promise

**Prevention:**

**Method signature compatibility matrix:**
```python
# Official Python SDK
def get_quote(symbol: str, oddLot: bool = False) -> Quote

# ❌ Wrong: different parameter name
def get_quote(symbol: str, odd_lot: bool = False) -> Quote

# ✅ Right: match exactly (PyO3 rename)
#[pyfunction]
#[pyo3(signature = (symbol, oddLot=false))]
fn get_quote(symbol: String, odd_lot: bool) -> PyResult<Quote>
```

**Type precision handling:**
```javascript
// Official Node SDK returns number for price
// JavaScript number is f64, can't represent all i64

// ❌ Wrong: precision loss
class Quote {
  get price(): number { return this.inner.price; }
}

// ✅ Right: use string for large integers
class Quote {
  get price(): string { return this.inner.price.toString(); }
  get priceAsNumber(): number { /* document precision loss */ }
}
```

**Error classification parity:**
```rust
// ❌ Wrong: different error types than original SDK
match status {
    401 => Err(Error::Auth),
    429 => Err(Error::Network),  // Original SDK: RateLimitError
}

// ✅ Right: match original error hierarchy
match status {
    401 => Err(Error::Auth(AuthError::InvalidToken)),
    429 => Err(Error::RateLimit(RateLimitError::TooManyRequests)),
}
```

**Strategies:**
- Generate compatibility test suite from official SDK tests
- Use property-based testing to find edge cases (QuickCheck, Hypothesis)
- Test with real API responses captured from production
- Document EVERY deviation from official SDK (even if "better")
- Version bindings independently - don't break compatibility in patch releases

**Detection:**
- User reports "code worked with official SDK"
- Type errors in production that don't appear in tests
- `JSON.parse` failures or invalid date parsing
- Tests use mocks that don't match real API behavior
- Integration tests pass but manual testing breaks

**Phase mapping:** Phase 1 (Python) - compare against fugle-marketdata-python test suite; Phase 2 (Node.js) - compare against fugle-marketdata-node; Phase 3 (C#) - compare against FubonNeo patterns; All phases need compatibility matrix testing

**Sources:**
- [Android SDK compatibility issues research](https://www.revenuecat.com/blog/engineering/binary-compatability/)
- [API compatibility testing best practices](https://blog.dreamfactory.com/how-to-ensure-api-compatibility-across-platforms)

---

## Moderate Pitfalls

Mistakes that cause delays, technical debt, or require significant rework.

### Pitfall 4: Cross-Platform Binary Distribution Failures
**What goes wrong:** Wheels/packages build on developer machine but fail on user machines due to glibc version mismatches, missing system dependencies, or wrong platform tags.

**Why it happens:**
- manylinux compliance: building on Ubuntu 24.04 (glibc 2.39) produces wheels that don't work on CentOS 7 (glibc 2.17)
- Native dependencies: OpenSSL, native-tls link to system libraries
- Platform tags: wheel tagged `linux_x86_64` instead of `manylinux2014_x86_64` rejected by PyPI
- macOS universal2 binaries: arm64 + x86_64 not built correctly
- Windows: MSVC vs GNU toolchain mismatch

**Consequences:**
- PyPI rejects wheel uploads: "linux platform tag not allowed"
- Users get "symbol not found" or "GLIBC_2.XX not found" at import time
- macOS M1 users forced to use Rosetta (slow)
- Windows users get "DLL load failed" errors
- CI builds succeed but releases fail

**Prevention:**

**For Python (maturin):**
```bash
# ❌ Wrong: uses host glibc
maturin build --release

# ✅ Right: use manylinux container
docker run --rm -v $(pwd):/io \
  quay.io/pypa/manylinux2014_x86_64 \
  /io/build-wheels.sh

# ✅ Better: use cibuildwheel (automates cross-platform)
pip install cibuildwheel
cibuildwheel --platform linux
```

**Cargo.toml configuration:**
```toml
[package.metadata.maturin]
# Ensure PyPI compatibility
compatibility = "manylinux2014"
# Or be explicit about minimum glibc
# compatibility = "manylinux_2_17"

[profile.release]
# Strip symbols to reduce size
strip = true
```

**For Node.js (napi-rs):**
```bash
# ❌ Wrong: only builds for host platform
napi build --release

# ✅ Right: cross-compile for all targets
napi build --release --target x86_64-unknown-linux-gnu
napi build --release --target aarch64-unknown-linux-gnu
napi build --release --target x86_64-apple-darwin
napi build --release --target aarch64-apple-darwin
```

**Platform support matrix:**
| Platform | Python | Node.js | C# |
|----------|--------|---------|-----|
| Linux x64 | manylinux2014 | glibc 2.17+ | .NET 6+ |
| Linux ARM64 | manylinux2014 | glibc 2.17+ | .NET 6+ |
| macOS x64 | 10.12+ | 10.13+ | .NET 6+ |
| macOS ARM64 | 11.0+ | 11.0+ | .NET 6+ |
| Windows x64 | 7+ | 10+ | .NET 6+ |

**Strategies:**
- Use official build containers (manylinux, quay.io/pypa)
- Test on minimal Docker images (Alpine, debian:slim)
- Check `auditwheel` / `delocate` output for external dependencies
- Use `--compatibility pypi` flag for maturin
- Set up CI matrix for all platforms (GitHub Actions, cross-rs)
- Document minimum OS versions in README

**Detection:**
- `pip install` works on dev machine, fails on server
- `ldd` shows missing shared libraries
- ImportError: dynamic module does not define init function
- macOS: "is damaged and can't be opened" (code signing)
- PyPI upload fails with platform tag rejection

**Phase mapping:** Phase 1 (Python) - critical, set up manylinux builds; Phase 2 (Node.js) - critical, set up cross-compilation; Phase 3 (C#) - moderate, .NET has better cross-platform story but still needs testing

**Sources:**
- [Maturin distribution guide](https://www.maturin.rs/distribution.html)
- [cibuildwheel documentation](https://cibuildwheel.pypa.io/)
- [manylinux PEP 513](https://peps.python.org/pep-0513/)
- [musllinux PEP 656](https://peps.python.org/pep-0656/)

---

### Pitfall 5: Memory Safety Violations at FFI Boundaries
**What goes wrong:** Use-after-free, double-free, dangling pointers, or memory leaks when objects cross FFI boundaries.

**Why it happens:**
- Ownership confusion: who frees the memory? (Rust, Python GC, Node.js V8, or C# GC?)
- Lifetime mismatches: Rust reference outlives Python object it points to
- Buffer ownership: passing Vec/String to FFI without proper handoff
- Circular references between Rust and host language prevent GC
- Async drop: objects dropped on wrong thread (not thread-safe)

**Consequences:**
- Segfaults in production (often non-deterministic)
- Memory leaks that grow unbounded
- Data races when shared between threads
- Corrupt data visible to users (wrong prices, wrong symbols)

**Prevention:**

**For PyO3:**
```rust
// ❌ Wrong: reference escapes PyO3 boundary
#[pyclass]
struct Client {
    config: &'static Config  // Dangerous!
}

// ✅ Right: own the data or use Arc
#[pyclass]
struct Client {
    config: Arc<Config>  // Shared ownership
}

// ❌ Wrong: mutable reference in async
#[pymethods]
impl Client {
    fn __aiter__(&mut self) -> PyResult<MessageIterator> {
        // &mut self in async is unsound!
    }
}

// ✅ Right: use interior mutability
#[pyclass]
struct Client {
    inner: Arc<Mutex<ClientInner>>
}
```

**For napi-rs:**
```rust
// ❌ Wrong: lifetime violation
#[napi]
struct Buffer {
    data: &'static [u8]  // Can't guarantee 'static
}

// ✅ Right: owned types with proper GC integration
#[napi]
impl Buffer {
    #[napi]
    pub fn from_vec(data: Vec<u8>) -> Self {
        // napi-rs handles buffer ownership correctly
        Buffer { inner: data.into() }
    }
}

// Buffer/TypedArray are reference-counted
// Safe to use across async boundaries
```

**For UniFFI:**
```rust
// UniFFI handles ownership automatically but:
// ❌ Wrong: mutable state in Arc<T>
pub fn set_config(client: Arc<Client>, config: Config) {
    client.config = config;  // Can't mutate Arc contents!
}

// ✅ Right: use Arc<Mutex<T>> or Arc<RwLock<T>>
pub fn set_config(client: Arc<Mutex<Client>>, config: Config) {
    client.lock().unwrap().config = config;
}
```

**Key rules:**
- NEVER use raw pointers (*const, *mut) unless absolutely necessary
- Use Arc for shared ownership across FFI boundaries
- Use Mutex/RwLock for mutable shared state
- Document ownership transfer with `#[must_use]` or explicit `take_ownership()`
- Check for circular references (Python object holds Rust, Rust holds Python)

**Detection:**
- Valgrind: "Invalid read/write" or "Use after free"
- AddressSanitizer (ASan): heap-use-after-free
- Memory profilers show unbounded growth
- Segfaults on GC collection cycles
- Intermittent crashes under load (threading issues)

**Phase mapping:** Phase 1 (Python) - PyO3 handles much of this, audit custom FFI; Phase 2 (Node.js) - critical with napi-rs Buffer/TypedArray usage; Phase 3 (C#) - UniFFI generates safe bindings but check manual FFI code

**Sources:**
- [napi-rs lifetime documentation](https://napi.rs/docs/concepts/understanding-lifetime)
- [PyO3 FAQ on memory management](https://pyo3.rs/main/faq)

---

### Pitfall 6: Type Conversion Edge Cases
**What goes wrong:** Data loses precision, gets truncated, or converts incorrectly when crossing language boundaries (especially numbers, dates, nulls).

**Why it happens:**
- JavaScript number is always f64 - can't represent i64 precisely
- Python int is arbitrary precision, Rust integers overflow
- null vs undefined vs None vs null (4 different "nothing" concepts)
- Timezone handling: naive vs aware datetimes, UTC vs local
- String encoding: UTF-8, UTF-16, UCS-2 differences

**Consequences:**
- Price 9007199254740993 becomes 9007199254740992 in JavaScript (precision loss)
- Large order IDs get corrupted
- Timestamps off by hours due to timezone assumptions
- null vs undefined causes type errors
- Emoji/unicode in symbols get corrupted

**Prevention:**

**Integer precision:**
```typescript
// JavaScript
// ❌ Wrong: precision loss for large integers
interface Quote {
  orderId: number;  // Can't safely represent > 2^53
}

// ✅ Right: use string for large integers
interface Quote {
  orderId: string;  // Safe for any size
  orderIdNum: number;  // Convenience getter with warning
}
```

```rust
// Rust side
#[napi(object)]
pub struct Quote {
    pub order_id: String,  // Serialize as string
}
```

**Null handling:**
```python
# Python
# ❌ Wrong: None vs null confusion
def get_quote(symbol: str) -> Optional[Quote]:
    return None  # Python None

# JavaScript sees: undefined or null?
# PyO3 converts None to Python None → JavaScript null
# But JavaScript undefined ≠ null!

# ✅ Right: explicit optional handling
#[pyfunction]
fn get_quote(symbol: String) -> PyResult<Option<Quote>> {
    Ok(Some(quote))  // or Ok(None)
}
// PyO3: Some(x) → x, None → None
// napi-rs: Some(x) → x, None → null (NOT undefined)
```

**DateTime handling:**
```rust
// ❌ Wrong: naive datetime ambiguity
pub struct Quote {
    pub timestamp: NaiveDateTime  // What timezone?
}

// ✅ Right: always UTC with explicit timezone
pub struct Quote {
    pub timestamp: DateTime<Utc>
}

// Python: converts to datetime.datetime with tzinfo=UTC
// JavaScript: converts to Date (always UTC internally)
// C#: converts to DateTimeOffset with UTC offset
```

**Strategies:**
- Use strings for integers > 2^53 in JavaScript bindings
- Document timezone assumptions (prefer UTC everywhere)
- Test with boundary values: i64::MAX, f64::INFINITY, empty strings
- Use property-based testing for edge cases (Hypothesis, fast-check)
- Explicit null handling: Option<T> in Rust, Optional in Python, nullable in TypeScript

**Detection:**
- User reports "wrong order ID" or "price is off"
- Tests with large numbers (> 10^15) fail
- Timezone-related bugs (off by N hours)
- Type errors: "undefined is not a function" in JavaScript
- JSON serialization round-trip fails

**Phase mapping:** Phase 1 (Python) - less critical (Python handles big ints); Phase 2 (Node.js) - critical (JavaScript number precision); Phase 3 (C#) - moderate (C# has decimal type but still watch for overflow)

---

### Pitfall 7: Test Coverage Gaps for Edge Cases
**What goes wrong:** Unit tests pass but production breaks due to untested edge cases: network errors, rate limiting, reconnection storms, concurrent access, etc.

**Why it happens:**
- Tests use mocks that don't match real API behavior
- Network errors hard to simulate in unit tests
- Concurrent access tests require complex setup
- Developers test "happy path" only
- Integration tests expensive/slow, so they're skipped

**Consequences:**
- Rate limiting causes infinite retry loops
- Reconnection storms during network flapping
- Race conditions in concurrent subscription management
- Panics on malformed API responses
- Memory leaks under high load

**Prevention:**

**Test categories needed:**

**1. API compatibility tests:**
```python
# Generate from official SDK test suite
def test_compatibility_with_official_sdk():
    official_result = official_sdk.get_quote("2330")
    our_result = our_sdk.get_quote("2330")

    # Compare structure, not values (API changes)
    assert type(official_result) == type(our_result)
    assert official_result.keys() == our_result.keys()
    # Test edge cases official SDK handles
```

**2. Error handling tests:**
```rust
#[test]
fn test_rate_limit_backoff() {
    let mock_server = mock_api_with_rate_limit();
    let client = RestClient::new();

    // Should NOT infinite loop
    let result = client.get_quote("2330");
    assert!(matches!(result, Err(Error::RateLimit(_))));

    // Should respect retry-after header
    assert!(mock_server.request_count() <= MAX_RETRIES);
}
```

**3. Concurrency tests:**
```rust
#[tokio::test]
async fn test_concurrent_subscriptions() {
    let client = WebSocketClient::new().await;

    // Multiple threads subscribing simultaneously
    let handles: Vec<_> = (0..10).map(|i| {
        let client = client.clone();
        tokio::spawn(async move {
            client.subscribe(format!("symbol_{}", i)).await
        })
    }).collect();

    // Should not deadlock or lose subscriptions
    for handle in handles {
        handle.await.unwrap().unwrap();
    }
}
```

**4. Network instability tests:**
```rust
#[test]
fn test_reconnection_under_flapping() {
    let mock = mock_server_that_flaps_every_5_seconds();
    let client = WebSocketClient::connect(&mock.url()).await;

    // Should handle frequent disconnects gracefully
    sleep(Duration::from_secs(60)).await;

    // Should NOT be in reconnection storm
    assert!(client.reconnection_count() < 20);
}
```

**5. Boundary value tests:**
```rust
#[test]
fn test_large_integers_in_javascript() {
    let quote = Quote {
        order_id: i64::MAX,  // 9223372036854775807
        price: 1234567890123.45
    };

    let js_value = quote.to_js();
    let round_trip = Quote::from_js(js_value);

    // JavaScript can't represent i64::MAX
    // Should use string representation
    assert_eq!(round_trip.order_id, quote.order_id);
}
```

**Strategies:**
- Use chaos engineering: inject random failures
- Capture real API traffic and replay in tests
- Set up CI/CD matrix: all platforms, all Python/Node versions
- Use code coverage tools (cargo-tllvm-cov) and aim for >80%
- Integration test against real API (staging environment)

**Detection:**
- Production incidents that tests didn't catch
- Code coverage report shows untested branches
- User reports "this worked before" (regression)
- Crash reports from error tracking (Sentry, Rollbar)

**Phase mapping:** All phases - set up testing infrastructure early; Phase 1 (Python) - test against official SDK test suite; Phase 2 (Node.js) - add concurrent access tests; Phase 3 (C#) - add .NET-specific threading tests

**Sources:**
- [Property-based testing best practices](https://hypothesis.readthedocs.io/)
- [Chaos engineering for APIs](https://principlesofchaos.org/)

---

## Minor Pitfalls

Mistakes that cause annoyance but are fixable without major rewrites.

### Pitfall 8: Documentation Mismatches Between Languages
**What goes wrong:** Documentation copy-pasted between languages doesn't reflect language-specific APIs, idioms, or error handling.

**Why it happens:**
- Doc comments generated from Rust, don't match Python/JavaScript conventions
- Examples show Rust syntax when users want Python/JavaScript
- Error types documented for Rust don't match Python exceptions
- Idioms differ: Python snake_case vs JavaScript camelCase

**Prevention:**
- Write language-specific docstrings in binding code
- Generate API docs per language (pydoc, JSDoc, XML docs for C#)
- Include examples in each language
- Document error mapping: Rust `Error::RateLimit` → Python `RateLimitError`

**Detection:**
- User confusion in issues: "documentation doesn't match API"
- Examples don't run when copy-pasted
- Type hints in Python don't match actual types

**Phase mapping:** All phases - write docs as bindings are created

---

### Pitfall 9: Version Skew Between Core and Bindings
**What goes wrong:** Rust core updated but bindings not rebuilt, causing ABI incompatibility or missing features.

**Why it happens:**
- Bindings built separately from core
- No version checks at FFI boundary
- Cached build artifacts from old version
- CI builds core and bindings in separate jobs

**Prevention:**
- Use `build.rs` to embed version info and check at runtime
- Monorepo with workspace dependencies
- CI builds all bindings together
- Version pinning: `core = "=1.2.3"` (exact version)

```rust
// core/build.rs
fn main() {
    println!("cargo:rustc-env=CORE_VERSION={}", env!("CARGO_PKG_VERSION"));
}

// bindings/src/lib.rs
const EXPECTED_CORE_VERSION: &str = "1.2.3";
const ACTUAL_CORE_VERSION: &str = env!("CORE_VERSION");

#[ctor::ctor]
fn check_version() {
    assert_eq!(EXPECTED_CORE_VERSION, ACTUAL_CORE_VERSION,
        "Core version mismatch!");
}
```

**Detection:**
- Crashes with "symbol not found"
- Features don't work despite being in core
- CI builds succeed but releases fail

**Phase mapping:** Phase 1 - set up version checking early

---

### Pitfall 10: Build Time Optimization Neglected
**What goes wrong:** Clean builds take 10+ minutes, slowing development iteration and CI costs.

**Why it happens:**
- No caching of dependencies (each build from scratch)
- Release builds in development (unnecessary optimization)
- No incremental compilation
- Large dependency trees (tokio, serde, etc.)

**Prevention:**
- Use sccache or cargo-chef for dependency caching
- Dev builds use `--profile dev`, only release for publishing
- Share target/ directory between workspaces
- Split into smaller crates to enable parallel compilation

**Detection:**
- `cargo build` takes >5 minutes on developer machine
- CI builds timeout or cost too much
- Developers avoid rebuilding

**Phase mapping:** Phase 1 - optimize Rust build first; Phase 2/3 - optimize per-language build (maturin, napi-rs)

---

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation |
|-------------|---------------|------------|
| Python API compatibility | Method signature differences (snake_case vs camelCase, parameter names) | Generate compatibility matrix from official SDK, test side-by-side |
| Python async integration | GIL deadlocks with Tokio runtime | Use pyo3-asyncio, never block_on() from Python thread |
| Python packaging | manylinux compatibility failures on older distributions | Use manylinux2014 containers for builds, test on CentOS 7 / Debian 9 |
| Node.js API compatibility | Type precision loss (JavaScript number for i64) | Use string for large integers, document precision limits |
| Node.js async integration | Blocking Node.js event loop with sync Rust calls | Always use `async fn` for any potentially blocking operation |
| Node.js packaging | Missing prebuilt binaries for obscure platforms (armv7, Alpine Linux) | Set up cross-compilation for common targets, document build from source |
| C# bindings creation | UniFFI learning curve and C# code generation quirks | Start with simple types, test generated C# code early, iterate |
| C# async integration | Task vs async/await mismatch with Rust futures | Use UniFFI's async support (if available) or bridge with TaskCompletionSource |
| Cross-language testing | API behavior divergence not caught until production | Set up integration test suite that runs against all language bindings simultaneously |
| Distribution | Platform-specific failures (macOS code signing, Windows DLL dependencies) | Test on all platforms in CI, use official build containers |

---

## Confidence Assessment

| Pitfall Category | Confidence | Source Quality |
|------------------|------------|----------------|
| PyO3 pitfalls | HIGH | Official PyO3 docs, GitHub issues with 2025 updates |
| napi-rs pitfalls | HIGH | Official napi-rs docs, recent blog posts |
| UniFFI pitfalls | MEDIUM | Third-party C# bindings docs (uniffi-bindgen-cs), general UniFFI docs |
| Async runtime conflicts | HIGH | Multiple sources (PyO3 discussions, napi-rs docs), recent 2025 issues |
| API compatibility testing | MEDIUM | General SDK best practices, not Rust-specific |
| Cross-platform distribution | HIGH | Official Python PEPs, maturin docs, recent 2025 updates |
| Memory safety at FFI | HIGH | Rust FFI RFCs, language-specific binding docs |
| Type conversion edge cases | HIGH | Well-documented in all binding frameworks |

---

## Sources

**PyO3 / Python Bindings:**
- [PyO3 FAQ and Troubleshooting](https://pyo3.rs/main/faq)
- [PyO3 Error Handling](https://pyo3.rs/main/function/error-handling.html)
- [PyO3 GIL Deadlock Discussion](https://github.com/PyO3/pyo3/discussions/3045)
- [PyO3 Async GIL Issues](https://github.com/PyO3/pyo3/discussions/1912)
- [Maturin Distribution Guide](https://www.maturin.rs/distribution.html)
- [Building Portable Python Extensions](https://blog.savant-ai.io/building-portable-native-python-extensions-with-rust-pyo3-and-maturin-3c1a1634d324)
- [pyo3-async-runtimes](https://github.com/PyO3/pyo3-async-runtimes)

**napi-rs / Node.js Bindings:**
- [napi-rs Documentation](https://napi.rs/)
- [napi-rs Async Functions](https://napi.rs/docs/concepts/async-fn)
- [napi-rs Understanding Lifetime](https://napi.rs/docs/concepts/understanding-lifetime)
- [napi-rs TypedArray Documentation](https://napi.rs/docs/concepts/typed-array)
- [Announcing NAPI-RS v2](https://napi.rs/blog/announce-v2)
- [Node.js Native Addons with N-API](https://medium.com/@2nick2patel2/node-js-native-addons-with-n-api-safely-wrapping-rust-c-for-hot-paths-7015cbbcd7b5)

**UniFFI / C# Bindings:**
- [UniFFI GitHub](https://github.com/mozilla/uniffi-rs)
- [uniffi-bindgen-cs (C# bindings)](https://github.com/NordSecurity/uniffi-bindgen-cs)
- [UniFFI Design Principles](https://mozilla.github.io/uniffi-rs/latest/internals/design_principles.html)
- [UniFFI Blog Post](https://blog.mozilla.org/data/2020/10/21/this-week-in-glean-cross-platform-language-binding-generation-with-rust-and-uniffi/)

**Rust FFI and Error Handling:**
- [Rust FFI Unwind RFC](https://rust-lang.github.io/rfcs/2797-project-ffi-unwind.html)
- [Rust C-unwind ABI RFC](https://rust-lang.github.io/rfcs/2945-c-unwind-abi.html)
- [FFI Best Practices for Rust 2025](https://medium.com/@QuarkAndCode/ffi-best-practices-for-rust-deno-mojo-5b9950dde5ce)
- [Rust FFI Documentation](https://doc.rust-lang.org/nomicon/ffi.html)

**Cross-Platform Distribution:**
- [manylinux GitHub](https://github.com/pypa/manylinux)
- [manylinux PEP 513](https://peps.python.org/pep-0513/)
- [manylinux_x_y PEP 600](https://peps.python.org/pep-0600/)
- [musllinux PEP 656](https://peps.python.org/pep-0656/)
- [cibuildwheel](https://cibuildwheel.pypa.io/)
- [Building Cross-Platform SDKs: From FFI to WebAssembly](https://blog.flipt.io/from-ffi-to-wasm)

**API Compatibility and Testing:**
- [Android API Compatibility Issues](https://www.revenuecat.com/blog/engineering/binary-compatability/)
- [How to Ensure API Compatibility Across Platforms](https://blog.dreamfactory.com/how-to-ensure-api-compatibility-across-platforms)
- [Semantic Versioning](https://semver.org/)
- [Beyond API Compatibility: Breaking Changes](https://www.infoq.com/articles/breaking-changes-are-broken-semver/)
- [Best SDK Generation Tools 2025](https://buildwithfern.com/post/best-sdk-generation-tools-multi-language-api)

**Async Runtime and Concurrency:**
- [Tokio Documentation](https://tokio.rs/)
- [The State of Async Rust: Runtimes](https://corrode.dev/blog/async/)
- [Async Programming in Rust](https://rust-lang.github.io/async-book/)

---

*Research completed: 2026-01-30*
*Confidence: HIGH for Python/Node.js, MEDIUM for C# (less mature ecosystem)*
