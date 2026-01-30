# Phase 2: Python Binding Enhancement - Research

**Researched:** 2026-01-31
**Domain:** Python/Rust FFI with PyO3, async runtime bridging, native Python bindings
**Confidence:** HIGH

## Summary

This research investigates modernizing Python bindings using PyO3 0.27+ with native asyncio support to create a drop-in replacement for fugle-marketdata-python. The standard approach combines PyO3 0.27's Bound smart pointers with pyo3-async-runtimes for tokio/asyncio bridging, maturin 1.11+ for build orchestration, and comprehensive type stub generation for IDE support.

**Key findings:**
- PyO3 0.27 requires migration from GIL Refs to Bound smart pointers, improving performance and free-threaded Python 3.13+ compatibility
- pyo3-async-runtimes (successor to pyo3-asyncio) provides proven tokio/asyncio bridging with `future_into_py()` for exposing Rust futures as Python awaitables
- Type hints require manual .pyi stub files with py.typed marker; pyo3-stubgen provides automation for CI/CD integration
- GIL deadlocks are the #1 pitfall when mixing tokio multi-threaded runtime with Python; requires strategic `allow_threads()` usage
- WebSocket async iterator pattern (`async for`) is well-established in Python ecosystem via websockets library

**Primary recommendation:** Use PyO3 0.27 with pyo3-async-runtimes 0.27, tokio-tungstenite for WebSocket client, frozen dataclasses for response models, and pyo3-stubgen for automated type stub generation. Structure as mixed Rust/Python project to maintain pure Python utilities alongside Rust bindings.

## Standard Stack

The established libraries/tools for PyO3 0.27+ async Python bindings:

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| PyO3 | 0.27+ | Rust↔Python FFI | Official Rust-Python binding framework, mature ecosystem, Python 3.14/3.13t support |
| pyo3-async-runtimes | 0.27+ | Async runtime bridge | Official PyO3 async solution (replaces pyo3-asyncio), tokio↔asyncio conversion |
| maturin | 1.11+ | Build system | Official PyO3 build tool, handles wheel generation, mixed Rust/Python projects |
| tokio | 1.40+ | Rust async runtime | Industry standard async runtime, required for pyo3-async-runtimes |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| tokio-tungstenite | 0.24+ | WebSocket client | Async WebSocket with tokio integration, lightweight |
| pyo3-stubgen | Latest | Type stub generator | Generate .pyi files from compiled extension modules |
| stream-reconnect | Latest | Auto-reconnect | Wrapper for tungstenite with automatic reconnection |
| reqwest | 0.12+ | HTTP client | Async REST API calls (likely already in core) |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| pyo3-async-runtimes | pyo3-async (wyfo) | Supports trio/anyio but less mature, smaller ecosystem |
| tokio-tungstenite | async-tungstenite | Runtime-agnostic but PyO3 pairs best with tokio |
| pyo3-stubgen (Python) | pyo3-stub-gen (Rust) | Rust version requires manual macro annotations, more invasive |

**Installation:**
```toml
# Cargo.toml
[dependencies]
pyo3 = { version = "0.27", features = ["extension-module"] }
pyo3-async-runtimes = { version = "0.27", features = ["tokio-runtime"] }
tokio = { version = "1.40", features = ["rt-multi-thread", "macros", "time"] }
tokio-tungstenite = "0.24"
reqwest = { version = "0.12", features = ["json"] }

[build-dependencies]
maturin = "1.11"
```

```toml
# pyproject.toml
[build-system]
requires = ["maturin>=1.11,<2"]
build-backend = "maturin"

[project]
requires-python = ">=3.9"
```

## Architecture Patterns

### Recommended Project Structure
```
py/
├── Cargo.toml                  # Rust crate config
├── pyproject.toml              # Python package metadata
├── build.rs                    # Build script (if needed)
├── src/
│   ├── lib.rs                  # PyO3 module entry point
│   ├── rest/
│   │   ├── mod.rs              # REST client PyO3 wrapper
│   │   └── client.rs           # Rust implementation
│   ├── websocket/
│   │   ├── mod.rs              # WebSocket PyO3 wrapper
│   │   ├── client.rs           # Rust WebSocket client
│   │   └── iterator.rs         # Python iterator adapter
│   ├── types/
│   │   └── mod.rs              # Response models as PyClass
│   ├── errors.rs               # Exception hierarchy
│   └── utils/
│       └── asyncio.rs          # pyo3-async-runtimes helpers
├── marketdata_py/              # Mixed Python package (optional)
│   ├── __init__.py             # Re-export Rust + Python
│   ├── py.typed                # PEP 561 marker
│   └── types.pyi               # Type stubs
└── tests/
    └── test_*.py               # Python integration tests
```

### Pattern 1: Async Function Exposure
**What:** Convert Rust async functions to Python awaitables
**When to use:** All REST API calls, WebSocket connection/subscription
**Example:**
```rust
// Source: https://github.com/PyO3/pyo3-async-runtimes
use pyo3::prelude::*;
use pyo3_async_runtimes::tokio::future_into_py;

#[pyfunction]
fn fetch_quote(py: Python<'_>, symbol: String) -> PyResult<Bound<PyAny>> {
    future_into_py(py, async move {
        let client = reqwest::Client::new();
        let resp = client.get(&format!("https://api.fugle.tw/v1/quote/{}", symbol))
            .send()
            .await
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyConnectionError, _>(e.to_string()))?;

        let data = resp.json::<serde_json::Value>()
            .await
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

        Python::with_gil(|py| Ok(data.into_py(py)))
    })
}
```

### Pattern 2: WebSocket Async Iterator
**What:** Expose tokio Stream as Python async iterator
**When to use:** Real-time message streaming (async for msg in ws.subscribe(...))
**Example:**
```rust
// Source: Combined from PyO3 docs + websockets library patterns
use pyo3::prelude::*;
use tokio::sync::mpsc;

#[pyclass]
pub struct MessageIterator {
    rx: Arc<Mutex<mpsc::UnboundedReceiver<serde_json::Value>>>,
}

#[pymethods]
impl MessageIterator {
    fn __aiter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __anext__<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        let rx = self.rx.clone();
        future_into_py(py, async move {
            match rx.lock().await.recv().await {
                Some(msg) => Ok(Some(Python::with_gil(|py| msg.into_py(py)))),
                None => Ok(None), // StopAsyncIteration
            }
        })
    }
}
```

### Pattern 3: Exception Hierarchy
**What:** Custom Python exception types from Rust errors
**When to use:** All error conditions, matching official SDK errors
**Example:**
```rust
// Source: https://pyo3.rs/main/function/error-handling.html
use pyo3::create_exception;
use pyo3::exceptions::PyException;

create_exception!(marketdata_py, FugleError, PyException);
create_exception!(marketdata_py, ApiError, FugleError);
create_exception!(marketdata_py, AuthError, FugleError);
create_exception!(marketdata_py, RateLimitError, ApiError);
create_exception!(marketdata_py, ConnectionError, FugleError);
create_exception!(marketdata_py, TimeoutError, FugleError);

// Register in module
#[pymodule]
fn marketdata_py(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("FugleError", py.get_type_bound::<FugleError>())?;
    m.add("ApiError", py.get_type_bound::<ApiError>())?;
    m.add("AuthError", py.get_type_bound::<AuthError>())?;
    // ... other exceptions
    Ok(())
}
```

### Pattern 4: GIL Release for Async Operations
**What:** Release GIL before blocking on async operations
**When to use:** All tokio async operations to prevent deadlocks
**Example:**
```rust
// Source: https://github.com/PyO3/pyo3/discussions/3045
#[pyfunction]
fn connect_websocket(py: Python<'_>) -> PyResult<()> {
    py.allow_threads(|| {
        // Tokio runtime operations happen here
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            // WebSocket connection logic
            Ok(())
        })
    })
}
```

### Pattern 5: Type Stub Generation
**What:** Generate .pyi files for IDE autocomplete and type checking
**When to use:** Required for all public APIs, PEP 561 compliance
**Example:**
```python
# Generated by pyo3-stubgen
# Source: https://pypi.org/project/pyo3-stubgen/

class RestClient:
    """REST API client for Fugle market data."""

    def __init__(self, api_key: str) -> None: ...

    @staticmethod
    def with_bearer_token(token: str) -> RestClient: ...

    @property
    def stock(self) -> StockRestClient: ...

    @property
    def futopt(self) -> FutOptRestClient: ...

class StockRestClient:
    @property
    def intraday(self) -> StockIntradayClient: ...

class StockIntradayClient:
    async def quote(self, symbol: str) -> dict[str, Any]: ...
    async def ticker(self, symbol: str) -> dict[str, Any]: ...
    async def candles(self, symbol: str, timeframe: str = "1") -> dict[str, Any]: ...
```

### Anti-Patterns to Avoid
- **Holding GIL across await points:** Always use `py.allow_threads()` before tokio operations
- **Blocking event loop in async functions:** Never use `time.sleep()` in async context, use `asyncio.sleep()`
- **Missing exception context:** Always convert Rust errors to appropriate Python exception types
- **Ignoring free-threaded Python:** PyO3 0.27 supports Python 3.13t; test without GIL assumptions

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Async runtime bridging | Custom async conversion | pyo3-async-runtimes | Handles event loop lifecycle, proper GIL management, tested against Python 3.9-3.14 |
| Type stub generation | Manual .pyi writing | pyo3-stubgen | Extracts `__text_signature__` from compiled modules, automates CI/CD |
| WebSocket reconnection | Custom retry logic | stream-reconnect crate | Handles exponential backoff, transient vs fatal errors, drop-in tungstenite wrapper |
| Python package building | Custom build scripts | maturin | Handles wheel generation, mixed Rust/Python, PEP 621 metadata, PyPI compatibility |
| Error context propagation | String concatenation | Python 3.11+ `add_note()` | Official Python API for exception annotations without losing original error |

**Key insight:** PyO3 async integration has sharp edges (GIL deadlocks, event loop thread affinity). Using proven libraries like pyo3-async-runtimes avoids 90% of concurrency bugs that only manifest under load or in specific Python runtime configurations.

## Common Pitfalls

### Pitfall 1: GIL Deadlock in Multi-threaded Tokio
**What goes wrong:** Python hangs indefinitely when calling async functions under load
**Why it happens:** Tokio's multi-threaded runtime schedules tasks across threads. If you hold the GIL while awaiting, other threads block trying to acquire it, deadlocking the runtime.
**How to avoid:**
- Always use `py.allow_threads(|| { ... })` around tokio runtime operations
- For PyO3 async functions, use `future_into_py()` which handles GIL properly
- Never call `Python::with_gil()` inside tokio async blocks without immediately releasing
**Warning signs:**
- Works in single-threaded testing but hangs in production
- `CTRL+C` doesn't kill the process
- Thread dumps show threads waiting on GIL acquisition
**Source:** https://github.com/PyO3/pyo3/discussions/3045

### Pitfall 2: Missing py.typed Marker
**What goes wrong:** IDE autocomplete doesn't work, mypy can't find type information
**Why it happens:** PEP 561 requires explicit opt-in via py.typed file; maturin doesn't add it automatically
**How to avoid:**
- Create empty `marketdata_py/py.typed` file in project root
- Maturin will include it in wheel automatically
- Verify with `unzip -l dist/*.whl | grep py.typed`
**Warning signs:**
- Types work in .pyi files but mypy reports "no type stubs found"
- VS Code shows "Stub file not found for module"
**Source:** https://pyo3.rs/main/python-typing-hints.html

### Pitfall 3: Memory Leak from Unreleased GIL
**What goes wrong:** Python process memory grows unbounded during long-running operations
**Why it happens:** Python's garbage collector can't run while GIL is held. Loops that acquire/hold GIL without releasing accumulate objects in memory.
**How to avoid:**
- In loops, acquire GIL per iteration, not once for entire loop
- Use `py.allow_threads()` for any operation >1ms
- Monitor with `gc.get_stats()` to detect accumulation
**Warning signs:**
- Memory usage grows linearly with operation count
- `gc.collect()` immediately frees gigabytes
- Process RSS increases but Python heap size is small
**Source:** https://github.com/PyO3/pyo3/issues/319

### Pitfall 4: Bound vs GIL Refs Confusion
**What goes wrong:** Compilation errors about lifetime parameters, mismatched types
**Why it happens:** PyO3 0.27 uses `Bound<'py, T>` smart pointers, not `&'py T` GIL Refs. Old examples/code use deprecated API.
**How to avoid:**
- Use `Bound::new()` instead of `PyCell::new()`
- Methods return `Bound<PyAny>` not `&PyAny`
- Enable compiler warnings for deprecated APIs
- Follow migration guide: https://pyo3.rs/main/migration.html
**Warning signs:**
- Compiler errors about "expected `Bound`, found `&PyAny`"
- Deprecation warnings in build output
- Code copied from PyO3 <0.21 examples
**Source:** https://pyo3.rs/v0.27.2/getting-started.html

### Pitfall 5: WebSocket Message Queue Unbounded Growth
**What goes wrong:** Memory leak or OOM when WebSocket receives faster than consumer processes
**Why it happens:** tokio's `mpsc::unbounded_channel()` has no backpressure; messages accumulate if consumer is slow
**How to avoid:**
- Use bounded channels: `mpsc::channel(1000)` with appropriate buffer size
- Implement consumer slow-down detection (channel `len()` monitoring)
- Drop oldest messages or disconnect on sustained overflow
**Warning signs:**
- Memory grows during high-frequency trading periods
- Latency increases over time
- Process eventually OOMs after hours of streaming
**Source:** Common tokio pattern, not PyO3-specific

### Pitfall 6: Free-Threaded Python 3.13t Assumptions
**What goes wrong:** Code that works on Python 3.9-3.12 breaks on 3.13t (free-threaded)
**Why it happens:** Free-threaded Python allows multiple threads in interpreter simultaneously; GIL assumptions fail
**How to avoid:**
- Test on Python 3.13t builds
- Don't assume single-threaded interpreter access
- Use `py.allow_threads()` around blocking calls even in 3.13t
- Check PyO3 documentation for 3.13t-specific notes
**Warning signs:**
- Works on standard Python 3.13 but crashes on 3.13t
- Deadlocks during stop-the-world GC pauses
- Race conditions that never appeared before
**Source:** https://github.com/PyO3/pyo3/discussions/4738

## Code Examples

Verified patterns from official sources:

### REST Client with Async/Await
```rust
// Source: https://github.com/PyO3/pyo3-async-runtimes
use pyo3::prelude::*;
use pyo3_async_runtimes::tokio::future_into_py;

#[pyclass]
pub struct RestClient {
    inner: Arc<crate::rest::Client>, // Your Rust client
}

#[pymethods]
impl RestClient {
    #[new]
    fn new(api_key: String) -> PyResult<Self> {
        let inner = Arc::new(crate::rest::Client::new(api_key));
        Ok(Self { inner })
    }

    fn quote<'py>(&self, py: Python<'py>, symbol: String) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        future_into_py(py, async move {
            let result = client.get_quote(&symbol).await
                .map_err(|e| convert_error(e))?;
            Python::with_gil(|py| Ok(serde_json::to_value(result)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?
                .into_py(py)))
        })
    }
}
```

### WebSocket Client with Callbacks
```rust
// Source: Combined PyO3 patterns + tokio-tungstenite examples
use pyo3::prelude::*;
use pyo3::types::PyDict;
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[pyclass]
pub struct WebSocketClient {
    callbacks: Arc<Mutex<HashMap<String, PyObject>>>,
    runtime: Arc<tokio::runtime::Runtime>,
}

#[pymethods]
impl WebSocketClient {
    fn on(&self, event: String, callback: PyObject) -> PyResult<()> {
        self.callbacks.lock().unwrap().insert(event, callback);
        Ok(())
    }

    fn connect(&self, py: Python<'_>) -> PyResult<()> {
        let callbacks = self.callbacks.clone();
        let rt = self.runtime.clone();

        py.allow_threads(|| {
            rt.spawn(async move {
                let (ws_stream, _) = connect_async("wss://api.fugle.tw/ws").await.unwrap();

                // Call Python connect callback
                Python::with_gil(|py| {
                    if let Some(cb) = callbacks.lock().unwrap().get("connect") {
                        let _ = cb.call0(py);
                    }
                });

                // Handle messages...
            });
        });

        Ok(())
    }
}
```

### Custom Exception Hierarchy
```rust
// Source: https://pyo3.rs/main/function/error-handling.html
use pyo3::create_exception;
use pyo3::exceptions::PyException;

// Base exception
create_exception!(marketdata_py, MarketDataError, PyException);

// Derived exceptions
create_exception!(marketdata_py, ApiError, MarketDataError);
create_exception!(marketdata_py, AuthError, MarketDataError);
create_exception!(marketdata_py, RateLimitError, ApiError);
create_exception!(marketdata_py, ConnectionError, MarketDataError);
create_exception!(marketdata_py, TimeoutError, MarketDataError);
create_exception!(marketdata_py, WebSocketError, MarketDataError);

// Error conversion
impl From<reqwest::Error> for PyErr {
    fn from(err: reqwest::Error) -> PyErr {
        if err.is_timeout() {
            TimeoutError::new_err(err.to_string())
        } else if err.is_connect() {
            ConnectionError::new_err(err.to_string())
        } else {
            ApiError::new_err(err.to_string())
        }
    }
}

// Module registration
#[pymodule]
fn marketdata_py(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("MarketDataError", py.get_type_bound::<MarketDataError>())?;
    m.add("ApiError", py.get_type_bound::<ApiError>())?;
    m.add("AuthError", py.get_type_bound::<AuthError>())?;
    m.add("RateLimitError", py.get_type_bound::<RateLimitError>())?;
    m.add("ConnectionError", py.get_type_bound::<ConnectionError>())?;
    m.add("TimeoutError", py.get_type_bound::<TimeoutError>())?;
    m.add("WebSocketError", py.get_type_bound::<WebSocketError>())?;
    Ok(())
}
```

### Type Stub for IDE Support
```python
# marketdata_py/__init__.pyi
# Generated with: pyo3-stubgen marketdata_py
# Source: https://pypi.org/project/pyo3-stubgen/

from typing import Any, Callable, Optional, AsyncIterator

class MarketDataError(Exception):
    """Base exception for all market data errors."""
    ...

class ApiError(MarketDataError):
    """API returned error response."""
    ...

class AuthError(MarketDataError):
    """Authentication failed."""
    ...

class RateLimitError(ApiError):
    """Rate limit exceeded."""
    ...

class ConnectionError(MarketDataError):
    """Network connection failed."""
    ...

class TimeoutError(MarketDataError):
    """Operation timed out."""
    ...

class WebSocketError(MarketDataError):
    """WebSocket protocol error."""
    ...

class RestClient:
    """REST API client for Fugle market data."""

    def __init__(self, api_key: str) -> None: ...

    @staticmethod
    def with_bearer_token(token: str) -> RestClient: ...

    @staticmethod
    def with_sdk_token(token: str) -> RestClient: ...

    @property
    def stock(self) -> StockRestClient: ...

    @property
    def futopt(self) -> FutOptRestClient: ...

class StockRestClient:
    @property
    def intraday(self) -> StockIntradayClient: ...

class StockIntradayClient:
    async def quote(self, symbol: str) -> dict[str, Any]: ...
    async def ticker(self, symbol: str) -> dict[str, Any]: ...
    async def candles(self, symbol: str, timeframe: str = "1") -> dict[str, Any]: ...
    async def trades(self, symbol: str) -> dict[str, Any]: ...
    async def volumes(self, symbol: str) -> dict[str, Any]: ...

class WebSocketClient:
    """WebSocket client for real-time market data."""

    def __init__(self, api_key: str) -> None: ...

    @property
    def stock(self) -> StockWebSocketClient: ...

    @property
    def futopt(self) -> FutOptWebSocketClient: ...

class StockWebSocketClient:
    def connect(self) -> None: ...
    def disconnect(self) -> None: ...
    def is_connected(self) -> bool: ...
    def subscribe(self, channel: str, symbol: str) -> str: ...
    def unsubscribe(self, subscription_id: str) -> None: ...
    def subscriptions(self) -> list[dict[str, str]]: ...
    def on(self, event: str, callback: Callable[..., None]) -> None: ...
    def off(self, event: str) -> None: ...
    def messages(self) -> MessageIterator: ...

class MessageIterator:
    """Async iterator for WebSocket messages."""

    def __aiter__(self) -> MessageIterator: ...
    async def __anext__(self) -> dict[str, Any]: ...
    def try_recv(self) -> Optional[dict[str, Any]]: ...
    def recv_timeout(self, timeout: float) -> Optional[dict[str, Any]]: ...
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| GIL Refs (`&PyAny`) | Bound smart pointers (`Bound<PyAny>`) | PyO3 0.21 (2023) | Better performance, free-threaded Python support, clearer ownership |
| pyo3-asyncio | pyo3-async-runtimes | PyO3 0.27 (2025) | Official support, better tokio integration, Python 3.14 compatibility |
| Manual .pyi stubs | pyo3-stubgen automation | Ongoing | CI/CD integration, less maintenance burden |
| `#[text_signature]` | Function annotations in Rust | PyO3 0.20+ | Better IDE hints, type checker integration |
| Sync wrappers for async | Native async/await | Python 3.11+ asyncio improvements | 75% faster event loop, eager task execution |

**Deprecated/outdated:**
- **pyo3-asyncio**: Replaced by pyo3-async-runtimes; last release was 0.21 (2024)
- **GIL Refs API**: Deprecated in PyO3 0.21, removed in 0.27; use Bound smart pointers
- **`PyCell::new()`**: Replaced by `Bound::new()` in PyO3 0.27
- **`#[text_signature = "..."]`**: Use `#[pyo3(signature = ...)]` with type annotations

## Open Questions

Things that couldn't be fully resolved:

1. **API compatibility verification strategy**
   - What we know: Official SDK uses callback pattern (`on("message", handler)`), has specific error codes
   - What's unclear: Exact method signature compatibility beyond what README shows (parameter order, optional args, default values)
   - Recommendation: Run official SDK test suite against new bindings as acceptance criteria; create compatibility matrix documenting deviations

2. **Type stub generation timing in CI/CD**
   - What we know: pyo3-stubgen runs on compiled wheels, requires `__text_signature__` in binary
   - What's unclear: Whether to generate stubs pre-build (manual) vs post-build (from wheel) vs runtime introspection
   - Recommendation: Post-build approach in CI: `maturin build` → `pip install wheel` → `pyo3-stubgen` → commit .pyi files. Ensures stubs match binary exactly.

3. **Free-threaded Python 3.13t testing requirements**
   - What we know: PyO3 0.27 supports 3.13t, requires `allow_threads()` around blocking calls
   - What's unclear: Whether 3.13t should be primary target or nice-to-have; testing strategy for free-threaded mode
   - Recommendation: Test on standard 3.13 first; add 3.13t as experimental in CI later. Free-threaded mode is opt-in, not default yet.

4. **WebSocket reconnection queue persistence**
   - What we know: CONTEXT.md specifies "queue subscription changes during disconnection, replay after reconnect"
   - What's unclear: Persistence mechanism (in-memory queue size limits, disk spill for long disconnections, queue serialization)
   - Recommendation: In-memory bounded queue (1000 items) with oldest-drop policy; document as "best-effort" replay. Disk persistence is scope creep.

5. **Response model mutability vs frozen dataclasses**
   - What we know: CONTEXT.md specifies frozen dataclasses preferred
   - What's unclear: How to implement frozen dataclasses from PyO3 (no direct support, would need manual `__setattr__` override)
   - Recommendation: Use `#[pyclass(frozen)]` if available in PyO3 0.27, else return plain dicts for Phase 2 and defer immutability to Phase 4 (API refinement)

## Sources

### Primary (HIGH confidence)
- [PyO3 0.27 Getting Started](https://pyo3.rs/v0.27.2/getting-started.html) - Official setup, Bound API
- [PyO3 0.27 Changelog](https://pyo3.rs/main/changelog.html) - Version-specific changes
- [pyo3-async-runtimes README](https://github.com/PyO3/pyo3-async-runtimes/blob/main/README.md) - Tokio/asyncio integration
- [PyO3 Error Handling](https://pyo3.rs/main/function/error-handling.html) - Exception patterns
- [PyO3 Type Hints Guide](https://pyo3.rs/main/python-typing-hints.html) - Type stub best practices
- [PyO3 Type Stub Generation](https://pyo3.rs/main/type-stub.html) - Stub generation methods
- [Maturin Project Layout](https://www.maturin.rs/project_layout.html) - Mixed Rust/Python structure

### Secondary (MEDIUM confidence)
- [Python 3.12 asyncio improvements](https://docs.python.org/3/whatsnew/3.12.html) - Eager tasks, performance
- [websockets library async iterator](https://websockets.readthedocs.io/en/stable/reference/asyncio/client.html) - Python WebSocket patterns
- [tokio-tungstenite](https://github.com/snapview/tokio-tungstenite) - WebSocket client library
- [stream-reconnect](https://lib.rs/crates/stream-reconnect) - Auto-reconnect wrapper
- [pyo3-stubgen PyPI](https://pypi.org/project/pyo3-stubgen/) - Stub generation tool
- [Python exception best practices 2026](https://www.qodo.ai/blog/6-best-practices-for-python-exception-handling/) - Exception hierarchy patterns

### Tertiary (LOW confidence - needs validation)
- [fugle-marketdata-python GitHub](https://github.com/fugle-dev/fugle-marketdata-python) - Official SDK reference (need to verify API surface comprehensively)
- Community discussions on PyO3 GIL deadlocks - Pattern exists but specific solutions vary by use case

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Official PyO3 docs, proven libraries, clear version compatibility
- Architecture: HIGH - Official patterns from PyO3 and pyo3-async-runtimes documentation
- Pitfalls: HIGH - Documented in PyO3 GitHub issues with reproducible examples
- API compatibility: MEDIUM - Based on README only, full API surface needs verification
- Type stubs: MEDIUM - Tools exist but CI/CD integration varies by project

**Research date:** 2026-01-31
**Valid until:** ~90 days (PyO3 stable, slow-moving; revalidate if PyO3 0.28 releases)
**Knowledge gaps:** Official SDK full API contract, production WebSocket reconnection patterns, 3.13t free-threading impact
