# Architecture Patterns: Multi-Language SDK with Rust Core

**Domain:** Multi-language SDK (Rust → Python/Node.js/C#)
**Researched:** 2026-01-30
**Current State:** Python (PyO3) and Node.js (napi-rs) bindings operational, UniFFI experimental

## Executive Summary

Multi-language SDK architecture with shared Rust core follows a **shared computation, language-specific interface** pattern. The core library (`marketdata-core`) contains business logic, while language-specific binding crates (`py/`, `js/`, `uniffi/`) provide idiomatic interfaces for each ecosystem. This architecture maximizes code reuse (~80-90% shared), ensures consistent behavior across platforms, and allows language-specific optimizations at the FFI boundary.

**Key architectural decision:** Three binding strategies coexist:
1. **PyO3** (Python) - Specialized, ergonomic, direct control over Python types
2. **napi-rs** (Node.js) - Specialized, async-first, event loop integration
3. **UniFFI** (C#/Go/etc.) - General-purpose, JSON serialization, broader language support

Each strategy represents different trade-offs between performance, ergonomics, and language coverage.

---

## Current Architecture Analysis

### Component Structure

```
marketdata-core/          # Rust core library
├── src/
│   ├── client/          # REST client (sync API wrapper)
│   ├── websocket/       # WebSocket client (async streams)
│   ├── auth/            # Authentication logic
│   └── error/           # Error types
└── Cargo.toml           # Core dependencies (tokio, ureq, etc.)

py/                       # Python bindings (PyO3)
├── src/
│   ├── lib.rs           # PyO3 module definition
│   ├── client.rs        # REST client wrapper
│   ├── websocket.rs     # WebSocket with callbacks
│   ├── iterator.rs      # Python iterator for messages
│   └── errors.rs        # Python exception mapping
└── Cargo.toml           # cdylib, depends on core + pyo3

js/                       # Node.js bindings (napi-rs)
├── src/
│   ├── lib.rs           # NAPI module definition
│   ├── client.rs        # REST client wrapper
│   ├── websocket.rs     # WebSocket with EventEmitter
│   └── errors.rs        # JavaScript error mapping
└── Cargo.toml           # cdylib, depends on core + napi

uniffi/                   # Multi-language bindings (UniFFI)
├── src/
│   ├── lib.rs           # UniFFI scaffolding
│   ├── client.rs        # REST client (JSON returns)
│   ├── futopt.rs        # FutOpt endpoints
│   └── marketdata.udl   # Interface definition
└── Cargo.toml           # lib + cdylib, depends on core + uniffi
```

**Observations:**
- **No workspace configuration** - Each crate independently depends on `marketdata-core` via `path = "../core"`
- **Duplication across bindings** - Each binding recreates similar client/websocket/error patterns
- **Inconsistent API surfaces** - Python has iterators, Node.js has EventEmitter, UniFFI returns JSON strings
- **Build independence** - Each binding can build separately (flexibility) but no shared build optimization

---

## Recommended Architecture Patterns

### 1. Component Boundaries

**Clear separation of concerns:**

| Component | Responsibility | Exposed To FFI | Language-Specific |
|-----------|---------------|----------------|-------------------|
| **Core** | Business logic, API protocol, tokio async operations | No | No |
| **FFI Bridge** | Type conversion, error mapping, async runtime bridging | Yes (C ABI) | Yes |
| **Language Layer** | Idiomatic API, language-specific patterns, documentation | Yes (language runtime) | Yes |

**What belongs where:**

```rust
// CORE LAYER (marketdata-core)
// ✅ Business logic, pure Rust types
pub struct RestClient {
    auth: Auth,
    http: ureq::Agent,
}

impl RestClient {
    pub async fn get_quote(&self, symbol: &str) -> Result<Quote, MarketDataError> {
        // HTTP request, parsing, error handling
    }
}

// FFI BRIDGE LAYER (py/js/uniffi)
// ✅ Type conversion, lifetime management, async bridging
#[pyclass]
pub struct PyRestClient {
    inner: Arc<RestClient>,  // Shared ownership for Python GC
    runtime: tokio::runtime::Handle,  // Async bridge
}

#[pymethods]
impl PyRestClient {
    fn quote(&self, symbol: String) -> PyResult<PyObject> {
        // Convert Python types → Rust types
        // Bridge tokio → asyncio
        // Convert Result → PyResult
    }
}

// LANGUAGE LAYER (Python/JS code)
// ✅ Pythonic/JavaScript patterns, documentation, convenience
class RestClient:
    """REST client for Fugle market data API.

    Examples:
        >>> client = RestClient("api-key")
        >>> quote = client.stock.intraday.quote("2330")
    """

    @property
    def stock(self) -> StockClient:
        return StockClient(self._inner)
```

**Boundary decisions:**

| Concern | Core | FFI Bridge | Language Layer |
|---------|------|------------|----------------|
| HTTP requests | ✅ | ❌ | ❌ |
| JSON parsing | ✅ | ❌ | ❌ |
| Error types | ✅ (Rust enums) | ✅ (conversion) | ❌ |
| Async runtime | ✅ (tokio) | ✅ (bridging) | ❌ |
| API design | ✅ (traits/structs) | ❌ | ✅ (idiomatic) |
| Documentation | Core docs | FFI docs | User docs |

**Rationale:** This three-layer separation ensures core logic remains pure Rust (testable, maintainable), FFI layer handles impedance mismatch (types, lifetimes, async), and language layer provides natural developer experience.

---

### 2. FFI Boundary Design

**What crosses the FFI boundary:**

#### Principle: Minimize Boundary Crossings

FFI calls incur overhead (~100-500ns per call). Design APIs to batch operations and reduce round-trips.

**Good boundary crossings:**

```rust
// ✅ Simple scalar types (Copy types)
fn get_price(symbol: String) -> f64

// ✅ Owned strings (allocated on Rust side, freed on Rust side)
fn get_symbol_name(symbol: &str) -> String

// ✅ Opaque handles (Arc<T> wrapped in newtype)
struct RestClient(Arc<RestClientInner>);

// ✅ Result types (mapped to language exceptions)
fn quote(symbol: &str) -> Result<String, MarketDataError>

// ✅ Serialized complex types (JSON for UniFFI)
fn get_candles(symbol: &str) -> Result<String, MarketDataError>  // Returns JSON
```

**Problematic boundary crossings:**

```rust
// ❌ Complex nested structures (serialization overhead)
fn get_orderbook() -> Result<OrderBook, Error>  // OrderBook has Vec<Vec<f64>>

// ❌ Callbacks crossing back and forth (threading issues)
fn register_callback(cb: impl Fn(Message))  // Callback might outlive FFI context

// ❌ Mutable references (lifetime/aliasing issues)
fn update_config(&mut self, config: &Config)  // &mut self unsafe in FFI

// ❌ Borrowed data (lifetime management across FFI)
fn get_symbols(&self) -> &[String]  // Who owns this slice?
```

**Recommended patterns:**

| Data Type | FFI Strategy | Example |
|-----------|-------------|---------|
| **Primitives** | Pass by value | `i32`, `f64`, `bool` |
| **Strings** | Owned `String` (Rust allocates/frees) | `symbol: String` |
| **Complex types** | JSON serialization | `Result<String, Error>` → parse in language layer |
| **Objects** | Opaque handles (`Arc<T>`) | `#[pyclass] struct Client(Arc<Inner>)` |
| **Collections** | JSON or owned `Vec<T>` | `Vec<String>` or `serde_json::to_string(&data)` |
| **Callbacks** | Thread-safe function pointers | `napi::ThreadsafeFunction`, `PyObject::call_method()` |
| **Errors** | Enum → Exception mapping | `Result<T, E>` → `PyResult<T>`, `napi::Result<T>` |

**Current project analysis:**

```rust
// Current PyO3 approach: Native Python types
#[pymethods]
impl StockIntradayClient {
    fn quote(&self, py: Python, symbol: String) -> PyResult<PyObject> {
        let core_result = self.core_client.quote(&symbol)?;
        pythonize(py, &core_result)  // Converts Rust struct → Python dict
    }
}

// Current UniFFI approach: JSON strings
impl StockIntradayClient {
    fn quote(&self, symbol: String) -> Result<String, MarketDataError> {
        let core_result = self.core_client.quote(&symbol)?;
        serde_json::to_string(&core_result)  // Returns JSON string
            .map_err(|e| MarketDataError::DeserializationError(e.to_string()))
    }
}
```

**Trade-offs:**

| Strategy | Performance | Type Safety | Flexibility | DX |
|----------|-------------|-------------|-------------|-----|
| **Native types (PyO3/napi-rs)** | High (5-10x faster) | High (compile-time) | Low (language-specific) | Excellent |
| **JSON serialization (UniFFI)** | Medium (parsing overhead) | Low (runtime) | High (any language) | Good |
| **Binary formats (MessagePack)** | Very high | Low | Medium | Poor |

**Recommendation:**
- **Specialized bindings (PyO3/napi-rs):** Use native types for performance-critical paths (quote, ticker, WebSocket messages)
- **General bindings (UniFFI):** Use JSON for C#/Go where ergonomics matter more than peak performance
- **Hybrid approach:** Offer both native and JSON APIs, let users choose based on use case

---

### 3. Async Runtime Bridging

**Challenge:** Each language has its own async runtime with different threading models and scheduling behavior.

#### Tokio ↔ Python asyncio

**Architecture:**

```
┌─────────────────────────────────────────────────────┐
│ Python Main Thread (asyncio event loop)            │
│                                                     │
│  async def main():                                  │
│      client = RestClient("key")                     │
│      quote = await client.quote("2330")  ←──────┐  │
│                                                  │  │
└──────────────────────────────────────────────────│──┘
                                                   │
                      FFI boundary                 │
                                                   │
┌──────────────────────────────────────────────────│──┐
│ Rust PyO3 Bridge                                 │  │
│                                                  │  │
│  #[pymethods]                                    │  │
│  impl PyRestClient {                             │  │
│      fn quote<'py>(&self, py: Python<'py>,      │  │
│                    symbol: String)               │  │
│          -> PyResult<Bound<'py, PyAny>> {        │  │
│                                                  │  │
│          // 1. Release GIL                       │  │
│          // 2. Spawn tokio task                  │  │
│          // 3. Block on completion               │  │
│          // 4. Reacquire GIL                     │  │
│          // 5. Return Python awaitable           │  │
│                                                  │  │
│          pyo3_async_runtimes::tokio::future_into_py(py, async {
│              self.inner.quote(&symbol).await     │  │
│          })                                      │  │
│      }                                           │  │
│  }                                               │  │
│                                                  │  │
└──────────────────────────────────────────────────│──┘
                                                   │
┌──────────────────────────────────────────────────│──┐
│ Tokio Runtime (background thread pool)          │  │
│                                                  ▼  │
│  tokio::spawn(async {                               │
│      let response = http_client.get(url).await;     │
│      parse_json(response)                           │
│  })                                                 │
│                                                     │
└─────────────────────────────────────────────────────┘
```

**Key patterns:**

1. **GIL management** - Release Python's Global Interpreter Lock during I/O to prevent blocking other Python threads
2. **Future conversion** - `pyo3-async-runtimes` bridges tokio futures → Python awaitables
3. **Error propagation** - Rust `Result` → Python exceptions with proper traceback

**Implementation:**

```rust
use pyo3::prelude::*;
use pyo3_async_runtimes::tokio::future_into_py;

#[pymethods]
impl PyRestClient {
    fn quote<'py>(&self, py: Python<'py>, symbol: String)
        -> PyResult<Bound<'py, PyAny>>
    {
        let client = self.inner.clone();

        future_into_py(py, async move {
            // This runs in tokio runtime
            client.quote(&symbol)
                .await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                    format!("Quote failed: {}", e)
                ))
        })
    }
}
```

**Reference:** [PyO3 async-runtimes](https://github.com/PyO3/pyo3-async-runtimes) provides bridges between Python and Rust async runtimes.

#### Tokio ↔ Node.js Event Loop

**Architecture:**

```
┌─────────────────────────────────────────────────────┐
│ Node.js Main Thread (libuv event loop)             │
│                                                     │
│  const client = new RestClient("key");              │
│  const quote = await client.quote("2330");  ←───┐  │
│                                                  │  │
└──────────────────────────────────────────────────│──┘
                                                   │
                      FFI boundary                 │
                                                   │
┌──────────────────────────────────────────────────│──┐
│ Rust NAPI-RS Bridge                              │  │
│                                                  │  │
│  #[napi]                                         │  │
│  impl RestClient {                               │  │
│      #[napi]                                     │  │
│      pub async fn quote(&self, symbol: String)   │  │
│          -> napi::Result<JsObject> {             │  │
│                                                  │  │
│          // napi-rs automatically:               │  │
│          // 1. Spawns tokio task                 │  │
│          // 2. Returns JS Promise                │  │
│          // 3. Resolves on completion            │  │
│                                                  │  │
│          let result = self.inner.quote(&symbol)  │  │
│              .await                              │  │
│              .map_err(|e| napi::Error::from_reason(e.to_string()))?;
│                                                  │  │
│          // Convert to JS object                 │  │
│          env.to_js_value(&result)                │  │
│      }                                           │  │
│  }                                               │  │
│                                                  │  │
└──────────────────────────────────────────────────│──┘
                                                   │
┌──────────────────────────────────────────────────│──┐
│ Tokio Runtime (shared with napi-rs)             │  │
│                                                  ▼  │
│  tokio::spawn(async {                               │
│      let response = http_client.get(url).await;     │
│      parse_json(response)                           │
│  })                                                 │
│                                                     │
└─────────────────────────────────────────────────────┘
```

**Key patterns:**

1. **Automatic bridging** - napi-rs with `tokio_rt` feature automatically converts `async fn` → JS Promise
2. **Shared runtime** - Tokio runtime lives for the lifetime of the Node.js process
3. **ThreadsafeFunction** - For callbacks from Rust → JavaScript across threads

**Implementation:**

```rust
use napi::bindgen_prelude::*;
use napi_derive::napi;

#[napi]
impl RestClient {
    // napi-rs automatically converts this to a JS Promise
    #[napi]
    pub async fn quote(&self, symbol: String) -> Result<JsObject> {
        // This runs in tokio runtime, returns Promise to JS
        let data = self.inner
            .quote(&symbol)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))?;

        // Convert to JS object (serde_json feature)
        Ok(to_js_value(&data)?)
    }
}
```

**Reference:** [NAPI-RS async functions](https://napi.rs/docs/concepts/async-fn) documentation on tokio runtime integration.

#### Tokio ↔ C# Task

**Architecture:**

```
┌─────────────────────────────────────────────────────┐
│ C# ThreadPool                                       │
│                                                     │
│  var client = new RestClient("key");                │
│  var quote = await client.QuoteAsync("2330"); ←─┐  │
│                                                  │  │
└──────────────────────────────────────────────────│──┘
                                                   │
                      FFI boundary                 │
                                                   │
┌──────────────────────────────────────────────────│──┐
│ Rust UniFFI Bridge                               │  │
│                                                  │  │
│  impl StockIntradayClient {                      │  │
│      pub fn quote(&self, symbol: String)         │  │
│          -> Result<String, MarketDataError> {    │  │
│                                                  │  │
│          // UniFFI is sync only!                 │  │
│          // Block on tokio future                │  │
│          let runtime = tokio::runtime::Handle::current();
│          runtime.block_on(async {                │  │
│              let result = self.core_client       │  │
│                  .quote(&symbol)                 │  │
│                  .await?;                        │  │
│              serde_json::to_string(&result)      │  │
│                  .map_err(|e| MarketDataError::DeserializationError(e.to_string()))
│          })                                      │  │
│      }                                           │  │
│  }                                               │  │
│                                                  │  │
└──────────────────────────────────────────────────│──┘
                                                   │
┌──────────────────────────────────────────────────│──┐
│ Tokio Runtime (background thread)               │  │
│                                                  ▼  │
│  tokio::spawn(async {                               │
│      let response = http_client.get(url).await;     │
│      parse_json(response)                           │
│  })                                                 │
│                                                     │
└─────────────────────────────────────────────────────┘
```

**Challenge:** UniFFI does not support async functions. Must either:
1. **Block on tokio** - Use `tokio::runtime::Handle::block_on()` (simple but blocks thread)
2. **Spawn + polling** - Return handle, poll for completion (complex but non-blocking)
3. **C# wrapper** - Sync Rust → wrap in `Task.Run()` in C# layer (best for UX)

**Current implementation (blocking):**

```rust
// uniffi/src/client.rs
impl StockIntradayClient {
    pub fn quote(&self, symbol: String) -> Result<String, MarketDataError> {
        // Block current thread until tokio future completes
        tokio::runtime::Handle::current()
            .block_on(async {
                let result = self.core_client.quote(&symbol).await?;
                serde_json::to_string(&result)
                    .map_err(|e| MarketDataError::DeserializationError(e.to_string()))
            })
    }
}
```

**Recommended C# wrapper:**

```csharp
// C# bindings layer
public class RestClient {
    private readonly RestClientFFI _inner;  // Generated by UniFFI

    public async Task<Quote> QuoteAsync(string symbol) {
        return await Task.Run(() => {
            // Call sync FFI on background thread
            string json = _inner.Quote(symbol);
            return JsonSerializer.Deserialize<Quote>(json);
        });
    }
}
```

**Alternative:** Use [Interoptopus](https://docs.rs/interoptopus/) or [csbindgen](https://github.com/Cysharp/csbindgen) which have better async support for C#.

**Trade-offs:**

| Strategy | Blocking | Threading | Complexity | Performance |
|----------|----------|-----------|------------|-------------|
| **block_on** | Yes (FFI thread) | Simple | Low | Medium (thread blocked) |
| **Spawn + poll** | No | Manual management | High | High (non-blocking) |
| **C# Task.Run** | Yes (C# threadpool) | C# manages | Low | High (C# scheduler optimized) |

**Recommendation:** Use C# `Task.Run()` wrapper for best balance of simplicity and performance.

---

### 4. WebSocket Async Callbacks

**Challenge:** WebSocket events are async and continuous (stream of messages), requiring different patterns than request-response.

#### Python: Callback + Iterator Pattern

```python
# Pattern 1: Callback
ws = WebSocketClient("key")

def on_message(msg):
    print(f"Received: {msg}")

ws.stock.on("message", on_message)
ws.stock.connect()
ws.stock.subscribe("trades", "2330")

# Pattern 2: Iterator (async generator)
async for msg in ws.stock.messages():
    print(msg)
```

**Implementation:**

```rust
use pyo3::prelude::*;
use tokio::sync::mpsc;

#[pyclass]
pub struct StockWebSocketClient {
    tx: mpsc::UnboundedSender<WsMessage>,
    callbacks: Arc<Mutex<HashMap<String, PyObject>>>,  // event -> callback
}

#[pymethods]
impl StockWebSocketClient {
    // Register callback
    fn on(&self, event: String, callback: PyObject) -> PyResult<()> {
        self.callbacks.lock().unwrap().insert(event, callback);
        Ok(())
    }

    // Async iterator
    fn messages<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, MessageIterator>> {
        let (tx, rx) = mpsc::unbounded_channel();
        // Spawn tokio task that forwards WebSocket messages to channel
        Ok(Bound::new(py, MessageIterator { rx })?)
    }
}

#[pyclass]
pub struct MessageIterator {
    rx: mpsc::UnboundedReceiver<String>,
}

#[pymethods]
impl MessageIterator {
    fn __aiter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __anext__<'py>(&mut self, py: Python<'py>) -> PyResult<Option<String>> {
        // Poll channel, return None if closed
        match self.rx.try_recv() {
            Ok(msg) => Ok(Some(msg)),
            Err(_) => Ok(None),
        }
    }
}
```

**Reference:** [pyo3-asyncio](https://crates.io/crates/pyo3-asyncio) for bridging tokio futures to Python asyncio.

#### Node.js: EventEmitter Pattern

```javascript
const ws = new WebSocketClient("key");

ws.stock.on("message", (msg) => {
    console.log("Received:", msg);
});

ws.stock.connect();
ws.stock.subscribe("trades", "2330");
```

**Implementation:**

```rust
use napi::bindgen_prelude::*;
use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};

#[napi]
pub struct StockWebSocketClient {
    inner: Arc<WebSocketClientInner>,
    message_callback: Option<ThreadsafeFunction<String>>,
}

#[napi]
impl StockWebSocketClient {
    #[napi]
    pub fn on(&mut self, env: Env, event: String, callback: JsFunction) -> Result<()> {
        if event == "message" {
            // Create thread-safe function that can be called from Rust threads
            let tsfn: ThreadsafeFunction<String> = callback
                .create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))?;

            self.message_callback = Some(tsfn);

            // Spawn tokio task to forward messages
            let tsfn_clone = tsfn.clone();
            let inner = self.inner.clone();
            tokio::spawn(async move {
                while let Some(msg) = inner.next_message().await {
                    // Call JS callback from Rust thread
                    tsfn_clone.call(msg, ThreadsafeFunctionCallMode::NonBlocking);
                }
            });
        }
        Ok(())
    }
}
```

**Reference:** [NAPI-RS ThreadsafeFunction](https://napi.rs/docs/concepts/async-fn#threadsafefunction) for cross-thread callbacks.

#### C#: Event Pattern

```csharp
var ws = new WebSocketClient("key");

ws.Stock.MessageReceived += (sender, msg) => {
    Console.WriteLine($"Received: {msg}");
};

await ws.Stock.ConnectAsync();
await ws.Stock.SubscribeAsync("trades", "2330");
```

**Implementation (C# wrapper over UniFFI):**

```csharp
public class StockWebSocketClient {
    private readonly StockWebSocketClientFFI _inner;
    private CancellationTokenSource _cts;

    public event EventHandler<string> MessageReceived;

    public async Task ConnectAsync() {
        _inner.Connect();

        _cts = new CancellationTokenSource();

        // Background task to poll messages
        _ = Task.Run(async () => {
            while (!_cts.Token.IsCancellationRequested) {
                // Poll FFI for next message (blocking call)
                string msg = await Task.Run(() => _inner.NextMessage());
                if (msg != null) {
                    MessageReceived?.Invoke(this, msg);
                }
            }
        }, _cts.Token);
    }
}
```

**Alternative:** Use [System.Threading.Channels](https://learn.microsoft.com/en-us/dotnet/api/system.threading.channels) for better async message passing.

---

### 5. Error Handling Across FFI

**Principle:** Map Rust `Result<T, E>` → language-specific error conventions.

#### Error Mapping Strategy

```rust
// Core library error type
#[derive(Debug, thiserror::Error)]
pub enum MarketDataError {
    #[error("Invalid symbol: {symbol}")]
    InvalidSymbol { symbol: String },

    #[error("API error (HTTP {status}): {message}")]
    ApiError { status: u16, message: String },

    #[error("Connection error: {msg}")]
    ConnectionError { msg: String },

    #[error("Timeout: {operation}")]
    TimeoutError { operation: String },
}
```

**PyO3 mapping:**

```rust
use pyo3::exceptions::PyException;

impl From<MarketDataError> for PyErr {
    fn from(err: MarketDataError) -> PyErr {
        match err {
            MarketDataError::InvalidSymbol { symbol } => {
                PyValueError::new_err(format!("Invalid symbol: {}", symbol))
            }
            MarketDataError::ApiError { status, message } => {
                PyRuntimeError::new_err(format!("API error ({}): {}", status, message))
            }
            MarketDataError::ConnectionError { msg } => {
                PyConnectionError::new_err(msg)
            }
            _ => PyException::new_err(err.to_string()),
        }
    }
}
```

**napi-rs mapping:**

```rust
impl From<MarketDataError> for napi::Error {
    fn from(err: MarketDataError) -> napi::Error {
        match err {
            MarketDataError::InvalidSymbol { symbol } => {
                napi::Error::new(
                    napi::Status::InvalidArg,
                    format!("Invalid symbol: {}", symbol)
                )
            }
            MarketDataError::ApiError { status, message } => {
                napi::Error::new(
                    napi::Status::GenericFailure,
                    format!("API error ({}): {}", status, message)
                )
            }
            _ => napi::Error::from_reason(err.to_string()),
        }
    }
}
```

**UniFFI mapping:**

```rust
// UniFFI requires explicit error enum in UDL
#[derive(Debug, thiserror::Error)]
pub enum MarketDataError {
    #[error("Invalid symbol: {0}")]
    InvalidSymbol(String),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("{0}")]
    Other(String),
}

impl From<CoreError> for MarketDataError {
    fn from(err: CoreError) -> Self {
        match err {
            CoreError::InvalidSymbol { symbol } => {
                MarketDataError::InvalidSymbol(symbol)
            }
            CoreError::ApiError { status, message } => {
                MarketDataError::ApiError(format!("HTTP {}: {}", status, message))
            }
            _ => MarketDataError::Other(err.to_string()),
        }
    }
}
```

**Trade-offs:**

| Strategy | Type Safety | Stack Traces | Localization | DX |
|----------|-------------|--------------|--------------|-----|
| **Native exceptions** | High | Excellent | Easy | Excellent |
| **Error codes** | Low | None | Hard | Poor |
| **JSON error objects** | Medium | Manual | Medium | Good |

**Recommendation:** Use native exception types (PyErr, napi::Error, C# Exception) for best developer experience. Preserve error details and stack traces.

---

## Build System Organization

### Current State: Independent Builds

```bash
# Build each binding separately
cd core && cargo build --release
cd py && maturin build --release
cd js && npm run build
cd uniffi && cargo build --release && cargo run --bin uniffi-bindgen
```

**Issues:**
- No dependency ordering enforcement
- Core changes require manual rebuild of all bindings
- No shared artifact caching
- Inconsistent versioning across bindings

### Recommended: Cargo Workspace

```toml
# Root Cargo.toml
[workspace]
members = [
    "core",
    "py",
    "js",
    "uniffi",
]

[workspace.package]
version = "0.2.0"
edition = "2021"
authors = ["Fugle <team@fugle.tw>"]
license = "MIT"

[workspace.dependencies]
# Shared dependencies across all crates
tokio = { version = "1.49", features = ["rt", "rt-multi-thread", "sync", "time", "macros"] }
thiserror = "2.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"

# FFI crates
pyo3 = { version = "0.22", features = ["extension-module"] }
napi = { version = "2.16", features = ["napi8", "async", "serde-json", "tokio_rt"] }
napi-derive = "2.16"
uniffi = { version = "0.28", features = ["cli"] }
```

```toml
# core/Cargo.toml
[package]
name = "marketdata-core"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true

[dependencies]
tokio.workspace = true
thiserror.workspace = true
serde.workspace = true
serde_json.workspace = true
# ... other deps
```

```toml
# py/Cargo.toml
[package]
name = "marketdata-py"
version.workspace = true
edition.workspace = true

[dependencies]
marketdata-core = { path = "../core" }
pyo3.workspace = true
serde_json.workspace = true
tokio.workspace = true
```

**Benefits:**
- **Single Cargo.lock** - Ensures version consistency across all crates
- **Shared target directory** - Faster builds, reuse compiled artifacts
- **Workspace dependencies** - Define once, use everywhere
- **Build ordering** - Cargo understands dependency graph
- **Version synchronization** - Workspace version applies to all crates

**Reference:** [Cargo Workspaces](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html) documentation.

### Build Targets and Release Packaging

**Multi-target matrix:**

| Language | Platforms | Architectures | Package Format |
|----------|-----------|---------------|----------------|
| Python | Linux, macOS, Windows | x86_64, aarch64 | Wheel (`.whl`) |
| Node.js | Linux, macOS, Windows | x86_64, aarch64, arm | npm package (`.tgz`) |
| C# | Windows, Linux, macOS | x86_64, aarch64 | NuGet (`.nupkg`) |

**Recommended build tool:** [cross](https://github.com/cross-rs/cross) for Rust cross-compilation.

**CI/CD strategy:**

```yaml
# .github/workflows/build.yml
name: Build Multi-Language Bindings

on:
  push:
    tags:
      - 'v*'

jobs:
  build-python:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        python: ['3.8', '3.9', '3.10', '3.11', '3.12']
    steps:
      - uses: actions/checkout@v4
      - uses: PyO3/maturin-action@v1
        with:
          working-directory: py
          target: ${{ matrix.target }}
          manylinux: auto
          args: --release --out dist
      - uses: actions/upload-artifact@v4
        with:
          name: wheels-${{ matrix.os }}-${{ matrix.python }}
          path: py/dist

  build-nodejs:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: '20'
      - run: cd js && npm install && npm run build
      - run: cd js && npm pack
      - uses: actions/upload-artifact@v4
        with:
          name: npm-package-${{ matrix.os }}
          path: js/*.tgz

  build-csharp:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-dotnet@v4
        with:
          dotnet-version: '8.0'
      - run: cd uniffi && cargo build --release
      - run: cd uniffi && cargo run --bin uniffi-bindgen generate src/marketdata.udl --language csharp
      # Package as NuGet (requires .csproj setup)
```

**Release checklist:**

1. **Version bump** - Update `workspace.package.version` in root `Cargo.toml`
2. **Build all targets** - Python wheels, npm packages, NuGet packages
3. **Test on all platforms** - Smoke tests for each binding
4. **Publish**:
   - Python: `maturin publish` or `twine upload dist/*`
   - Node.js: `npm publish` from `js/`
   - C#: `dotnet nuget push *.nupkg`
5. **Tag release** - `git tag v0.2.0 && git push origin v0.2.0`
6. **GitHub release** - Attach built artifacts

**Reference:**
- [Temporal's Rust at the Core](https://www.infoq.com/news/2025/11/temporal-rust-polygot-sdk/) - Real-world example of multi-language SDK development with Rust core
- [Spikard](https://github.com/Goldziher/spikard) - Multi-language web toolkit with PyO3, napi-rs, magnus, ext-php-rs

---

## Data Flow Architecture

### Request-Response Flow (REST API)

```
User Code                FFI Boundary          Core Library
─────────                ────────────          ────────────

Python:
client.quote("2330")
    │
    ├─→ PyObject → String conversion
    │
    └─→ Release GIL
            │
            ├─→ PyRestClient.quote()
            │       │
            │       ├─→ spawn tokio task
            │       │
            │       └─→ RestClientInner.quote("2330")
            │               │
            │               ├─→ HTTP GET /stock/intraday/quote?symbol=2330
            │               │
            │               ├─→ JSON parse → Quote struct
            │               │
            │               └─→ return Result<Quote, Error>
            │
            ├─→ Convert Quote → PyDict
            │
            └─→ Reacquire GIL
    │
    ├─→ Return PyDict to Python
    │
    ▼
print(quote)
```

**Key points:**
- **Single crossing** - Data crosses FFI boundary twice (in: String, out: PyDict/JsObject)
- **Serialization** - Complex types serialized at boundary (Rust struct → language object)
- **Error propagation** - Rust Result → language exception

### Streaming Flow (WebSocket)

```
User Code                FFI Boundary          Core Library
─────────                ────────────          ────────────

Python:
async for msg in ws.messages():
    │
    ├─→ MessageIterator.__anext__()
    │       │
    │       ├─→ rx.try_recv()  (tokio channel)
    │       │       │
    │       │       ◄─┤ WebSocket task (spawned once)
    │       │         │
    │       │         ├─→ ws.next().await
    │       │         │       │
    │       │         │       ◄─┤ TCP socket
    │       │         │         │
    │       │         │         └─→ Raw bytes
    │       │         │
    │       │         ├─→ Deserialize JSON → Message
    │       │         │
    │       │         └─→ tx.send(Message)
    │       │
    │       └─→ Convert Message → PyDict
    │
    └─→ Return PyDict
    │
    ▼
process(msg)
```

**Key points:**
- **Persistent connection** - WebSocket task runs in background
- **Channel-based** - tokio mpsc channel decouples WebSocket from iterator
- **Continuous crossing** - Each message crosses FFI boundary (Rust Message → Python dict)
- **Backpressure** - Channel bounded to prevent memory issues

### Callback Flow (Node.js EventEmitter)

```
JavaScript                FFI Boundary          Core Library
──────────                ────────────          ────────────

ws.on("message", callback)
    │
    ├─→ JsFunction → ThreadsafeFunction
    │       │
    │       └─→ Store in StockWebSocketClient
    │
ws.connect()
    │
    ├─→ StockWebSocketClient.connect()
    │       │
    │       └─→ spawn tokio task
    │               │
    │               ├─→ WebSocket connect
    │               │
    │               └─→ loop {
    │                       │
    │                       ├─→ ws.next().await
    │                       │       │
    │                       │       └─→ Message
    │                       │
    │                       ├─→ Serialize Message → String
    │                       │
    │                       └─→ tsfn.call(msg)  ◄─┐
    │                                             │
    ◄───────────────────────────────────────────────┘
    │
    ├─→ JS callback invoked on event loop
    │       │
    │       └─→ callback(msg)
    │
    ▼
process(msg)
```

**Key points:**
- **Callback registration** - JsFunction converted to ThreadsafeFunction (can be called from any thread)
- **Cross-thread invocation** - Rust task calls into JS from tokio thread
- **Event loop integration** - ThreadsafeFunction schedules callback on JS event loop
- **Minimal blocking** - Only callback execution blocks event loop, not I/O

---

## Suggested Build Order

**Phase dependency graph:**

```
Phase 1: Core Library Stabilization
    │
    ├─→ API finalization
    ├─→ Error handling consistency
    ├─→ Performance benchmarks
    └─→ Documentation

Phase 2: Workspace Migration
    │
    ├─→ Create root Cargo.toml with [workspace]
    ├─→ Extract shared dependencies
    ├─→ Update child Cargo.toml with workspace = true
    └─→ Verify builds (cargo build --workspace)

Phase 3: Python Binding Enhancement (Parallel)
    │
    ├─→ Async runtime bridging (pyo3-async-runtimes)
    ├─→ WebSocket iterator pattern
    ├─→ Type stubs generation (.pyi files)
    └─→ Testing (pytest)

Phase 3: Node.js Binding Enhancement (Parallel)
    │
    ├─→ TypeScript definitions refinement
    ├─→ EventEmitter pattern for WebSocket
    ├─→ Memory leak testing
    └─→ Testing (Jest)

Phase 4: C# Binding Development (Sequential, depends on UniFFI stability)
    │
    ├─→ Evaluate UniFFI vs csbindgen vs Interoptopus
    ├─→ Async wrapper (Task.Run pattern)
    ├─→ Event pattern for WebSocket
    ├─→ NuGet packaging
    └─→ Testing (xUnit)

Phase 5: Cross-Platform CI/CD
    │
    ├─→ GitHub Actions matrix builds
    ├─→ Cross-compilation setup
    ├─→ Release automation
    └─→ Documentation generation

Phase 6: Performance Optimization (Cross-cutting)
    │
    ├─→ FFI call reduction (batching)
    ├─→ Serialization optimization
    ├─→ Memory profiling
    └─→ Async runtime tuning
```

**Critical path:**
1. Workspace migration (1 week) - Blocks all parallel work
2. Core stabilization (2 weeks) - Blocks binding improvements
3. C# binding (3 weeks) - Sequential, requires UniFFI decisions
4. CI/CD (1 week) - Can start once first binding is complete

**Parallelization opportunities:**
- Python and Node.js enhancements can happen concurrently
- Documentation can be written alongside implementation
- Performance testing can run in parallel across languages

---

## Architecture Anti-Patterns to Avoid

### 1. Leaking Rust Types Across FFI

**❌ Bad:**

```rust
#[napi]
pub struct Quote {
    pub price: f64,
    pub volume: i64,
    // ... 20 more fields
}
```

**Problem:** Changes to Rust struct require changes to all language bindings.

**✅ Good:**

```rust
// Core library
pub struct Quote { /* ... */ }

// FFI layer
impl Quote {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

// Language binding
#[napi]
pub fn get_quote(symbol: String) -> Result<String> {
    let quote = core_client.quote(&symbol)?;
    Ok(quote.to_json())
}
```

### 2. Synchronous FFI Calls in Async Context

**❌ Bad:**

```python
# This blocks the entire event loop!
async def get_quote(symbol):
    return client.quote_sync(symbol)  # Blocks 100-500ms
```

**Problem:** Synchronous FFI calls block the async runtime, preventing other tasks from running.

**✅ Good:**

```python
# Run FFI call on thread pool
async def get_quote(symbol):
    loop = asyncio.get_event_loop()
    return await loop.run_in_executor(None, client.quote_sync, symbol)
```

Or use async FFI bridge (pyo3-asyncio, napi-rs async).

### 3. Unbounded WebSocket Message Buffers

**❌ Bad:**

```rust
let (tx, rx) = mpsc::unbounded_channel();  // No backpressure!

tokio::spawn(async move {
    while let Some(msg) = ws.next().await {
        tx.send(msg).unwrap();  // Accumulates infinitely if consumer is slow
    }
});
```

**Problem:** Fast WebSocket + slow consumer = memory exhaustion.

**✅ Good:**

```rust
let (tx, rx) = mpsc::channel(100);  // Bounded channel with capacity

tokio::spawn(async move {
    while let Some(msg) = ws.next().await {
        if tx.send(msg).await.is_err() {
            break;  // Consumer dropped, exit task
        }
    }
});
```

### 4. Mixing Async Runtimes

**❌ Bad:**

```rust
// Core uses tokio
impl RestClient {
    pub async fn quote(&self) -> Result<Quote> { /* tokio */ }
}

// Binding uses async-std
#[napi]
impl RestClient {
    pub async fn quote(&self) -> Result<String> {
        async_std::task::block_on(self.inner.quote())  // Runtime conflict!
    }
}
```

**Problem:** Nested runtimes cause deadlocks and panics.

**✅ Good:**

```rust
// Use the same runtime everywhere (tokio)
// napi-rs with tokio_rt feature shares tokio runtime
#[napi]
impl RestClient {
    pub async fn quote(&self) -> Result<String> {
        self.inner.quote().await  // Same tokio runtime
    }
}
```

### 5. Not Handling Language-Specific Lifecycle

**❌ Bad:**

```rust
#[pyclass]
pub struct WebSocketClient {
    ws: WebSocket,  // Dropped when Python object GC'd
}

// No __del__ method, WebSocket connection leaks!
```

**Problem:** Language GC != Rust Drop. WebSocket may not close properly.

**✅ Good:**

```rust
#[pyclass]
pub struct WebSocketClient {
    ws: Arc<Mutex<Option<WebSocket>>>,
}

#[pymethods]
impl WebSocketClient {
    fn close(&mut self) -> PyResult<()> {
        if let Some(ws) = self.ws.lock().unwrap().take() {
            ws.close().await;
        }
        Ok(())
    }

    fn __del__(&mut self) {
        // Ensure cleanup on GC
        let _ = self.close();
    }
}
```

---

## Performance Considerations

### FFI Call Overhead

**Measurements:**

| Operation | Overhead | Frequency | Total Impact |
|-----------|----------|-----------|--------------|
| FFI call (no-op) | ~100-500ns | Per API call | Low (1-2% for 200ms API) |
| String copy (10KB) | ~5µs | Per API call | Low (2-3%) |
| JSON serialize (1KB) | ~10-50µs | Per message | Medium (5-10%) |
| GIL acquire/release | ~1-10µs | Per async call | Low (Python-specific) |
| ThreadsafeFunction call | ~10-50µs | Per callback | Medium (high-freq streams) |

**Optimization strategies:**

1. **Batch operations** - Reduce FFI crossings
   ```rust
   // ❌ Bad: N FFI calls
   for symbol in symbols:
       client.quote(symbol)

   // ✅ Good: 1 FFI call
   client.quote_batch(symbols)
   ```

2. **Zero-copy serialization** - Use binary formats (MessagePack) instead of JSON for large payloads
   ```rust
   // JSON: ~50µs for 1KB
   serde_json::to_string(&data)

   // MessagePack: ~10µs for 1KB
   rmp_serde::to_vec(&data)
   ```

3. **Async batching** - Buffer messages before crossing FFI
   ```rust
   let mut buffer = Vec::with_capacity(100);

   while let Some(msg) = ws.next().await {
       buffer.push(msg);

       if buffer.len() >= 100 || elapsed > Duration::from_millis(100) {
           // Cross FFI once for 100 messages
           callback(serde_json::to_string(&buffer));
           buffer.clear();
       }
   }
   ```

**Reference:** [Effective Rust Item 34: Control what crosses FFI boundaries](https://www.effective-rust.com/ffi.html)

---

## Summary

### Component Boundaries (What Talks to What)

```
┌─────────────────────────────────────────────────────────┐
│ User Application Layer (Python/JS/C#)                  │
│ - Idiomatic API                                         │
│ - Documentation                                         │
│ - Convenience wrappers                                  │
└─────────────────────────────────────────────────────────┘
                         ▲
                         │ Language-specific API
                         ▼
┌─────────────────────────────────────────────────────────┐
│ FFI Binding Layer (py/js/uniffi crates)                │
│ - Type conversion (Rust ↔ Python/JS/C#)                │
│ - Async runtime bridging                                │
│ - Error mapping                                         │
│ - Lifetime management                                   │
└─────────────────────────────────────────────────────────┘
                         ▲
                         │ C ABI (FFI boundary)
                         ▼
┌─────────────────────────────────────────────────────────┐
│ Core Library (marketdata-core)                         │
│ - Business logic                                        │
│ - API protocol                                          │
│ - tokio async operations                                │
│ - Pure Rust types                                       │
└─────────────────────────────────────────────────────────┘
                         ▲
                         │ HTTP/WebSocket
                         ▼
┌─────────────────────────────────────────────────────────┐
│ Fugle API (external)                                    │
└─────────────────────────────────────────────────────────┘
```

### Data Flow Direction

1. **Request flow:** User → Language layer → FFI bridge (type conversion) → Core (business logic) → API
2. **Response flow:** API → Core (parse JSON) → FFI bridge (serialize to language type) → Language layer → User
3. **Stream flow:** API → Core (WebSocket task) → tokio channel → FFI bridge (callback/iterator) → Language layer → User

### Build Order Dependencies

```
1. Core stabilization (blocks all)
    ↓
2. Workspace migration (blocks parallel work)
    ↓
3a. Python enhancement ────┐
3b. Node.js enhancement ───┤ (parallel)
3c. C# development ────────┤ (sequential after 3a/3b if using UniFFI)
    ↓
4. CI/CD setup (requires 3a-3c)
    ↓
5. Performance tuning (continuous)
```

---

## Confidence Assessment

| Area | Confidence | Source | Notes |
|------|------------|--------|-------|
| **PyO3 patterns** | HIGH | Official docs, pyo3-async-runtimes | Well-established, production-ready |
| **napi-rs patterns** | HIGH | Official docs, napi.rs | Mature ecosystem, active development |
| **UniFFI for C#** | MEDIUM | Mozilla docs, community reports | Experimental, limited C# examples |
| **Async bridging** | HIGH | pyo3-async-runtimes, NAPI-RS tokio_rt | Documented approaches |
| **Build systems** | HIGH | Cargo workspaces docs, real projects | Standard Rust practice |
| **Performance numbers** | MEDIUM | Various benchmarks | Context-dependent, need validation |
| **C# alternatives** | LOW | Web search only | Need deeper evaluation (csbindgen vs Interoptopus vs UniFFI) |

---

## Sources

**FFI and Multi-Language Bindings:**
- [Mozilla UniFFI](https://github.com/mozilla/uniffi-rs) - Multi-language bindings generator
- [PyO3](https://github.com/PyO3/pyo3) - Rust bindings for Python
- [NAPI-RS](https://napi.rs/) - Node.js bindings framework
- [Effective Rust Item 34: Control what crosses FFI boundaries](https://www.effective-rust.com/ffi.html)
- [Maturin User Guide](https://www.maturin.rs/bindings.html) - PyO3/UniFFI binding comparison

**Async Runtime Bridging:**
- [PyO3 async-runtimes](https://github.com/PyO3/pyo3-async-runtimes) - Bridges between Python and Rust async runtimes
- [NAPI-RS async functions](https://napi.rs/docs/concepts/async-fn) - Tokio runtime integration
- [Tokio bridging guide](https://tokio.rs/tokio/topics/bridging) - Bridging sync and async code
- [pyo3-asyncio crate](https://crates.io/crates/pyo3-asyncio) - Python/Rust async interop

**C# Bindings:**
- [csbindgen](https://github.com/Cysharp/csbindgen) - Generate C# FFI from Rust
- [Calling Rust from C#](https://www.strathweb.com/2023/06/calling-rust-code-from-csharp/) - Strathweb article
- [Rust at Datalust](https://blog.datalust.co/rust-at-datalust-how-we-integrate-rust-with-csharp/) - C# integration case study
- [Interoptopus C# backend](https://docs.rs/interoptopus_backend_csharp/)

**Architecture and Build Systems:**
- [Cargo Workspaces](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html) - Official Rust book
- [Cargo Workspaces reference](https://doc.rust-lang.org/cargo/reference/workspaces.html)
- [Temporal Rust Core SDK](https://www.infoq.com/news/2025/11/temporal-rust-polygot-sdk/) - Multi-language SDK architecture
- [Spikard](https://github.com/Goldziher/spikard) - Multi-language web toolkit example
- [Monorepo build systems](https://monorepo.tools/) - Comparison of build tools

**Performance and Serialization:**
- [Data Serialization Comparison](https://www.sitepoint.com/data-serialization-comparison-json-yaml-bson-messagepack/)
- [JSON vs Protobuf vs FlatBuffers benchmarks](https://medium.com/@harshiljani2002/benchmarking-data-serialization-json-vs-protobuf-vs-flatbuffers-3218eecdba77)
- [ORJSON vs JSON performance](https://onchana01.medium.com/orjson-vs-json-choosing-the-right-serializer-for-high-performance-api-development-f7e4db123d18)

**Project Examples:**
- [BridgeRust](https://dev.to/josias1997/bridgerust-one-rust-core-every-ecosystem-5bi1) - Unified multi-language framework
- [Multi-language SDK management](https://medium.com/@parserdigital/how-to-manage-multi-language-open-source-sdks-on-githug-best-practices-tools-1a401b22544e)
