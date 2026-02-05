# Phase 12: Python Config Exposure - Research

**Researched:** 2026-02-05
**Domain:** PyO3 FFI bindings, Python constructor patterns, config validation
**Confidence:** HIGH

## Summary

This phase adds options-based constructor and configuration exposure to the Python binding, completing the v0.3.0 API compatibility goal. The core domain involves PyO3 0.27 constructor patterns, kwargs handling, Python exception conventions, and FFI-safe configuration passing.

Phase 9 delivered the async foundation (PyO3 0.27, `future_into_py`, GIL-safe patterns) and a partial `ReconnectConfig` class. Phase 8 delivered comprehensive validation in core's `ReconnectionConfig` and `HealthCheckConfig`. This phase connects them through kwargs-based constructors matching the official SDK patterns.

The official `fugle-marketdata-python` SDK uses kwargs-based constructors (`api_key`, `bearer_token`, `sdk_token`, `base_url`) with exactly-one-auth validation. The `HealthCheckConfig` class uses `ping_interval` (ms) matching the time-unit-in-milliseconds decision from v0.3.0 roadmap.

**Primary recommendation:** Wire PyO3 kwargs constructors to core's validated configs via millisecond conversions, failing fast with `ValueError` at construction time per Python conventions.

## Standard Stack

The established libraries/tools for PyO3 FFI binding configuration:

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| PyO3 | 0.27+ | Rust-Python FFI | Already used in Phase 9, modern async support |
| pyo3-async-runtimes | tokio feature | Async bridge | `future_into_py` pattern for GIL release |
| marketdata-core | local | Core SDK logic | Phase 8 validation logic already complete |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| tokio | 1.x | Async runtime | Already used for WebSocket operations |
| std::time::Duration | stdlib | Time representation | Convert from ms at FFI boundary |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| PyO3 0.27 | PyO3 0.22 | Older version lacks `future_into_py`, Phase 9 already upgraded |
| Duration (ms conversion) | Raw u64 ms everywhere | Core uses Duration internally, conversion at boundary is cleaner |
| ValueError | RuntimeError | ValueError is Python convention for invalid arguments |

**Installation:**
Already in `py/Cargo.toml` from Phase 9:
```toml
pyo3 = { version = "0.27", features = ["extension-module", "abi3-py39"] }
pyo3-async-runtimes = { version = "0.27", features = ["tokio-runtime"] }
```

## Architecture Patterns

### Recommended Project Structure
```
py/src/
├── lib.rs              # Module registration
├── client.rs           # RestClient with kwargs constructor
├── websocket.rs        # WebSocketClient, ReconnectConfig (partial), add HealthCheckConfig
├── errors.rs           # Exception hierarchy (already complete)
└── types.rs            # Type conversions
```

### Pattern 1: PyO3 Kwargs Constructor
**What:** `#[pyo3(signature = (*, api_key=None, bearer_token=None, ...))]` for Python kwargs
**When to use:** When mimicking official SDK constructors with multiple optional auth methods
**Example:**
```rust
// Source: PyO3 0.27 documentation + Phase 9 existing pattern
#[pymethods]
impl RestClient {
    #[new]
    #[pyo3(signature = (*, api_key=None, bearer_token=None, sdk_token=None, base_url=None))]
    pub fn new(
        api_key: Option<String>,
        bearer_token: Option<String>,
        sdk_token: Option<String>,
        base_url: Option<String>,
    ) -> PyResult<Self> {
        // Count non-None auth methods
        let auth_count = [&api_key, &bearer_token, &sdk_token]
            .iter()
            .filter(|opt| opt.is_some())
            .count();

        if auth_count != 1 {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "Provide exactly one of: api_key, bearer_token, sdk_token"
            ));
        }

        let auth = if let Some(key) = api_key {
            marketdata_core::Auth::ApiKey(key)
        } else if let Some(token) = bearer_token {
            marketdata_core::Auth::BearerToken(token)
        } else {
            marketdata_core::Auth::SdkToken(sdk_token.unwrap())
        };

        let mut inner = marketdata_core::RestClient::new(auth);
        if let Some(url) = base_url {
            inner = inner.with_base_url(&url);
        }

        Ok(Self { inner })
    }
}
```

### Pattern 2: Config Class as PyClass
**What:** Expose Rust config struct as Python class with validation in `__new__`
**When to use:** For configuration objects passed to client constructors
**Example:**
```rust
// Source: py/src/websocket.rs existing ReconnectConfig + Phase 8 validation
#[pyclass]
#[derive(Clone)]
pub struct HealthCheckConfig {
    #[pyo3(get)]
    pub enabled: bool,
    #[pyo3(get)]
    pub interval_ms: u64,
    #[pyo3(get)]
    pub max_missed_pongs: u64,
}

#[pymethods]
impl HealthCheckConfig {
    #[new]
    #[pyo3(signature = (*, enabled=false, interval_ms=30000, max_missed_pongs=2))]
    pub fn new(enabled: bool, interval_ms: u64, max_missed_pongs: u64) -> PyResult<Self> {
        // Validate using core's logic
        let duration = std::time::Duration::from_millis(interval_ms);
        let _ = marketdata_core::HealthCheckConfig::new(enabled, duration, max_missed_pongs)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

        Ok(Self { enabled, interval_ms, max_missed_pongs })
    }
}
```

### Pattern 3: FFI Millisecond Conversion
**What:** Accept ms as u64 at FFI boundary, convert to Duration for core
**When to use:** All time-based config parameters (follows v0.3.0 CON-03)
**Example:**
```rust
// Convert Python ms to core Duration
impl HealthCheckConfig {
    pub fn to_core(&self) -> Result<marketdata_core::HealthCheckConfig, marketdata_core::MarketDataError> {
        marketdata_core::HealthCheckConfig::new(
            self.enabled,
            std::time::Duration::from_millis(self.interval_ms),
            self.max_missed_pongs,
        )
    }
}
```

### Pattern 4: Optional Config Parameters
**What:** Accept `Option<PyRef<ConfigClass>>` for optional config objects
**When to use:** WebSocketClient that can work with or without custom config
**Example:**
```rust
// Source: PyO3 0.27 patterns for optional PyClass arguments
#[pymethods]
impl WebSocketClient {
    #[new]
    #[pyo3(signature = (*, api_key=None, bearer_token=None, sdk_token=None,
                        base_url=None, reconnect=None, health_check=None))]
    pub fn new(
        api_key: Option<String>,
        bearer_token: Option<String>,
        sdk_token: Option<String>,
        base_url: Option<String>,
        reconnect: Option<&Bound<'_, ReconnectConfig>>,
        health_check: Option<&Bound<'_, HealthCheckConfig>>,
    ) -> PyResult<Self> {
        // Auth validation (same as RestClient)
        // ...

        // Config extraction with defaults
        let reconnect_config = if let Some(cfg) = reconnect {
            cfg.borrow().clone()
        } else {
            ReconnectConfig::default()
        };

        Ok(Self {
            api_key: api_key.unwrap(), // We validated exactly one exists
            reconnect_config,
            // ... rest of fields
        })
    }
}
```

### Anti-Patterns to Avoid
- **Positional string constructor:** Phase 9 used `RestClient::new(api_key: String)`, but v0.3.0 requires kwargs for API compatibility. CONTEXT.md confirms "breaking change, no deprecation needed."
- **Deferred validation:** Don't defer config validation to `connect()`. PyO3 convention and CONTEXT.md decision: fail fast at construction.
- **Python bool → Option<Config>:** Don't use `enabled: bool` parameter in client constructors. Use `Option<ConfigClass>` where None means defaults.
- **Mixing time units:** Don't accept seconds in some places, ms in others. CON-03: milliseconds at all FFI boundaries.

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Config validation logic | Custom Python-side checks | Core's `ReconnectionConfig::new()` | Phase 8 already has comprehensive validation with proper error messages |
| Auth method selection | Match statement in Python layer | Core's `Auth` enum | Already exists, type-safe |
| Time unit conversion | Custom ms→Duration helpers | `std::time::Duration::from_millis()` | Standard library, no dependencies |
| Exception mapping | Manual if-else chains | `map_err()` with PyO3 exceptions | Idiomatic Rust + PyO3 error handling |
| GIL-safe async | Custom thread pools | `future_into_py` + `spawn_blocking` | Phase 9 pattern already proven |

**Key insight:** Phase 8 (core validation) and Phase 9 (async foundation) already solved the hard problems. This phase is primarily wiring existing pieces through PyO3 kwargs.

## Common Pitfalls

### Pitfall 1: PyO3 0.27 `Bound<'_, T>` Lifetime Issues
**What goes wrong:** Attempting to store `&Bound<'_, PyClass>` beyond function scope causes borrow checker errors
**Why it happens:** PyO3 0.27 changed from `PyRef<T>` to `Bound<'_, T>` with stricter lifetimes for GIL safety
**How to avoid:** Clone the inner data immediately via `.borrow().clone()`, don't hold `Bound` references
**Warning signs:** Compiler errors mentioning "borrowed value does not live long enough" with `Bound`

**Example:**
```rust
// ❌ WRONG: Tries to store Bound reference
pub fn new(config: Option<&Bound<'_, HealthCheckConfig>>) -> PyResult<Self> {
    Ok(Self { config }) // ERROR: lifetime issues
}

// ✅ CORRECT: Clone inner data
pub fn new(config: Option<&Bound<'_, HealthCheckConfig>>) -> PyResult<Self> {
    let config = config.map(|c| c.borrow().clone());
    Ok(Self { config })
}
```

### Pitfall 2: Auth Validation Order
**What goes wrong:** Checking auth after creating core client, allowing invalid state to propagate
**Why it happens:** Natural tendency to pass through to core first, validate later
**How to avoid:** Validate exactly-one-auth FIRST in Python constructor, before touching core
**Warning signs:** Core errors escaping to Python instead of clean ValueError

**Example:**
```rust
// ❌ WRONG: Creates client before validating
pub fn new(api_key: Option<String>, bearer_token: Option<String>) -> PyResult<Self> {
    let auth = api_key.map(Auth::ApiKey).or(bearer_token.map(Auth::BearerToken));
    let inner = RestClient::new(auth?); // Could panic on None
    Ok(Self { inner })
}

// ✅ CORRECT: Validate first
pub fn new(api_key: Option<String>, bearer_token: Option<String>) -> PyResult<Self> {
    let count = [&api_key, &bearer_token].iter().filter(|o| o.is_some()).count();
    if count != 1 {
        return Err(PyValueError::new_err("Provide exactly one of: api_key, bearer_token"));
    }
    // Now safe to unwrap
}
```

### Pitfall 3: Forgetting `#[pyo3(get)]` for Config Fields
**What goes wrong:** Python users can't read config attributes: `AttributeError: 'HealthCheckConfig' object has no attribute 'interval_ms'`
**Why it happens:** PyO3 fields are private by default, need explicit `#[pyo3(get)]` or `#[pymethods]` getter
**How to avoid:** Add `#[pyo3(get)]` to all config fields users should read (for immutable configs)
**Warning signs:** Python test failures when trying to access `config.enabled` or similar

**Example:**
```rust
// ❌ WRONG: Fields not exposed
#[pyclass]
pub struct HealthCheckConfig {
    pub enabled: bool,  // Can't access from Python!
}

// ✅ CORRECT: Expose with #[pyo3(get)]
#[pyclass]
pub struct HealthCheckConfig {
    #[pyo3(get)]
    pub enabled: bool,  // Python can read via config.enabled
}
```

### Pitfall 4: Missing Module Registration
**What goes wrong:** `ImportError: cannot import name 'HealthCheckConfig' from 'marketdata_py'`
**Why it happens:** Forgetting to add new PyClass to module registration in `lib.rs`
**How to avoid:** After creating new `#[pyclass]`, immediately add `m.add_class::<NewClass>()?;` to `marketdata_py()` function
**Warning signs:** Class compiles but can't be imported in Python

**Example:**
```rust
// In lib.rs
#[pymodule]
fn marketdata_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<RestClient>()?;
    m.add_class::<WebSocketClient>()?;
    m.add_class::<ReconnectConfig>()?;
    // ⚠️ DON'T FORGET: Add new classes here
    m.add_class::<HealthCheckConfig>()?;  // <-- REQUIRED
    Ok(())
}
```

### Pitfall 5: Type Conversion Edge Cases with PyO3 0.27
**What goes wrong:** Runtime errors when PyO3 tries to convert Python types to Rust
**Why it happens:** PyO3 0.27 changed `IntoPyObject` trait, some conversions require explicit handling
**How to avoid:** Let PyO3 handle primitive types (str, int, bool), only custom-convert complex types
**Warning signs:** `TypeError: argument 'interval_ms': 'str' object cannot be converted to 'int'`

## Code Examples

Verified patterns from existing codebase and PyO3 0.27 documentation:

### RestClient Kwargs Constructor
```rust
// Source: py/src/client.rs (Phase 9) + CONTEXT.md requirements
#[pyclass]
pub struct RestClient {
    inner: marketdata_core::RestClient,
}

#[pymethods]
impl RestClient {
    /// Create a new REST client with options
    ///
    /// Args:
    ///     api_key: Your Fugle API key (optional)
    ///     bearer_token: Bearer token for authentication (optional)
    ///     sdk_token: SDK token for authentication (optional)
    ///     base_url: Override base URL (optional)
    ///
    /// Exactly one of api_key, bearer_token, or sdk_token must be provided.
    ///
    /// Returns:
    ///     RestClient instance
    ///
    /// Raises:
    ///     ValueError: If zero or multiple auth methods provided
    ///
    /// Example:
    ///     ```python
    ///     from marketdata_py import RestClient
    ///
    ///     # API key auth
    ///     client = RestClient(api_key="your-key")
    ///
    ///     # Bearer token auth with custom base URL
    ///     client = RestClient(bearer_token="token", base_url="https://custom.api")
    ///     ```
    #[new]
    #[pyo3(signature = (*, api_key=None, bearer_token=None, sdk_token=None, base_url=None))]
    pub fn new(
        api_key: Option<String>,
        bearer_token: Option<String>,
        sdk_token: Option<String>,
        base_url: Option<String>,
    ) -> PyResult<Self> {
        // Validate exactly one auth method (fail fast per CONTEXT.md)
        let auth_count = [&api_key, &bearer_token, &sdk_token]
            .iter()
            .filter(|opt| opt.is_some())
            .count();

        if auth_count == 0 {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "Provide exactly one of: api_key, bearer_token, sdk_token"
            ));
        }

        if auth_count > 1 {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "Provide exactly one of: api_key, bearer_token, sdk_token"
            ));
        }

        // Build auth (safe to unwrap after validation)
        let auth = if let Some(key) = api_key {
            marketdata_core::Auth::ApiKey(key)
        } else if let Some(token) = bearer_token {
            marketdata_core::Auth::BearerToken(token)
        } else {
            marketdata_core::Auth::SdkToken(sdk_token.unwrap())
        };

        // Create client with optional base_url
        let mut inner = marketdata_core::RestClient::new(auth);
        if let Some(url) = base_url {
            inner = inner.with_base_url(&url);
        }

        Ok(Self { inner })
    }

    // Keep static methods for backwards compatibility
    #[staticmethod]
    pub fn with_bearer_token(token: String) -> Self {
        Self { inner: marketdata_core::RestClient::new(marketdata_core::Auth::BearerToken(token)) }
    }

    #[staticmethod]
    pub fn with_sdk_token(sdk_token: String) -> Self {
        Self { inner: marketdata_core::RestClient::new(marketdata_core::Auth::SdkToken(sdk_token)) }
    }
}
```

### HealthCheckConfig PyClass
```rust
// Source: Phase 8 core validation + CONTEXT.md time unit decisions
/// Health check configuration for WebSocket connections
///
/// Configures ping/pong based connection monitoring.
///
/// # Example (Python)
///
/// ```python
/// from marketdata_py import HealthCheckConfig, WebSocketClient
///
/// # Custom health check
/// health_check = HealthCheckConfig(
///     enabled=True,
///     interval_ms=15000,  # 15 seconds
///     max_missed_pongs=3
/// )
///
/// ws = WebSocketClient(
///     api_key="your-key",
///     health_check=health_check
/// )
/// ```
#[pyclass]
#[derive(Clone)]
pub struct HealthCheckConfig {
    /// Whether health check is enabled
    #[pyo3(get)]
    pub enabled: bool,
    /// Interval between ping messages in milliseconds
    #[pyo3(get)]
    pub interval_ms: u64,
    /// Maximum missed pongs before disconnect
    #[pyo3(get)]
    pub max_missed_pongs: u64,
}

#[pymethods]
impl HealthCheckConfig {
    /// Create a new health check configuration
    ///
    /// Args:
    ///     enabled: Whether health check is enabled (default: False)
    ///     interval_ms: Interval between pings in milliseconds (default: 30000, min: 5000)
    ///     max_missed_pongs: Max missed pongs before disconnect (default: 2, min: 1)
    ///
    /// Raises:
    ///     ValueError: If validation fails (interval < 5000ms or max_missed_pongs == 0)
    ///
    /// Example:
    ///     ```python
    ///     # Default config (disabled)
    ///     config = HealthCheckConfig()
    ///
    ///     # Enabled with custom settings
    ///     config = HealthCheckConfig(enabled=True, interval_ms=15000, max_missed_pongs=3)
    ///     ```
    #[new]
    #[pyo3(signature = (*, enabled=false, interval_ms=30000, max_missed_pongs=2))]
    pub fn new(enabled: bool, interval_ms: u64, max_missed_pongs: u64) -> PyResult<Self> {
        // Validate using core's validation logic (fail fast)
        let duration = std::time::Duration::from_millis(interval_ms);
        let _ = marketdata_core::HealthCheckConfig::new(enabled, duration, max_missed_pongs)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;

        Ok(Self {
            enabled,
            interval_ms,
            max_missed_pongs,
        })
    }
}

impl HealthCheckConfig {
    /// Convert to core HealthCheckConfig
    ///
    /// This should not fail since validation happened in __new__
    pub fn to_core(&self) -> marketdata_core::HealthCheckConfig {
        marketdata_core::HealthCheckConfig::new(
            self.enabled,
            std::time::Duration::from_millis(self.interval_ms),
            self.max_missed_pongs,
        ).expect("Config already validated in constructor")
    }
}
```

### WebSocketClient with Config Parameters
```rust
// Source: py/src/websocket.rs existing structure + CONTEXT.md requirements
#[pyclass]
pub struct WebSocketClient {
    api_key: String,
    reconnect_config: ReconnectConfig,
    health_check_config: HealthCheckConfig,
}

#[pymethods]
impl WebSocketClient {
    /// Create a new WebSocket client with configuration
    ///
    /// Args:
    ///     api_key: Your Fugle API key (optional)
    ///     bearer_token: Bearer token for authentication (optional)
    ///     sdk_token: SDK token for authentication (optional)
    ///     base_url: Override WebSocket URL (optional)
    ///     reconnect: ReconnectConfig instance (optional, defaults to enabled with 5 attempts)
    ///     health_check: HealthCheckConfig instance (optional, defaults to disabled)
    ///
    /// Exactly one of api_key, bearer_token, or sdk_token must be provided.
    ///
    /// Example:
    ///     ```python
    ///     from marketdata_py import WebSocketClient, ReconnectConfig, HealthCheckConfig
    ///
    ///     # Simple usage with defaults
    ///     ws = WebSocketClient(api_key="your-key")
    ///
    ///     # Custom reconnection config
    ///     reconnect = ReconnectConfig(max_attempts=10, initial_delay_ms=2000)
    ///     ws = WebSocketClient(api_key="your-key", reconnect=reconnect)
    ///
    ///     # Enable health check
    ///     health_check = HealthCheckConfig(enabled=True, interval_ms=20000)
    ///     ws = WebSocketClient(api_key="your-key", health_check=health_check)
    ///     ```
    #[new]
    #[pyo3(signature = (*, api_key=None, bearer_token=None, sdk_token=None,
                        base_url=None, reconnect=None, health_check=None))]
    pub fn new(
        api_key: Option<String>,
        bearer_token: Option<String>,
        sdk_token: Option<String>,
        base_url: Option<String>,
        reconnect: Option<&Bound<'_, ReconnectConfig>>,
        health_check: Option<&Bound<'_, HealthCheckConfig>>,
    ) -> PyResult<Self> {
        // Same auth validation as RestClient
        let auth_count = [&api_key, &bearer_token, &sdk_token]
            .iter()
            .filter(|opt| opt.is_some())
            .count();

        if auth_count != 1 {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "Provide exactly one of: api_key, bearer_token, sdk_token"
            ));
        }

        // Extract the one provided auth method
        let auth_key = api_key.or(bearer_token).or(sdk_token).unwrap();

        // Extract configs with defaults (clone from Bound to avoid lifetime issues)
        let reconnect_config = if let Some(cfg) = reconnect {
            cfg.borrow().clone()
        } else {
            ReconnectConfig::default()
        };

        let health_check_config = if let Some(cfg) = health_check {
            cfg.borrow().clone()
        } else {
            HealthCheckConfig::new(false, 30000, 2)?
        };

        // TODO: Store base_url if provided (for Phase 12 implementation)

        Ok(Self {
            api_key: auth_key,
            reconnect_config,
            health_check_config,
            // ... rest of fields from existing implementation
        })
    }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Positional string: `RestClient("key")` | Kwargs: `RestClient(api_key="key")` | Phase 12 (v0.3.0) | API-compatible with official SDK, breaking change per CONTEXT.md |
| No config exposure | Config classes as parameters | Phase 12 (v0.3.0) | Users can customize reconnection and health check behavior |
| Validation at connection time | Validation at construction time | Phase 8 + 12 | Fail-fast principle, better DX |
| PyO3 0.22 | PyO3 0.27 | Phase 9 (v0.2.0) | Modern async patterns, `future_into_py` |
| Seconds for time values | Milliseconds at FFI | v0.3.0 decision | Avoids Duration serialization complexity (CON-03) |
| Health check default: true | Health check default: false | Phase 8 | Aligned with official SDKs (CON-01) |

**Deprecated/outdated:**
- `RestClient::new(api_key: String)`: Replaced by kwargs constructor, remove old signature (no deprecation per CONTEXT.md)
- `ReconnectConfig` field names: Phase 9 used `max_retries`/`base_delay_ms`, should be `max_attempts`/`initial_delay_ms` to match core

## Open Questions

Things that couldn't be fully resolved:

1. **`ReconnectConfig` field name mismatch**
   - What we know: Phase 9 created `ReconnectConfig` with `max_retries`/`base_delay_ms` fields
   - What's unclear: Core uses `max_attempts`/`initial_delay`. Need to align naming.
   - Recommendation: Rename Python fields to match core (breaking change acceptable per CONTEXT.md)

2. **Static method deprecation strategy**
   - What we know: Phase 9 added `RestClient.with_bearer_token()` and `.with_sdk_token()` static methods
   - What's unclear: CONTEXT.md says "no deprecation needed" but doesn't address static methods
   - Recommendation: Keep static methods for convenience, document kwargs constructor as primary

3. **`base_url` parameter for WebSocketClient**
   - What we know: CONTEXT.md requires `base_url` kwarg for both REST and WebSocket
   - What's unclear: How to pass custom URL to `ConnectionConfig::fugle_stock(auth)`
   - Recommendation: Add `base_url` parameter to ConnectionConfig constructor, or use `ConnectionConfig::new(url, auth)` when custom URL provided

## Sources

### Primary (HIGH confidence)
- py/src/websocket.rs (existing ReconnectConfig PyClass pattern)
- py/src/client.rs (existing RestClient structure, static methods)
- core/src/websocket/reconnection.rs (ReconnectionConfig validation, exported constants)
- core/src/websocket/health_check.rs (HealthCheckConfig validation, exported constants)
- core/src/lib.rs (exported constants for binding layers)
- fugle-marketdata-python/fugle_marketdata/websocket/client.py (official SDK HealthCheckConfig pattern)
- .planning/phases/12-python-config/12-CONTEXT.md (locked implementation decisions)
- .planning/ROADMAP.md (v0.3.0 phase dependencies and deliverables)
- .planning/REQUIREMENTS.md (API-01 to API-05, WS-01 to WS-06, VAL-01 to VAL-04)

### Secondary (MEDIUM confidence)
- PyO3 0.27 documentation (kwargs syntax, Bound<'_, T> lifetime patterns)
- PyO3 0.27 migration guide (IntoPyObject trait changes, future_into_py usage)

### Tertiary (LOW confidence)
- None (all findings verified with primary sources)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - PyO3 0.27 already in use from Phase 9, core validation complete from Phase 8
- Architecture: HIGH - Existing py/src/ structure clear, patterns from Phase 9 proven
- Pitfalls: HIGH - Based on actual py/src/ code review and PyO3 0.27 common issues

**Research date:** 2026-02-05
**Valid until:** 2026-03-07 (30 days - stable domain, PyO3 0.27 patterns established)
