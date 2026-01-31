# Domain Pitfalls: SDK API Changes (v0.3.0)

**Context:** Breaking changes to constructor signatures and configuration options across 5 language bindings
**Milestone:** v0.3.0 - Constructor from string → options object, adding reconnection/health check config
**Researched:** 2026-02-01

## Critical Pitfalls

Mistakes that cause production incidents, user migration failures, or major rewrites.

---

### Pitfall 1: Breaking Existing User Code Without Migration Path

**What goes wrong:** Users upgrade to v0.3.0 and their code immediately breaks with cryptic errors. No clear migration guidance, no deprecation warnings, no backward compatibility shims.

**Why it happens:**
- Direct string-to-object constructor change: `new RestClient('api-key')` → `new RestClient({ apiKey: 'api-key' })` breaks immediately
- No gradual migration period with deprecation warnings
- Package managers auto-update to minor versions (if using `^0.2.0` in package.json)
- Users don't read changelogs before updating
- Error messages don't explain what changed: "TypeError: Cannot read property 'apiKey' of undefined" vs helpful migration hint

**Consequences:**
- Production applications break during routine dependency updates
- Support burden: flood of GitHub issues "SDK stopped working after update"
- Users pin to v0.2.x indefinitely, fragmenting user base
- Negative sentiment: "SDK is unstable", damages trust
- Emergency rollback and hotfix cycles

**Prevention:**

**Strategy 1: Deprecation-first approach (RECOMMENDED)**
```rust
// Phase 1 (v0.2.1): Add new constructor, deprecate old
impl RestClient {
    // Old constructor - deprecated but still works
    #[deprecated(since = "0.2.1", note = "Use RestClient::new(config) with RestClientConfig instead")]
    pub fn from_api_key(api_key: String) -> Self {
        Self::new(RestClientConfig {
            api_key,
            timeout: None,
            ..Default::default()
        })
    }

    // New constructor
    pub fn new(config: RestClientConfig) -> Self {
        // Implementation
    }
}

// Phase 2 (v0.3.0): Remove deprecated constructor
// Users had 1-2 months to migrate
```

**Strategy 2: Constructor overloading (language-specific)**
```python
# Python: Accept both string and dict
@classmethod
def __new__(cls, config):
    if isinstance(config, str):
        warnings.warn(
            "Passing api_key as string is deprecated. "
            "Use RestClient({'api_key': '...'}) instead.",
            DeprecationWarning,
            stacklevel=2
        )
        config = {'api_key': config}
    return super().__new__(cls)
```

```javascript
// JavaScript/TypeScript: Union types
constructor(config: string | RestClientConfig) {
    if (typeof config === 'string') {
        console.warn('Passing api_key as string is deprecated...');
        this.config = { apiKey: config };
    } else {
        this.config = config;
    }
}
```

**Strategy 3: Clear error messages with migration guidance**
```rust
// If we MUST break immediately, at least make errors helpful
impl RestClient {
    pub fn new(config: RestClientConfig) -> Result<Self, ConfigError> {
        if config.api_key.is_empty() {
            return Err(ConfigError::MigrationHelp(
                "RestClient constructor changed in v0.3.0.\n\
                 Old: RestClient::new('api-key')\n\
                 New: RestClient::new(RestClientConfig { api_key: 'api-key', ..Default::default() })\n\
                 See migration guide: https://docs.fugle.tw/sdk/migration-v0.3"
            ));
        }
        // ...
    }
}
```

**Detection:**
- User reports: "code broke after update"
- High volume of identical issues on GitHub
- Package download stats show users NOT upgrading to v0.3.x
- Automated dependency scanners (Dependabot) create PRs that break CI
- Rollback commits with messages like "revert to v0.2.x, v0.3.x broken"

**Warning signs:**
- No deprecation warnings in v0.2.x logs
- Migration guide not written before v0.3.0 release
- Breaking changes not clearly marked in CHANGELOG with "BREAKING CHANGE:" prefix
- No automated migration tool or codemod provided
- Test coverage doesn't include "old API still works" cases in deprecation phase

**Phase mapping:**
- **Phase 0 (Pre-v0.3.0)**: Add deprecation warnings to v0.2.x, publish migration guide
- **Phase 1**: Implement dual-constructor support in Rust core
- **Phase 2-5**: Language bindings implement graceful fallback or clear errors
- **Phase 6**: Documentation shows both old (deprecated) and new patterns
- **Phase 7**: Remove deprecated constructors AFTER 2-3 month migration window

**Sources:**
- [Managing API Changes: 8 Strategies That Reduce Disruption by 70% (2026 Guide)](https://www.theneo.io/blog/managing-api-changes-strategies)
- [What Are Breaking Changes and How Do You Avoid Them?](https://nordicapis.com/what-are-breaking-changes-and-how-do-you-avoid-them/)
- [Handling Breaking Changes in SDKs | Speakeasy](https://www.speakeasy.com/docs/sdks/manage/breaking-changes)

---

### Pitfall 2: Inconsistent Configuration Defaults Across Languages

**What goes wrong:** Python defaults reconnection to enabled, Node.js defaults to disabled, C# requires explicit setting. Users migrating between languages get different behavior, production systems behave unpredictably.

**Why it happens:**
- Each language binding implemented independently without coordination
- "Sensible defaults" differ per language ecosystem culture
- Python: "batteries included" → enable reconnection by default
- Node.js: "explicit over implicit" → require opt-in
- C#: "fail fast" → throw if not configured
- No central configuration schema enforcing consistency
- Language-specific idioms override cross-language consistency goal

**Consequences:**
- Polyglot teams get confused: "works in Python, fails in Node.js"
- Documentation becomes language-specific, can't share examples
- Users switching languages encounter surprising behavior differences
- Production incidents when same config works in dev (Python) but not prod (C#)
- Testing nightmare: need language-specific test matrices

**Prevention:**

**Strategy 1: Single source of truth for defaults**
```rust
// core/src/config.rs - SHARED defaults
pub const DEFAULT_RECONNECT_ENABLED: bool = true;
pub const DEFAULT_RECONNECT_MAX_ATTEMPTS: u32 = 5;
pub const DEFAULT_RECONNECT_INITIAL_DELAY_MS: u64 = 1000;
pub const DEFAULT_RECONNECT_MAX_DELAY_MS: u64 = 30000;
pub const DEFAULT_HEALTH_CHECK_INTERVAL_MS: u64 = 30000;
pub const DEFAULT_HEALTH_CHECK_TIMEOUT_MS: u64 = 5000;
pub const DEFAULT_REST_TIMEOUT_MS: u64 = 30000;

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            reconnect_enabled: DEFAULT_RECONNECT_ENABLED,
            reconnect_max_attempts: DEFAULT_RECONNECT_MAX_ATTEMPTS,
            // ... all languages MUST use these
        }
    }
}
```

**Strategy 2: Configuration compatibility matrix documentation**
```markdown
# Configuration Defaults Across Languages

| Option | Default | Python | Node.js | C# | Java | Go |
|--------|---------|--------|---------|----|----- |----|
| reconnect_enabled | `true` | ✅ | ✅ | ✅ | ✅ | ✅ |
| max_attempts | `5` | ✅ | ✅ | ✅ | ✅ | ✅ |
| health_check_interval | `30s` | ✅ | ✅ | ✅ | ✅ | ✅ |

ALL languages MUST use identical defaults. Deviations require RFC approval.
```

**Strategy 3: Cross-language configuration tests**
```python
# tests/cross_language_config_test.py
def test_default_configs_match_across_languages():
    """Ensure Python defaults match Rust core defaults"""
    python_config = WebSocketConfig()

    # These MUST match core/src/config.rs constants
    assert python_config.reconnect_enabled == True  # DEFAULT_RECONNECT_ENABLED
    assert python_config.max_attempts == 5  # DEFAULT_RECONNECT_MAX_ATTEMPTS
    assert python_config.health_check_interval == 30.0  # DEFAULT_HEALTH_CHECK_INTERVAL_MS / 1000
```

```javascript
// js/tests/config-consistency.test.js
test('Node.js defaults match Rust core', () => {
    const config = new WebSocketConfig();

    // Must match core/src/config.rs
    expect(config.reconnectEnabled).toBe(true);
    expect(config.maxAttempts).toBe(5);
});
```

**Strategy 4: Automated default extraction**
```bash
# CI check: extract Rust defaults and generate language-specific constants
cargo run --bin extract-defaults > defaults.json

# Validate each language reads from same source
python scripts/validate_defaults.py --language python --expected defaults.json
node scripts/validate_defaults.js --language nodejs --expected defaults.json
```

**Detection:**
- Bug reports: "reconnection works in Python, not in JavaScript"
- Integration test failures when running same scenario in different languages
- Documentation contradictions between language sections
- Code review finds hardcoded defaults instead of importing from constants
- Config schema validation tests don't exist or don't cover all languages

**Warning signs:**
- Each language binding has its own `config.rs` / `config.py` / `config.js` with different values
- No automated test enforcing default consistency
- Language binding docs don't reference "official defaults" from core
- Contributors add language-specific "improvements" without cross-language review

**Phase mapping:**
- **Phase 1 (Rust core)**: Define canonical defaults, export as public constants
- **Phase 2 (Python)**: Import defaults from Rust via FFI or code generation
- **Phase 3 (Node.js)**: Same approach as Python
- **Phase 4 (C#)**: UniFFI should expose defaults automatically
- **Phase 5 (Java)**: Validate defaults match core in tests
- **All phases**: Add CI check that fails if defaults diverge

**Sources:**
- [Azure SDK Design Guidelines](https://learn.microsoft.com/en-us/azure/developer/python/sdk/fundamentals/language-design-guidelines)
- [Smart configuration defaults - AWS SDKs](https://docs.aws.amazon.com/sdkref/latest/guide/feature-smart-config-defaults.html)

---

### Pitfall 3: Configuration Validation Fails at Wrong Time

**What goes wrong:** Invalid configuration accepted at construction time, fails minutes later during actual connection. User loses context about what was wrong, debugging nightmare.

**Why it happens:**
- Lazy validation: defer checks until configuration is used
- Constructor doesn't validate, WebSocket.connect() does
- Validation happens async in background task, error lost or delayed
- Complex interdependencies: timeout < interval only detected at runtime
- Language boundaries: Python validates, Rust re-validates, error messages inconsistent

**Consequences:**
- User creates client successfully, gets error 30 seconds later when connecting
- Error message: "invalid timeout" - which timeout? (connect? read? health check?)
- Production systems deploy successfully, fail at runtime under load
- Debugging difficulty: error far from source of misconfiguration
- Logs scattered: config created in one place, error logged elsewhere

**Prevention:**

**Strategy 1: Fail-fast at construction time**
```rust
// core/src/config.rs
impl WebSocketConfig {
    pub fn new(builder: WebSocketConfigBuilder) -> Result<Self, ConfigError> {
        let config = builder.build();

        // VALIDATE IMMEDIATELY, not later
        Self::validate(&config)?;
        Ok(config)
    }

    fn validate(config: &WebSocketConfig) -> Result<(), ConfigError> {
        // Timeout constraints
        if config.connect_timeout == Duration::ZERO {
            return Err(ConfigError::InvalidTimeout {
                field: "connect_timeout",
                value: "0s",
                constraint: "must be > 0",
            });
        }

        // Logical constraints
        if config.health_check_timeout >= config.health_check_interval {
            return Err(ConfigError::ConstraintViolation {
                fields: vec!["health_check_timeout", "health_check_interval"],
                constraint: "timeout must be < interval",
                hint: "Set interval to at least 2x timeout for reliable health checks",
            });
        }

        // Reconnection constraints
        if config.reconnect_enabled && config.max_attempts == 0 {
            return Err(ConfigError::ConstraintViolation {
                fields: vec!["reconnect_enabled", "max_attempts"],
                constraint: "max_attempts must be > 0 when reconnection enabled",
                hint: "Set max_attempts to at least 1, or disable reconnection",
            });
        }

        // Range validation
        if config.max_attempts > 100 {
            return Err(ConfigError::OutOfRange {
                field: "max_attempts",
                value: config.max_attempts,
                min: 1,
                max: 100,
                hint: "Very high retry counts may cause long delays. Consider max 10-20.",
            });
        }

        Ok(())
    }
}
```

**Strategy 2: Helpful error messages with context**
```rust
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Invalid {field}: {value}. Constraint: {constraint}")]
    InvalidTimeout {
        field: &'static str,
        value: String,
        constraint: &'static str,
    },

    #[error("Configuration constraint violation: {constraint}\n  Fields: {fields:?}\n  Hint: {hint}")]
    ConstraintViolation {
        fields: Vec<&'static str>,
        constraint: &'static str,
        hint: &'static str,
    },

    #[error("{field} = {value} out of range [{min}, {max}]\n  Hint: {hint}")]
    OutOfRange {
        field: &'static str,
        value: u32,
        min: u32,
        max: u32,
        hint: &'static str,
    },
}
```

**Strategy 3: Builder pattern with compile-time guarantees**
```rust
// Make invalid states unrepresentable
pub struct WebSocketConfigBuilder {
    url: Option<String>,  // Required
    api_key: Option<String>,  // Required
    reconnect: ReconnectConfig,  // Has its own validated builder
    health_check: HealthCheckConfig,  // Has its own validated builder
}

impl WebSocketConfigBuilder {
    pub fn build(self) -> Result<WebSocketConfig, ConfigError> {
        let url = self.url.ok_or(ConfigError::MissingRequired("url"))?;
        let api_key = self.api_key.ok_or(ConfigError::MissingRequired("api_key"))?;

        // Sub-configs already validated by their builders
        Ok(WebSocketConfig {
            url,
            api_key,
            reconnect: self.reconnect,
            health_check: self.health_check,
        })
    }
}
```

**Strategy 4: Language-specific validation at FFI boundary**
```python
# py/src/config.py
class WebSocketConfig:
    def __init__(self, **kwargs):
        # Validate BEFORE passing to Rust
        self._validate_python_side(kwargs)

        # Create Rust config (which also validates)
        self._inner = _marketdata.WebSocketConfig(**kwargs)

    def _validate_python_side(self, kwargs):
        """Python-specific validation for better error messages"""
        if 'health_check_timeout' in kwargs and 'health_check_interval' in kwargs:
            timeout = kwargs['health_check_timeout']
            interval = kwargs['health_check_interval']

            if timeout >= interval:
                raise ValueError(
                    f"health_check_timeout ({timeout}s) must be < "
                    f"health_check_interval ({interval}s)\n"
                    f"Recommended: timeout = interval / 2"
                )
```

**Detection:**
- Bug reports: "error happens later, not at initialization"
- Errors with generic messages like "invalid configuration" without specifics
- Users report difficulty debugging configuration issues
- Logs show successful construction followed by connection failure
- Integration tests pass locally, fail in production with different config

**Warning signs:**
- Constructor doesn't return Result/throws no exceptions
- Validation code scattered in multiple places (constructor, connect, runtime)
- Error messages don't mention field names or constraint violations
- No test cases for invalid configurations
- Documentation doesn't list valid ranges for numeric options

**Phase mapping:**
- **Phase 1**: Add validation to Rust core config types with comprehensive error types
- **Phase 2-5**: Language bindings validate at construction, delegate to Rust for complete validation
- **Phase 6**: Add validation test suite covering all constraint violations
- **Phase 7**: Update documentation with valid ranges and constraints

**Sources:**
- [AWS SDK Configuration Validation](https://docs.aws.amazon.com/sdk-for-java/latest/developer-guide/timeouts.html)
- [Parameter validation in AWS JavaScript SDK](https://docs.aws.amazon.com/AWSJavaScriptSDK/latest/AWS/Config.html)

---

## Moderate Pitfalls

Mistakes that cause delays, technical debt, or require significant rework.

---

### Pitfall 4: Timeout Configuration Confusion

**What goes wrong:** Users don't understand which timeout applies where. Set connect_timeout thinking it affects health checks. Production deadlocks or premature disconnections.

**Why it happens:**
- Multiple timeout types: connect_timeout, read_timeout, health_check_timeout, rest_timeout
- Unclear naming: "timeout" could mean connection, operation, or idle
- No documentation of timeout hierarchy or interaction
- Users copy-paste config from examples without understanding
- Different defaults for different operations create inconsistency

**Consequences:**
- Health check timeout > interval → connection never considered unhealthy
- REST timeout < actual API latency → requests always fail
- Connect timeout too short → can't connect on slow networks
- No timeout set → operations hang indefinitely
- Production incidents: "why did connection drop after exactly 30 seconds?"

**Prevention:**

**Strategy 1: Self-documenting configuration names**
```rust
// ❌ CONFUSING
pub struct WebSocketConfig {
    pub timeout: Duration,  // Timeout for what?
    pub interval: Duration,  // Interval for what?
}

// ✅ CLEAR
pub struct WebSocketConfig {
    /// Maximum time to wait for WebSocket connection establishment
    /// Recommended: 10-30 seconds for production
    pub connect_timeout: Duration,

    /// Maximum time to wait for a message from server before considering connection stale
    /// Recommended: 2x health_check_interval to avoid false positives
    pub read_timeout: Duration,

    /// Health check ping interval - how often to send ping to server
    /// Recommended: 30-60 seconds for market data streams
    pub health_check_interval: Duration,

    /// Health check pong timeout - max time to wait for pong response
    /// MUST be < health_check_interval (recommended: interval / 3)
    pub health_check_timeout: Duration,
}

pub struct RestClientConfig {
    /// Maximum time for entire REST request (including connection + read)
    /// Recommended: 30-60 seconds for production APIs
    pub request_timeout: Duration,
}
```

**Strategy 2: Validation of timeout relationships**
```rust
impl WebSocketConfig {
    fn validate_timeouts(&self) -> Result<(), ConfigError> {
        // Health check timeout must be less than interval
        if self.health_check_timeout >= self.health_check_interval {
            return Err(ConfigError::TimeoutConstraint {
                issue: "health_check_timeout >= health_check_interval",
                fix: "Set timeout to at most 1/2 of interval",
                example: format!(
                    "interval: {}s → timeout: max {}s",
                    self.health_check_interval.as_secs(),
                    self.health_check_interval.as_secs() / 2
                ),
            });
        }

        // Connect timeout should be reasonable (warn if extreme)
        if self.connect_timeout > Duration::from_secs(60) {
            eprintln!(
                "Warning: connect_timeout = {}s is very high. \
                 Most connections succeed within 10-30s. \
                 Long timeouts delay error detection.",
                self.connect_timeout.as_secs()
            );
        }

        Ok(())
    }
}
```

**Strategy 3: Timeout configuration guide in documentation**
```markdown
# Timeout Configuration Guide

## WebSocket Timeouts

| Timeout | Purpose | Recommended Value | Notes |
|---------|---------|-------------------|-------|
| `connect_timeout` | Initial connection | 10-30s | Higher for slow networks |
| `read_timeout` | Message receive | 2x `health_check_interval` | Prevents false disconnects |
| `health_check_interval` | Ping frequency | 30-60s | Lower = more overhead |
| `health_check_timeout` | Pong wait | `interval / 3` | Must be < interval |

## REST Timeouts

| Timeout | Purpose | Recommended Value | Notes |
|---------|---------|-------------------|-------|
| `request_timeout` | Total request time | 30-60s | Includes connection + read |

## Configuration Examples

### Conservative (Slow Networks)
```python
config = WebSocketConfig(
    connect_timeout=30,  # Allow slow connection
    health_check_interval=60,  # Reduce ping frequency
    health_check_timeout=10,  # Wait up to 10s for pong
)
```

### Aggressive (Fast Networks, Quick Failure Detection)
```python
config = WebSocketConfig(
    connect_timeout=10,  # Fail fast if can't connect
    health_check_interval=30,  # Frequent health checks
    health_check_timeout=5,  # Expect fast pong
)
```

### Production Default (Balanced)
```python
config = WebSocketConfig(
    connect_timeout=20,
    health_check_interval=45,
    health_check_timeout=10,
)
```
```

**Strategy 4: Runtime timeout diagnostics**
```rust
impl WebSocketClient {
    /// Enable timeout diagnostics logging
    pub fn enable_timeout_diagnostics(&mut self) {
        self.diagnostics_enabled = true;
    }

    // Internal: log timeout events
    fn log_timeout_event(&self, event: TimeoutEvent) {
        if !self.diagnostics_enabled {
            return;
        }

        match event {
            TimeoutEvent::ConnectTimeout { elapsed } => {
                eprintln!(
                    "TIMEOUT: Connection failed after {:.1}s (limit: {:.1}s)\n  \
                     Check: network latency, firewall, server availability",
                    elapsed.as_secs_f64(),
                    self.config.connect_timeout.as_secs_f64()
                );
            }
            TimeoutEvent::HealthCheckTimeout { elapsed, expected } => {
                eprintln!(
                    "TIMEOUT: Health check pong not received after {:.1}s (limit: {:.1}s)\n  \
                     Possible causes: server overloaded, network congestion, timeout too short\n  \
                     Recommendation: Increase health_check_timeout or health_check_interval",
                    elapsed.as_secs_f64(),
                    expected.as_secs_f64()
                );
            }
        }
    }
}
```

**Detection:**
- Bug reports: "connection keeps dropping" (timeout too aggressive)
- Bug reports: "hangs forever" (timeout too long or missing)
- Users ask "which timeout should I change?" (unclear documentation)
- Production incidents correlate with timeout expiration
- Logs show timeout errors but unclear which timeout triggered

**Warning signs:**
- Configuration struct has generic names like `timeout` instead of specific names
- No validation of timeout relationships (e.g., health check timeout < interval)
- Documentation doesn't provide recommended values
- No timeout diagnostic logging in production
- Users setting all timeouts to same value (cargo-cult configuration)

**Phase mapping:**
- **Phase 1**: Rename timeout fields to be self-documenting in Rust core
- **Phase 2-5**: Update language bindings to match Rust naming
- **Phase 6**: Add timeout relationship validation
- **Phase 7**: Write comprehensive timeout configuration guide with examples
- **Phase 8**: Add timeout diagnostic logging

**Sources:**
- [Kubernetes Health Check Timeouts Best Practices](https://spacelift.io/blog/kubernetes-health-check)
- [Configure timeouts in AWS SDK](https://docs.aws.amazon.com/sdk-for-java/latest/developer-guide/timeouts.html)
- [Cloud Run Health Check Configuration](https://docs.cloud.google.com/run/docs/configuring/healthchecks)

---

### Pitfall 5: Reconnection Configuration Creates Infinite Loops

**What goes wrong:** Reconnection enabled with max_attempts=0 or initial_delay=0 causes tight loop consuming 100% CPU, flooding server with connection attempts.

**Why it happens:**
- max_attempts=0 interpreted as "infinite" instead of "disabled"
- initial_delay=0 → no backoff → instant reconnection storm
- Exponential backoff misconfigured: base=1, exponent=0 → always 1ms delay
- No jitter → all clients reconnect simultaneously (thundering herd)
- Validation doesn't catch these edge cases

**Consequences:**
- Client CPU pinned at 100%, drains battery on mobile
- Server flooded with connection requests, triggers DDoS protection
- IP address gets banned/rate-limited
- Logs filled with connection attempt messages
- Application becomes unresponsive
- Production outage from misconfiguration

**Prevention:**

**Strategy 1: Validate reconnection configuration**
```rust
pub struct ReconnectionConfig {
    pub enabled: bool,
    pub max_attempts: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub multiplier: f64,
}

impl ReconnectionConfig {
    pub fn validate(&self) -> Result<(), ConfigError> {
        if !self.enabled {
            return Ok(()); // Disabled, no validation needed
        }

        // If enabled, must have reasonable limits
        if self.max_attempts == 0 {
            return Err(ConfigError::InvalidReconnection {
                issue: "max_attempts = 0 with reconnection enabled",
                fix: "Set max_attempts to at least 1, or disable reconnection",
                hint: "max_attempts = 0 is ambiguous (infinite? or disabled?)",
            });
        }

        if self.initial_delay < Duration::from_millis(100) {
            return Err(ConfigError::InvalidReconnection {
                issue: format!("initial_delay = {}ms is too short", self.initial_delay.as_millis()),
                fix: "Set initial_delay to at least 100ms",
                hint: "Very short delays create connection storms",
            });
        }

        if self.max_delay < self.initial_delay {
            return Err(ConfigError::InvalidReconnection {
                issue: "max_delay < initial_delay",
                fix: "Set max_delay >= initial_delay",
                hint: "Exponential backoff needs room to grow",
            });
        }

        if self.multiplier < 1.0 {
            return Err(ConfigError::InvalidReconnection {
                issue: format!("multiplier = {} would decrease delays", self.multiplier),
                fix: "Set multiplier >= 1.0 (typically 2.0 for exponential backoff)",
                hint: "multiplier < 1.0 causes delays to shrink instead of grow",
            });
        }

        if self.multiplier > 10.0 {
            eprintln!(
                "Warning: multiplier = {} is very aggressive. \
                 Delays will grow extremely fast. Typical values: 1.5-2.0",
                self.multiplier
            );
        }

        Ok(())
    }
}
```

**Strategy 2: Safe defaults with explicit opt-in for aggressive settings**
```rust
impl Default for ReconnectionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_attempts: 5,  // NOT 0, NOT infinite
            initial_delay: Duration::from_secs(1),  // NOT milliseconds
            max_delay: Duration::from_secs(30),  // Cap exponential growth
            multiplier: 2.0,  // Standard exponential backoff
        }
    }
}

impl ReconnectionConfig {
    /// Create configuration for aggressive reconnection (use with caution)
    pub fn aggressive() -> Self {
        Self {
            enabled: true,
            max_attempts: 20,  // Many attempts
            initial_delay: Duration::from_millis(100),  // Fast start
            max_delay: Duration::from_secs(60),
            multiplier: 1.5,  // Slower growth
        }
    }

    /// Create configuration for infinite reconnection (DANGEROUS - use only if you have circuit breaker)
    pub fn infinite() -> Self {
        eprintln!(
            "WARNING: Using infinite reconnection. \
             Ensure you have circuit breaker or manual intervention capability."
        );
        Self {
            enabled: true,
            max_attempts: u32::MAX,  // Explicit, not 0
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(300),  // 5 minutes max
            multiplier: 2.0,
        }
    }
}
```

**Strategy 3: Add jitter to prevent thundering herd**
```rust
impl ReconnectionConfig {
    /// Calculate next delay with jitter to prevent thundering herd
    pub fn next_delay(&self, attempt: u32) -> Duration {
        let base_delay = self.initial_delay.as_millis() as u64;
        let exponential = base_delay * (self.multiplier.powi(attempt as i32) as u64);
        let capped = exponential.min(self.max_delay.as_millis() as u64);

        // Add random jitter: ±25% of delay
        let jitter_range = capped / 4;
        let jitter = rand::thread_rng().gen_range(0..jitter_range * 2);
        let final_delay = capped.saturating_sub(jitter_range) + jitter;

        Duration::from_millis(final_delay)
    }
}
```

**Strategy 4: Circuit breaker pattern**
```rust
pub struct CircuitBreaker {
    failures_in_window: AtomicU32,
    window_start: Mutex<Instant>,
    state: AtomicU8,  // 0=closed, 1=open, 2=half-open
}

impl CircuitBreaker {
    pub fn should_allow_attempt(&self) -> bool {
        const FAILURE_THRESHOLD: u32 = 10;  // 10 failures in 60s → open circuit
        const WINDOW_DURATION: Duration = Duration::from_secs(60);

        let failures = self.failures_in_window.load(Ordering::Relaxed);

        if failures >= FAILURE_THRESHOLD {
            eprintln!(
                "CIRCUIT BREAKER OPEN: {} connection failures in {}s. \
                 Temporarily stopping reconnection attempts.",
                failures,
                WINDOW_DURATION.as_secs()
            );
            return false;
        }

        true
    }
}
```

**Detection:**
- CPU usage at 100% for client process
- Server logs show rapid connection attempts from same IP
- Network traffic shows tight loop of connection establishment attempts
- Logs filled with "reconnection attempt #999..."
- No exponential backoff visible in timestamps (all within milliseconds)

**Warning signs:**
- Reconnection config allows max_attempts=0 without validation
- No minimum delay validation (accepts 0ms, 1ms)
- No jitter added to delays
- No circuit breaker or rate limiting
- Tests don't cover reconnection edge cases

**Phase mapping:**
- **Phase 1**: Add reconnection config validation to Rust core
- **Phase 2-5**: Language bindings validate before passing to Rust
- **Phase 6**: Add jitter to exponential backoff calculation
- **Phase 7**: Implement circuit breaker pattern
- **Phase 8**: Add reconnection diagnostics and rate limiting

**Sources:**
- [MQTT Client Auto-Reconnection Best Practices](https://www.emqx.com/en/blog/mqtt-client-auto-reconnect-best-practices)
- [Kubernetes Health Check Configuration](https://komodor.com/blog/kubernetes-health-checks-everything-you-need-to-know/)

---

### Pitfall 6: Options Object Breaks Type Safety in TypeScript

**What goes wrong:** TypeScript users pass invalid options object, gets runtime error instead of compile-time error. IntelliSense doesn't help, autocompletion broken.

**Why it happens:**
- Options object typed as `any` or too permissive
- Optional fields not clearly marked
- No discriminated unions for mutually exclusive options
- Default values not reflected in type definition
- napi-rs generates weak TypeScript types from Rust

**Consequences:**
- Users discover errors at runtime, not during development
- Typos in field names silently ignored: `apikey` vs `apiKey`
- Invalid combinations accepted: `reconnect_enabled=false, max_attempts=10`
- IDE autocomplete shows all fields, including internal ones
- Migration from v0.2.x loses type safety benefits

**Prevention:**

**Strategy 1: Strict TypeScript type definitions**
```typescript
// ❌ WEAK - accepts anything
export interface RestClientConfig {
    apiKey?: string;
    timeout?: number;
    [key: string]: any;  // BAD - allows typos
}

// ✅ STRONG - compile-time validation
export interface RestClientConfig {
    /** API key for authentication (required) */
    readonly apiKey: string;

    /** Request timeout in milliseconds (optional, default: 30000) */
    readonly requestTimeout?: number;

    /** Base URL override (optional, default: https://api.fugle.tw/...) */
    readonly baseUrl?: string;
}

// Even better: branded types prevent primitive obsession
export type ApiKey = string & { readonly __brand: 'ApiKey' };
export type Milliseconds = number & { readonly __brand: 'Milliseconds' };

export interface RestClientConfig {
    readonly apiKey: ApiKey;
    readonly requestTimeout?: Milliseconds;
}

// Helper to create branded types
export function apiKey(key: string): ApiKey {
    if (key.length === 0) {
        throw new TypeError('API key cannot be empty');
    }
    return key as ApiKey;
}

export function milliseconds(ms: number): Milliseconds {
    if (ms < 0) {
        throw new TypeError('Milliseconds cannot be negative');
    }
    return ms as Milliseconds;
}
```

**Strategy 2: Discriminated unions for mutually exclusive options**
```typescript
// Reconnection: either disabled or configured
export type ReconnectionConfig =
    | { enabled: false }
    | {
        enabled: true;
        maxAttempts: number;  // Required when enabled
        initialDelay: Milliseconds;
        maxDelay: Milliseconds;
        multiplier: number;
    };

export interface WebSocketConfig {
    readonly apiKey: ApiKey;
    readonly reconnection: ReconnectionConfig;
}

// TypeScript enforces consistency:
const config1: WebSocketConfig = {
    apiKey: apiKey('xxx'),
    reconnection: { enabled: false }  // ✅ OK
};

const config2: WebSocketConfig = {
    apiKey: apiKey('xxx'),
    reconnection: {
        enabled: true,
        // ❌ Compile error: missing maxAttempts, initialDelay, etc.
    }
};
```

**Strategy 3: Builder pattern with type-state**
```typescript
// Use builder to guide users through configuration
export class RestClientConfigBuilder {
    private config: Partial<RestClientConfig> = {};

    apiKey(key: string): this {
        this.config.apiKey = apiKey(key);
        return this;
    }

    requestTimeout(ms: number): this {
        this.config.requestTimeout = milliseconds(ms);
        return this;
    }

    build(): RestClientConfig {
        if (!this.config.apiKey) {
            throw new TypeError('apiKey is required');
        }

        return {
            apiKey: this.config.apiKey,
            requestTimeout: this.config.requestTimeout ?? milliseconds(30000),
        };
    }
}

// Usage: fluent API with IntelliSense
const config = new RestClientConfigBuilder()
    .apiKey('my-key')  // IntelliSense shows available methods
    .requestTimeout(60000)
    .build();
```

**Strategy 4: Runtime validation in constructor**
```typescript
export class RestClient {
    constructor(config: RestClientConfig) {
        // Validate despite TypeScript types (users might use 'as any')
        this.validateConfig(config);
        this.config = config;
    }

    private validateConfig(config: RestClientConfig): void {
        if (typeof config !== 'object' || config === null) {
            throw new TypeError('Config must be an object');
        }

        if (typeof config.apiKey !== 'string' || config.apiKey.length === 0) {
            throw new TypeError('apiKey must be a non-empty string');
        }

        if (config.requestTimeout !== undefined) {
            if (typeof config.requestTimeout !== 'number' || config.requestTimeout < 0) {
                throw new TypeError('requestTimeout must be a non-negative number');
            }
        }

        // Check for common typos
        const validKeys = new Set(['apiKey', 'requestTimeout', 'baseUrl']);
        for (const key of Object.keys(config)) {
            if (!validKeys.has(key)) {
                throw new TypeError(
                    `Unknown config option: '${key}'. Did you mean: ${this.suggestCorrection(key, validKeys)}`
                );
            }
        }
    }

    private suggestCorrection(key: string, validKeys: Set<string>): string {
        // Simple Levenshtein distance or substring matching
        const matches = Array.from(validKeys).filter(valid =>
            valid.toLowerCase().includes(key.toLowerCase()) ||
            key.toLowerCase().includes(valid.toLowerCase())
        );
        return matches.length > 0 ? matches.join(', ') : Array.from(validKeys).join(', ');
    }
}
```

**Detection:**
- TypeScript users report runtime errors that should be compile-time
- GitHub issues: "IntelliSense not working for config options"
- Users request "better types" or "stricter validation"
- Code reviews find excessive use of `as any` to bypass type checking
- Tests for invalid configs don't exist in TypeScript codebase

**Warning signs:**
- .d.ts file has `[key: string]: any` in config interfaces
- Optional fields not marked with `?` or have unclear purpose
- No JSDoc comments explaining fields
- No runtime validation in constructor despite having types
- napi-rs generated types not manually reviewed/improved

**Phase mapping:**
- **Phase 2 (Node.js binding)**: Generate strict TypeScript definitions from Rust
- **Phase 2**: Add runtime validation despite TypeScript types
- **Phase 2**: Implement builder pattern for complex configs
- **Phase 2**: Add typo detection for config options
- **Phase 6**: Document TypeScript patterns in migration guide

**Sources:**
- [TypeScript Best Practices 2026](https://www.xano.com/blog/modern-api-design-best-practices/)
- [.NET API Change Rules](https://learn.microsoft.com/en-us/dotnet/core/compatibility/library-change-rules)

---

## Minor Pitfalls

Mistakes that cause annoyance but are fixable without major rewrites.

---

### Pitfall 7: Documentation Examples Still Show Old API

**What goes wrong:** README and docs show v0.2.x string constructor after v0.3.0 release. Users copy-paste, get errors, confusion.

**Why it happens:**
- Documentation updated manually, easy to miss instances
- Example code in multiple places: README, docs/, API reference, blog posts
- Automated doc generation from code comments not updated
- Contributors update implementation, forget docs
- No CI check that examples actually compile/run

**Prevention:**

**Strategy 1: Testable documentation examples**
```rust
// In Rust doc comments - these are tested by cargo test
/// # Example (v0.3.0+)
/// ```
/// use marketdata_core::{RestClient, RestClientConfig};
///
/// let config = RestClientConfig {
///     api_key: "your-api-key".to_string(),
///     request_timeout: Some(Duration::from_secs(60)),
///     ..Default::default()
/// };
/// let client = RestClient::new(config)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Migrating from v0.2.x
/// ```
/// # use marketdata_core::{RestClient, RestClientConfig};
/// // Old (v0.2.x) - DEPRECATED
/// // let client = RestClient::new("your-api-key");
///
/// // New (v0.3.0+)
/// let config = RestClientConfig {
///     api_key: "your-api-key".to_string(),
///     ..Default::default()
/// };
/// let client = RestClient::new(config)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
```

**Strategy 2: CI checks for runnable examples**
```bash
# CI job: extract and test all README examples
npm run extract-readme-examples
npm test -- --grep "README examples"

# Python: use doctest
python -m doctest README.md

# Markdown linter: check for v0.2.x patterns
rg "new RestClient\('.*'\)" docs/ README.md && exit 1 || exit 0
```

**Strategy 3: Version-aware documentation**
```markdown
# Installation and Quick Start

## Version 0.3.0+ (Current)

```python
from marketdata_py import RestClient, RestClientConfig

config = RestClientConfig(
    api_key="your-api-key",
    request_timeout=60
)
client = RestClient(config)
```

## Version 0.2.x (Legacy)

<details>
<summary>Click to expand legacy v0.2.x example</summary>

```python
from marketdata_py import RestClient

# This API is deprecated in v0.3.0
client = RestClient("your-api-key")
```

See [Migration Guide](./MIGRATION.md) for upgrading to v0.3.0.

</details>
```

**Phase mapping:**
- **Phase 6 (Documentation)**: Update all examples to v0.3.0 API
- **Phase 6**: Add CI check for outdated patterns
- **Phase 6**: Write migration guide with before/after examples
- **Phase 7**: Deprecation warnings in v0.2.x point to migration guide

---

### Pitfall 8: Optional Configuration Fields Create Ambiguity

**What goes wrong:** User doesn't set `reconnect_enabled`, expects default `true`, but different language defaults to `false`. Inconsistent behavior.

**Why it happens:**
- Optional fields use language-specific defaults (Python: None, JavaScript: undefined, C#: null)
- No clear distinction between "not set" vs "explicitly set to false"
- Default value applied at different layers (Rust, language binding, application)
- Documentation doesn't clarify when defaults apply

**Prevention:**

**Strategy 1: Required fields with explicit defaults**
```rust
// ❌ Ambiguous: is None "disabled" or "use default"?
pub struct ReconnectionConfig {
    pub enabled: Option<bool>,  // None, Some(true), Some(false) - 3 states!
}

// ✅ Clear: always explicit
pub struct ReconnectionConfig {
    pub enabled: bool,  // Just true or false
}

impl Default for ReconnectionConfig {
    fn default() -> Self {
        Self { enabled: true }  // Explicit default
    }
}
```

**Strategy 2: Builder with clear defaults**
```python
# Python: use dataclass with explicit defaults
from dataclasses import dataclass, field

@dataclass
class ReconnectionConfig:
    """Reconnection configuration with explicit defaults."""

    enabled: bool = True  # Explicit: default is True
    max_attempts: int = 5
    initial_delay: float = 1.0  # seconds

    def __post_init__(self):
        """Validate after initialization."""
        if self.enabled and self.max_attempts == 0:
            raise ValueError("max_attempts must be > 0 when enabled=True")
```

**Phase mapping:**
- **Phase 1**: Define explicit defaults in Rust core
- **Phase 2-5**: Language bindings use same defaults, no "unset" state
- **Phase 6**: Document all defaults in configuration reference

---

### Pitfall 9: Migration Tooling Doesn't Exist

**What goes wrong:** Users must manually update all code from v0.2.x to v0.3.0 API. Error-prone, time-consuming, frustrating.

**Why it happens:**
- No automated migration script or codemod provided
- Breaking changes released without migration tool
- Assumption that users will manually update (they won't, or will make mistakes)
- No thought given to user experience during migration

**Prevention:**

**Strategy 1: Provide migration script**
```python
# scripts/migrate_to_v0_3.py
import re
import sys

def migrate_file(content: str) -> str:
    """Migrate Python code from v0.2.x to v0.3.0."""

    # Pattern: RestClient('api-key') → RestClient({'api_key': 'api-key'})
    pattern = r"RestClient\(['\"]([^'\"]+)['\"]\)"
    replacement = r"RestClient({'api_key': '\1'})"
    content = re.sub(pattern, replacement, content)

    # Pattern: WebSocketClient('api-key') → WebSocketClient({'api_key': 'api-key'})
    pattern = r"WebSocketClient\(['\"]([^'\"]+)['\"]\)"
    replacement = r"WebSocketClient({'api_key': '\1'})"
    content = re.sub(pattern, replacement, content)

    return content

if __name__ == '__main__':
    for filepath in sys.argv[1:]:
        with open(filepath) as f:
            content = f.read()

        migrated = migrate_file(content)

        if migrated != content:
            print(f"Migrating {filepath}...")
            with open(filepath, 'w') as f:
                f.write(migrated)
        else:
            print(f"No changes needed in {filepath}")
```

**Strategy 2: Use jscodeshift for JavaScript**
```javascript
// scripts/migrate-to-v0.3.js - jscodeshift codemod
module.exports = function(fileInfo, api) {
    const j = api.jscodeshift;
    const root = j(fileInfo.source);

    // Find: new RestClient('api-key')
    // Replace: new RestClient({ apiKey: 'api-key' })
    root
        .find(j.NewExpression, {
            callee: { name: 'RestClient' },
            arguments: args => args.length === 1 && j.Literal.check(args[0])
        })
        .replaceWith(path => {
            const apiKey = path.node.arguments[0].value;
            return j.newExpression(
                j.identifier('RestClient'),
                [j.objectExpression([
                    j.property('init', j.identifier('apiKey'), j.literal(apiKey))
                ])]
            );
        });

    return root.toSource();
};

// Usage: npx jscodeshift -t scripts/migrate-to-v0.3.js src/**/*.js
```

**Phase mapping:**
- **Phase 0 (Before v0.3.0)**: Write and test migration scripts
- **Phase 6**: Document migration scripts in migration guide
- **Phase 7**: Publish migration scripts to npm/PyPI

---

### Pitfall 10: No Configuration Schema for Validation

**What goes wrong:** Third-party tools (infrastructure-as-code, config management) can't validate SDK config. Users deploy invalid config to production.

**Why it happens:**
- No JSON Schema or equivalent for configuration
- Config validation only in SDK runtime, not at deployment time
- Infrastructure tools can't catch config errors before deployment

**Prevention:**

**Strategy 1: Publish JSON Schema**
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Fugle MarketData SDK Configuration",
  "type": "object",
  "required": ["apiKey"],
  "properties": {
    "apiKey": {
      "type": "string",
      "minLength": 1,
      "description": "API key for authentication"
    },
    "requestTimeout": {
      "type": "integer",
      "minimum": 1000,
      "maximum": 300000,
      "default": 30000,
      "description": "Request timeout in milliseconds"
    },
    "reconnection": {
      "type": "object",
      "properties": {
        "enabled": { "type": "boolean", "default": true },
        "maxAttempts": { "type": "integer", "minimum": 1, "maximum": 100, "default": 5 }
      }
    }
  }
}
```

**Phase mapping:**
- **Phase 6**: Generate JSON Schema from Rust types
- **Phase 6**: Publish schema for third-party tool integration

---

## Phase-Specific Warnings

| Phase | Pitfall Risk | Mitigation |
|-------|--------------|------------|
| **Phase 1: Rust Core Config** | Inconsistent defaults, missing validation | Define canonical defaults as public constants, implement comprehensive validation with helpful errors |
| **Phase 2: Python Binding** | Different defaults than Rust, unclear error messages | Import Rust defaults via FFI, add Python-side validation for better DX |
| **Phase 3: Node.js Binding** | Weak TypeScript types, runtime errors | Generate strict TypeScript definitions, add runtime validation despite types |
| **Phase 4: C# Binding** | UniFFI config mapping issues, .NET naming conventions | Test UniFFI-generated config types early, follow .NET casing (PascalCase) |
| **Phase 5: Java Binding** | Java null handling, builder pattern complexity | Use Optional<T> correctly, provide fluent builder with validation |
| **Phase 6: Documentation** | Examples show old API, migration guide missing | Update all examples before v0.3.0, provide runnable migration examples |
| **Phase 7: Migration Support** | Users can't migrate easily, no tooling | Provide migration scripts (Python/JavaScript), deprecation warnings in v0.2.x |
| **Phase 8: Release** | Breaking changes surprise users, inadequate notice | Publish deprecation warnings 2+ months before v0.3.0, clear CHANGELOG |

---

## Sources

**SDK Breaking Changes and Versioning:**
- [Handling Breaking Changes in SDKs | Speakeasy](https://www.speakeasy.com/docs/sdks/manage/breaking-changes)
- [Managing API Changes: 8 Strategies That Reduce Disruption by 70% (2026 Guide)](https://www.theneo.io/blog/managing-api-changes-strategies)
- [What Are Breaking Changes and How Do You Avoid Them? | Nordic APIs](https://nordicapis.com/what-are-breaking-changes-and-how-do-you-avoid-them/)
- [.NET API changes that affect compatibility | Microsoft Learn](https://learn.microsoft.com/en-us/dotnet/core/compatibility/library-change-rules)

**Configuration Best Practices:**
- [Smart configuration defaults - AWS SDKs and Tools](https://docs.aws.amazon.com/sdkref/latest/guide/feature-smart-config-defaults.html)
- [AWS SDKs and tools settings reference](https://docs.aws.amazon.com/sdkref/latest/guide/settings-reference.html)
- [Configure timeouts in AWS SDK for Java 2.x](https://docs.aws.amazon.com/sdk-for-java/latest/developer-guide/timeouts.html)

**Multi-Language SDK Consistency:**
- [Azure SDK Language Design Guidelines for Python | Microsoft Learn](https://learn.microsoft.com/en-us/azure/developer/python/sdk/fundamentals/language-design-guidelines)
- [Top Programming Languages for SDK Development in 2025](https://liblab.com/blog/top-programming-languages-for-sdk-development)

**Health Check and Timeout Configuration:**
- [Kubernetes Health Checks: Types, Configuration & Debugging](https://spacelift.io/blog/kubernetes-health-check)
- [Configure container health checks for services | Cloud Run](https://docs.cloud.google.com/run/docs/configuring/healthchecks)
- [Health checks in ASP.NET Core | Microsoft Learn](https://learn.microsoft.com/en-us/aspnet/core/host-and-deploy/health-checks?view=aspnetcore-9.0)

**Reconnection Best Practices:**
- [Ensuring Reliable IoT Device Connectivity: Best Practices for MQTT Client Auto-Reconnection](https://www.emqx.com/en/blog/mqtt-client-auto-reconnect-best-practices)
- [Understanding Kubernetes Health Checks & How-To with Examples](https://komodor.com/blog/kubernetes-health-checks-everything-you-need-to-know/)

**API Design Best Practices:**
- [8 API Versioning Best Practices for Developers in 2026](https://getlate.dev/blog/api-versioning-best-practices)
- [Modern API Design Best Practices for 2026](https://www.xano.com/blog/modern-api-design-best-practices/)

---

**Research completed:** 2026-02-01
**Confidence:** HIGH - Based on current AWS/Azure SDK patterns, 2026 API design guidelines, and existing project architecture
**Primary focus:** Constructor signature changes, configuration validation, cross-language consistency
