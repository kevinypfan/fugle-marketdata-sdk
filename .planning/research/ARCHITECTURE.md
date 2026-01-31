# Configuration Integration Architecture

**Project:** fugle-marketdata-sdk v0.3.0
**Researched:** 2026-02-01
**Confidence:** HIGH

## Executive Summary

This document describes how to wire configuration options (reconnection, health check, REST timeouts) from language bindings through FFI boundaries into the Rust core. The SDK has a well-established pattern: **Core owns config structs, bindings convert from idiomatic types to core types.**

**Key Finding:** The architecture already supports configuration constructors (`with_reconnection_config`, `with_full_config`). The implementation task is **not adding new architecture patterns**, but **exposing existing config constructors through each FFI layer**.

## Current State Assessment

### Core Layer (Already Complete)

**Location:** `core/src/websocket/`

The Rust core already has complete configuration infrastructure:

```rust
// core/src/websocket/reconnection.rs
pub struct ReconnectionConfig {
    pub max_attempts: u32,           // Default: 5
    pub initial_delay: Duration,     // Default: 1s
    pub max_delay: Duration,         // Default: 60s
}

impl ReconnectionConfig {
    pub fn with_max_attempts(mut self, max_attempts: u32) -> Self { ... }
    pub fn with_initial_delay(mut self, initial_delay: Duration) -> Self { ... }
    pub fn with_max_delay(mut self, max_delay: Duration) -> Self { ... }
}

// core/src/websocket/health_check.rs
pub struct HealthCheckConfig {
    pub enabled: bool,                // Default: true
    pub interval: Duration,           // Default: 30s
    pub max_missed_pongs: u64,        // Default: 2
}

impl HealthCheckConfig {
    pub fn with_interval(mut self, interval: Duration) -> Self { ... }
    pub fn with_max_missed_pongs(mut self, max: u64) -> Self { ... }
    pub fn with_enabled(mut self, enabled: bool) -> Self { ... }
}

// core/src/websocket/connection.rs (lines 95-147)
impl WebSocketClient {
    pub fn new(config: ConnectionConfig) -> Self {
        Self::with_reconnection_config(config, ReconnectionConfig::default())
    }

    pub fn with_reconnection_config(
        config: ConnectionConfig,
        reconnection_config: ReconnectionConfig,
    ) -> Self { ... }

    pub fn with_health_check_config(
        config: ConnectionConfig,
        health_check_config: HealthCheckConfig,
    ) -> Self { ... }

    pub fn with_full_config(
        config: ConnectionConfig,
        reconnection_config: ReconnectionConfig,
        health_check_config: HealthCheckConfig,
    ) -> Self { ... }
}
```

**Status:** Core configuration is production-ready. No changes needed.

### Binding Layers (Need Wiring)

#### Python Binding (PyO3)

**Current State:** Has `ReconnectConfig` struct but not wired to core

**Location:** `py/src/websocket.rs` (lines 37-107)

```rust
#[pyclass]
#[derive(Clone)]
pub struct ReconnectConfig {
    #[pyo3(get, set)]
    pub enabled: bool,
    #[pyo3(get, set)]
    pub max_retries: u32,
    #[pyo3(get, set)]
    pub base_delay_ms: u64,
    #[pyo3(get, set)]
    pub max_delay_ms: u64,
}
```

**Gap:** This `ReconnectConfig` exists in Python binding but is never converted to core's `ReconnectionConfig`. The `StockWebSocketClient::new()` creates core `WebSocketClient` with defaults only (line 324).

**What's Missing:**
- No health check config exposed
- `ReconnectConfig` not passed to core
- Constructor doesn't accept config parameters

#### Node.js Binding (napi-rs)

**Current State:** No config exposed

**Location:** `js/src/websocket.rs`

**Gap:** The binding creates core `WebSocketClient` with default configs (lines 224-225):

```rust
let config = ConnectionConfig::fugle_stock(AuthRequest::with_api_key(&api_key));
let client = CoreClient::new(config);  // Uses defaults
```

**What's Missing:**
- No config structs exposed to JavaScript
- No way to pass reconnection or health check options
- Hard-coded to defaults

#### UniFFI Binding

**Current State:** No config exposed

**Location:** `uniffi/src/websocket.rs`

**Gap:** Creates core `WebSocketClient` with defaults (lines 162-168):

```rust
let config = match self.endpoint {
    WebSocketEndpoint::Stock => ConnectionConfig::fugle_stock(auth),
    WebSocketEndpoint::FutOpt => ConnectionConfig::fugle_futopt(auth),
};
let core_ws = CoreWebSocketClient::new(config);  // Uses defaults
```

**What's Missing:**
- No configuration options exposed to C#/Java/Go
- Constructor doesn't accept config
- Hard-coded to defaults

## Reference Implementation: Official SDKs

### Python Official SDK Pattern

**Location:** `/Users/zackfan/Project/fugle/fugle-marketdata-python/fugle_marketdata/websocket/client.py`

```python
class HealthCheckConfig:
    def __init__(self, enabled: bool = False, ping_interval: int = 30000, max_missed_pongs: int = 2):
        self.enabled = enabled
        self.ping_interval = ping_interval
        self.max_missed_pongs = max_missed_pongs

class WebSocketClient():
    def __init__(self, **config):
        self.config = config
        self.health_check: Optional[HealthCheckConfig] = config.get('health_check')
```

**Usage:**
```python
ws = WebSocketClient(
    api_key='my-key',
    health_check=HealthCheckConfig(enabled=True, ping_interval=30000)
)
```

### Node.js Official SDK Pattern

**Location:** `/Users/zackfan/Project/fugle/fugle-marketdata-node/src/websocket/client.ts`

```typescript
export interface HealthCheckConfig {
  enabled: boolean;
  pingInterval?: number;      // milliseconds
  maxMissedPongs?: number;
}

export interface WebSocketClientOptions {
  url: string;
  apiKey?: string;
  bearerToken?: string;
  sdkToken?: string;
  healthCheck?: HealthCheckConfig;
}

export class WebSocketClient extends events.EventEmitter {
  constructor(protected readonly options: WebSocketClientOptions) {
    super();
  }
}
```

**Usage:**
```typescript
const ws = new WebSocketClient({
  url: 'wss://...',
  apiKey: 'my-key',
  healthCheck: {
    enabled: true,
    pingInterval: 30000,
    maxMissedPongs: 2
  }
});
```

**Pattern:** Options object with nested config structs.

## Recommended Architecture

### Design Principles

1. **Core owns structs** — Config definitions live in `core/`, bindings convert to them
2. **Builder pattern in core** — `ReconnectionConfig::default().with_max_attempts(10)`
3. **Options object in bindings** — Match official SDK patterns for API compatibility
4. **Duration as milliseconds at FFI** — Avoid complex Duration serialization, use `u64` milliseconds
5. **Defaults match official SDKs** — Ensure same behavior as reference implementations

### Layer-by-Layer Strategy

#### Layer 1: Core (No Changes)

**Status:** Complete ✓

The core already has:
- `ReconnectionConfig` with builder methods
- `HealthCheckConfig` with builder methods
- `WebSocketClient::with_full_config()` constructor
- All defaults match requirements

#### Layer 2: Python Binding (PyO3)

**Approach:** Options object constructor pattern

**New Types Needed:**

```rust
// py/src/websocket.rs

#[pyclass]
#[derive(Clone)]
pub struct ReconnectConfig {
    #[pyo3(get, set)]
    pub max_attempts: u32,
    #[pyo3(get, set)]
    pub initial_delay_ms: u64,
    #[pyo3(get, set)]
    pub max_delay_ms: u64,
}

#[pymethods]
impl ReconnectConfig {
    #[new]
    #[pyo3(signature = (max_attempts=5, initial_delay_ms=1000, max_delay_ms=60000))]
    pub fn new(max_attempts: u32, initial_delay_ms: u64, max_delay_ms: u64) -> Self { ... }
}

#[pyclass]
#[derive(Clone)]
pub struct HealthCheckConfig {
    #[pyo3(get, set)]
    pub enabled: bool,
    #[pyo3(get, set)]
    pub interval_ms: u64,
    #[pyo3(get, set)]
    pub max_missed_pongs: u64,
}

#[pymethods]
impl HealthCheckConfig {
    #[new]
    #[pyo3(signature = (enabled=true, interval_ms=30000, max_missed_pongs=2))]
    pub fn new(enabled: bool, interval_ms: u64, max_missed_pongs: u64) -> Self { ... }
}
```

**Constructor Modification:**

```rust
#[pymethods]
impl StockWebSocketClient {
    // Existing: new(api_key: String)
    // Add: new_with_config
    #[new]
    #[pyo3(signature = (api_key, reconnect=None, health_check=None))]
    pub fn new(
        api_key: String,
        reconnect: Option<ReconnectConfig>,
        health_check: Option<HealthCheckConfig>,
    ) -> Self {
        // Convert to core types
        let core_reconnect = reconnect
            .map(|r| marketdata_core::websocket::ReconnectionConfig {
                max_attempts: r.max_attempts,
                initial_delay: Duration::from_millis(r.initial_delay_ms),
                max_delay: Duration::from_millis(r.max_delay_ms),
            })
            .unwrap_or_default();

        let core_health = health_check
            .map(|h| marketdata_core::websocket::HealthCheckConfig {
                enabled: h.enabled,
                interval: Duration::from_millis(h.interval_ms),
                max_missed_pongs: h.max_missed_pongs,
            })
            .unwrap_or_default();

        // Store configs for use in connect()
        // ...
    }
}
```

**Connect Flow:**

When `connect()` is called, use stored configs to create core `WebSocketClient`:

```rust
pub fn connect(&self, py: Python<'_>) -> PyResult<()> {
    let auth = marketdata_core::AuthRequest::with_api_key(&self.api_key);
    let config = marketdata_core::ConnectionConfig::fugle_stock(auth);

    // Use with_full_config instead of new()
    let ws_client = marketdata_core::WebSocketClient::with_full_config(
        config,
        self.reconnect_config.clone(),  // stored from constructor
        self.health_config.clone(),     // stored from constructor
    );

    // ... rest of connect logic
}
```

**Python Usage (API Compatible):**

```python
from marketdata_py import WebSocketClient, ReconnectConfig, HealthCheckConfig

# Default
ws = WebSocketClient("my-api-key")

# Custom config
ws = WebSocketClient(
    "my-api-key",
    reconnect=ReconnectConfig(max_attempts=10, initial_delay_ms=2000),
    health_check=HealthCheckConfig(enabled=True, interval_ms=30000)
)
```

#### Layer 3: Node.js Binding (napi-rs)

**Approach:** TypeScript options object

**New Types Needed:**

```rust
// js/src/websocket.rs

#[napi(object)]
pub struct ReconnectOptions {
    pub max_attempts: Option<u32>,
    pub initial_delay_ms: Option<u32>,
    pub max_delay_ms: Option<u32>,
}

#[napi(object)]
pub struct HealthCheckOptions {
    pub enabled: Option<bool>,
    pub interval_ms: Option<u32>,
    pub max_missed_pongs: Option<u32>,
}

#[napi(object)]
pub struct WebSocketOptions {
    pub reconnect: Option<ReconnectOptions>,
    pub health_check: Option<HealthCheckOptions>,
}
```

**Constructor Modification:**

```rust
#[napi]
impl StockWebSocketClient {
    // Add optional options parameter
    fn new(api_key: String, options: Option<WebSocketOptions>) -> Self {
        let opts = options.unwrap_or_default();

        // Convert to core types
        let reconnect_config = if let Some(r) = opts.reconnect {
            marketdata_core::websocket::ReconnectionConfig::default()
                .with_max_attempts(r.max_attempts.unwrap_or(5))
                .with_initial_delay(Duration::from_millis(r.initial_delay_ms.unwrap_or(1000) as u64))
                .with_max_delay(Duration::from_millis(r.max_delay_ms.unwrap_or(60000) as u64))
        } else {
            marketdata_core::websocket::ReconnectionConfig::default()
        };

        // Similar for health_check_config
        // Store for use in connect()
    }
}
```

**TypeScript Usage (API Compatible):**

```typescript
import { WebSocketClient } from '@fubon/marketdata-js';

// Default
const ws = new WebSocketClient('my-api-key');

// Custom config
const ws = new WebSocketClient('my-api-key', {
  reconnect: {
    maxAttempts: 10,
    initialDelayMs: 2000,
    maxDelayMs: 120000
  },
  healthCheck: {
    enabled: true,
    intervalMs: 30000,
    maxMissedPongs: 2
  }
});

const client = ws.stock;
await client.connect();
```

#### Layer 4: UniFFI Binding (C#/Java/Go)

**Approach:** Record structs with optional constructors

**UniFFI Types:**

```rust
// uniffi/src/websocket.rs

#[derive(uniffi::Record)]
pub struct ReconnectConfig {
    pub max_attempts: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            initial_delay_ms: 1000,
            max_delay_ms: 60000,
        }
    }
}

#[derive(uniffi::Record)]
pub struct HealthCheckConfig {
    pub enabled: bool,
    pub interval_ms: u64,
    pub max_missed_pongs: u64,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_ms: 30000,
            max_missed_pongs: 2,
        }
    }
}
```

**Constructor with Optional Configs:**

```rust
#[uniffi::export]
impl WebSocketClient {
    #[uniffi::constructor]
    pub fn new_with_config(
        api_key: String,
        listener: Arc<dyn WebSocketListener>,
        endpoint: WebSocketEndpoint,
        reconnect: Option<ReconnectConfig>,
        health_check: Option<HealthCheckConfig>,
    ) -> Arc<Self> {
        // Convert to core types
        let core_reconnect = reconnect
            .map(|r| marketdata_core::websocket::ReconnectionConfig {
                max_attempts: r.max_attempts,
                initial_delay: Duration::from_millis(r.initial_delay_ms),
                max_delay: Duration::from_millis(r.max_delay_ms),
            })
            .unwrap_or_default();

        let core_health = health_check
            .map(|h| marketdata_core::websocket::HealthCheckConfig {
                enabled: h.enabled,
                interval: Duration::from_millis(h.interval_ms),
                max_missed_pongs: h.max_missed_pongs,
            })
            .unwrap_or_default();

        // Store and use in connect()
        Arc::new(Self {
            reconnect_config: core_reconnect,
            health_config: core_health,
            // ... rest
        })
    }
}
```

**C# Usage:**

```csharp
using Fugle.MarketData;

// Default
var ws = WebSocketClient.New(apiKey, listener);

// Custom config
var ws = WebSocketClient.NewWithConfig(
    apiKey,
    listener,
    WebSocketEndpoint.Stock,
    new ReconnectConfig {
        MaxAttempts = 10,
        InitialDelayMs = 2000,
        MaxDelayMs = 120000
    },
    new HealthCheckConfig {
        Enabled = true,
        IntervalMs = 30000,
        MaxMissedPongs = 2
    }
);

await ws.ConnectAsync();
```

**Java Usage:**

```java
import com.fubon.marketdata.*;

// Default
WebSocketClient ws = WebSocketClient.new(apiKey, listener);

// Custom config
ReconnectConfig reconnect = new ReconnectConfig(10, 2000, 120000);
HealthCheckConfig health = new HealthCheckConfig(true, 30000, 2);
WebSocketClient ws = WebSocketClient.newWithConfig(
    apiKey,
    listener,
    WebSocketEndpoint.STOCK,
    reconnect,
    health
);

ws.connect();
```

**Go Usage:**

```go
import "github.com/fubon/marketdata-go"

// Default
ws := marketdata.NewWebSocketClient(apiKey, listener)

// Custom config
ws := marketdata.NewWebSocketClientWithConfig(
    apiKey,
    listener,
    marketdata.WebSocketEndpointStock,
    &marketdata.ReconnectConfig{
        MaxAttempts:     10,
        InitialDelayMs:  2000,
        MaxDelayMs:      120000,
    },
    &marketdata.HealthCheckConfig{
        Enabled:         true,
        IntervalMs:      30000,
        MaxMissedPongs:  2,
    },
)

ws.Connect()
```

## FFI Boundary Considerations

### Duration Type Handling

**Challenge:** Rust's `std::time::Duration` is not FFI-safe and doesn't serialize well.

**Solution:** Use `u64` milliseconds at all FFI boundaries.

**Conversion Pattern:**

```rust
// Binding → Core
let core_duration = Duration::from_millis(binding_millis);

// Core → Binding (if needed for getters)
let binding_millis = core_duration.as_millis() as u64;
```

**Applied to:**
- `ReconnectionConfig::initial_delay` → `initial_delay_ms: u64`
- `ReconnectionConfig::max_delay` → `max_delay_ms: u64`
- `HealthCheckConfig::interval` → `interval_ms: u64`

### Optional Parameters

**PyO3 Pattern:**

```rust
#[pyo3(signature = (api_key, reconnect=None, health_check=None))]
pub fn new(
    api_key: String,
    reconnect: Option<ReconnectConfig>,
    health_check: Option<HealthCheckConfig>,
) -> Self
```

**napi-rs Pattern:**

```rust
fn new(api_key: String, options: Option<WebSocketOptions>) -> Self
```

**UniFFI Pattern:**

```rust
pub fn new_with_config(
    api_key: String,
    reconnect: Option<ReconnectConfig>,
    health_check: Option<HealthCheckConfig>,
) -> Arc<Self>
```

**Common Pattern:** Use `Option<T>` in Rust, convert to language-specific optionals (Python `None`, TypeScript `undefined`, C# `null`, Java `null`, Go `nil`).

### Config Storage in Binding Layer

**Challenge:** Core `WebSocketClient` is created in `connect()`, not constructor.

**Solution:** Store config in binding struct, use when creating core client.

**Pattern:**

```rust
pub struct StockWebSocketClient {
    api_key: String,
    reconnect_config: ReconnectionConfig,  // Store here
    health_config: HealthCheckConfig,      // Store here
    state: Arc<Mutex<Option<WebSocketState>>>,
    // ...
}

impl StockWebSocketClient {
    pub fn new(api_key: String, reconnect: Option<...>, health: Option<...>) -> Self {
        Self {
            api_key,
            reconnect_config: convert_or_default(reconnect),
            health_config: convert_or_default(health),
            state: Arc::new(Mutex::new(None)),
        }
    }

    pub fn connect(&self) -> Result<()> {
        let ws_client = marketdata_core::WebSocketClient::with_full_config(
            config,
            self.reconnect_config.clone(),
            self.health_config.clone(),
        );
        // ...
    }
}
```

### REST Client Configuration (Future)

**Current State:** REST client uses hardcoded timeouts in core:

```rust
// core/src/rest/client.rs (lines 37-40)
let agent = ureq::AgentBuilder::new()
    .timeout_read(std::time::Duration::from_secs(30))
    .timeout_write(std::time::Duration::from_secs(30))
    .build();
```

**Recommended Addition:**

```rust
// core/src/rest/client.rs
pub struct RestClientConfig {
    pub timeout_read: Duration,
    pub timeout_write: Duration,
}

impl Default for RestClientConfig {
    fn default() -> Self {
        Self {
            timeout_read: Duration::from_secs(30),
            timeout_write: Duration::from_secs(30),
        }
    }
}

impl RestClient {
    pub fn with_config(auth: Auth, config: RestClientConfig) -> Self {
        let agent = ureq::AgentBuilder::new()
            .timeout_read(config.timeout_read)
            .timeout_write(config.timeout_write)
            .build();
        // ...
    }
}
```

**Bindings follow same pattern:** Options object with `timeout_read_ms` and `timeout_write_ms`.

## Implementation Order

### Phase 1: Core Extensions (if REST config needed)

1. Add `RestClientConfig` to `core/src/rest/client.rs`
2. Add `RestClient::with_config()` constructor
3. Add tests for custom timeouts

**Effort:** 1-2 hours (optional for v0.3.0)

### Phase 2: Python Binding

1. Update `ReconnectConfig` to match core field names
2. Add `HealthCheckConfig` PyClass
3. Modify `StockWebSocketClient::new()` to accept optional configs
4. Store configs in struct
5. Use `with_full_config()` in `connect()`
6. Add tests with custom configs

**Files to modify:**
- `py/src/websocket.rs` (config structs + constructor)
- `py/tests/test_websocket.py` (test custom configs)

**Effort:** 4-6 hours

### Phase 3: Node.js Binding

1. Add `ReconnectOptions`, `HealthCheckOptions`, `WebSocketOptions` napi objects
2. Modify `StockWebSocketClient::new()` to accept options
3. Convert options to core types and store
4. Use `with_full_config()` in `connect()`
5. Update TypeScript types in `js/index.d.ts`
6. Add tests with custom configs

**Files to modify:**
- `js/src/websocket.rs` (config objects + constructor)
- `js/index.d.ts` (TypeScript types)
- `js/tests/websocket.test.js` (test custom configs)

**Effort:** 4-6 hours

### Phase 4: UniFFI Binding

1. Add `ReconnectConfig` and `HealthCheckConfig` Records
2. Add `new_with_config()` constructor
3. Modify `connect()` to use stored configs with `with_full_config()`
4. Test in C# wrapper
5. Test in Java wrapper
6. Test in Go wrapper

**Files to modify:**
- `uniffi/src/websocket.rs` (config records + constructor)
- `dotnet/` (C# wrapper with config)
- `java/` (Java wrapper with config)
- `go/` (Go wrapper with config)

**Effort:** 6-8 hours (testing across 3 languages)

### Phase 5: Documentation

1. Update README with config examples for each language
2. Add config reference documentation
3. Document default values
4. Add migration guide (defaults → custom)

**Files to modify:**
- `README.md` (usage examples)
- `.planning/docs/CONFIGURATION.md` (reference)
- API docs for each binding

**Effort:** 2-3 hours

**Total Estimated Effort:** 17-25 hours

## Validation Approach

### Unit Tests

Each binding layer should test:
1. Default config (matches current behavior)
2. Custom reconnect config (verify core receives correct values)
3. Custom health check config (verify core receives correct values)
4. Full custom config (both reconnect + health check)
5. Invalid configs (negative delays, zero attempts)

### Integration Tests

Test actual behavior:
1. Reconnection triggers with custom max_attempts
2. Health check pings at custom intervals
3. Missed pongs trigger disconnect at custom threshold

### Compatibility Tests

Verify API compatibility:
1. Python usage matches reference SDK patterns
2. Node.js usage matches reference SDK patterns
3. C#/Java/Go usage follows platform conventions

## Risk Assessment

### Low Risk

- **Core changes:** None required, only using existing constructors
- **Breaking changes:** None, all changes are additive (optional parameters)
- **FFI complexity:** Standard Duration → milliseconds conversion

### Medium Risk

- **Type mismatches:** Must carefully convert binding types to core types
- **Default behavior:** Must preserve existing defaults exactly
- **Testing burden:** Need to test across 5 language bindings

### Mitigation

- **Preserve defaults:** Use `Option<T>` everywhere, unwrap to current defaults
- **Comprehensive tests:** Unit + integration + compatibility tests
- **Incremental rollout:** Python → Node.js → UniFFI
- **Documentation:** Clear examples for each language

## Open Questions

1. **REST client config priority:** Should v0.3.0 include REST timeout config or defer to v0.3.1?
   - **Recommendation:** Include in v0.3.0 for completeness, minimal effort

2. **Reconnection disable:** Should `max_attempts: 0` disable reconnection entirely?
   - **Recommendation:** Yes, document as disable mechanism

3. **Health check defaults:** Official Python SDK has `enabled: False` by default, but our core has `enabled: true`. Which to follow?
   - **Recommendation:** Match official SDKs (`enabled: false` default) for API compatibility, update core default

4. **Config validation:** Should bindings validate configs (e.g., `interval > 0`) or let core handle it?
   - **Recommendation:** Bindings validate for better error messages, core validates as defense

## Conclusion

Configuration integration is straightforward: the core already supports it, we just need to expose it through each binding layer. The architecture follows a clear pattern:

1. **Core owns config structs** (already complete)
2. **Bindings convert to core types** (Duration → milliseconds)
3. **Constructors accept optional configs** (preserve backward compatibility)
4. **Connect uses stored configs** (call `with_full_config()`)

**No new architecture patterns needed.** Implementation is wiring, not design.

## Sources

- [PyO3 Rust-Python FFI Documentation](https://pyo3.rs/)
- [PyO3 - Rust](https://docs.rs/pyo3/latest/pyo3/)
- [GitHub - PyO3/pyo3: Rust bindings for the Python interpreter](https://github.com/PyO3/pyo3)
- [NAPI-RS Documentation](https://napi.rs/)
- [Types Overwrite – NAPI-RS](https://napi.rs/docs/concepts/types-overwrite)
- [GitHub - napi-rs/napi-rs: A framework for building compiled Node.js add-ons in Rust via Node-API](https://github.com/napi-rs/napi-rs)
- [GitHub - mozilla/uniffi-rs: a multi-language bindings generator for rust](https://github.com/mozilla/uniffi-rs)
- [UniFFI — Rust utility // Lib.rs](https://lib.rs/crates/uniffi)
- [The UniFFI user guide - FFI converter traits](https://mozilla.github.io/uniffi-rs/latest/internals/ffi_converter_traits.html)

---

*Architecture research: 2026-02-01*
*Confidence: HIGH - Based on existing codebase patterns and official SDK analysis*
