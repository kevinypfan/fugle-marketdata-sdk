# Phase 1: Core Config Validation & Defaults - Research

**Researched:** 2026-02-01
**Domain:** Rust configuration validation, SDK config patterns
**Confidence:** HIGH

## Summary

Phase 1 establishes the foundation for v0.3.0's configuration system by adding comprehensive validation to existing Rust core config structs (`ReconnectionConfig`, `HealthCheckConfig`) and exporting canonical defaults. The core insight: **the configuration infrastructure already exists and works correctly**, but lacks validation at construction time and public default constants.

Research confirms:
1. Current defaults in `reconnection.rs` and `health_check.rs` are production-ready
2. Health check default (`enabled: true`) **misaligns** with official SDKs (which default to `false`)
3. No validation prevents invalid configs (e.g., `max_attempts = 0`, `timeout >= interval`)
4. Defaults are not exported as public constants, making cross-language consistency difficult

**Primary recommendation:** Add fail-fast validation with helpful error messages, align health check default to `false`, export defaults as public constants for binding layers to reference.

## Standard Stack

The established libraries/tools for Rust configuration validation:

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `std::time::Duration` | stdlib | Time representation | Zero-cost abstraction, FFI-compatible via milliseconds |
| `thiserror` | 1.x | Error types | Industry standard for Rust error derivation |
| Built-in validation | N/A | Constructor validation | Zero dependencies, fail-fast pattern |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `validator` crate | Optional | Complex validation rules | If validation complexity grows beyond simple bounds checks |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Manual validation | `validator` crate | Manual validation sufficient for simple constraints, avoids dependency |
| Public constants | Code generation | Constants are simpler, more transparent, easier to maintain |

**Installation:**
```toml
# No new dependencies needed for Phase 1
# Existing Cargo.toml already has required dependencies
```

## Architecture Patterns

### Recommended Project Structure
```
core/src/
├── websocket/
│   ├── reconnection.rs      # ReconnectionConfig + validation
│   ├── health_check.rs      # HealthCheckConfig + validation
│   ├── config.rs            # ConnectionConfig (already has validation)
│   └── mod.rs               # Re-exports
├── lib.rs                   # Public API surface
└── errors.rs                # MarketDataError types
```

### Pattern 1: Fail-Fast Constructor Validation
**What:** Validate configuration at construction time, not at usage time
**When to use:** Always for configuration structs that will be passed across FFI boundaries
**Example:**
```rust
// Source: AWS SDK patterns, Azure SDK guidelines
impl ReconnectionConfig {
    pub fn new(max_attempts: u32, initial_delay: Duration, max_delay: Duration) -> Result<Self, ConfigError> {
        // Validate before construction
        Self::validate_params(max_attempts, initial_delay, max_delay)?;

        Ok(Self {
            max_attempts,
            initial_delay,
            max_delay,
        })
    }

    fn validate_params(max_attempts: u32, initial_delay: Duration, max_delay: Duration) -> Result<(), ConfigError> {
        if max_attempts == 0 {
            return Err(ConfigError::invalid_field(
                "max_attempts",
                "0",
                "must be >= 1",
            ));
        }

        if initial_delay < Duration::from_millis(100) {
            return Err(ConfigError::invalid_field(
                "initial_delay",
                &format!("{}ms", initial_delay.as_millis()),
                "must be >= 100ms to avoid connection storms",
            ));
        }

        if max_delay < initial_delay {
            return Err(ConfigError::constraint_violation(
                &["initial_delay", "max_delay"],
                "max_delay must be >= initial_delay",
            ));
        }

        Ok(())
    }
}
```

### Pattern 2: Public Default Constants
**What:** Export defaults as public constants for binding layers to reference
**When to use:** When multiple language bindings need to use identical defaults
**Example:**
```rust
// Source: Cross-language SDK consistency patterns
/// Default maximum reconnection attempts
pub const DEFAULT_RECONNECT_MAX_ATTEMPTS: u32 = 5;

/// Default initial reconnection delay (milliseconds)
pub const DEFAULT_RECONNECT_INITIAL_DELAY_MS: u64 = 1000;

/// Default maximum reconnection delay (milliseconds)
pub const DEFAULT_RECONNECT_MAX_DELAY_MS: u64 = 60000;

/// Default health check enabled state (aligns with official SDKs)
pub const DEFAULT_HEALTH_CHECK_ENABLED: bool = false;

/// Default health check interval (milliseconds)
pub const DEFAULT_HEALTH_CHECK_INTERVAL_MS: u64 = 30000;

/// Default maximum missed pongs before disconnect
pub const DEFAULT_HEALTH_CHECK_MAX_MISSED_PONGS: u64 = 2;

impl Default for ReconnectionConfig {
    fn default() -> Self {
        Self {
            max_attempts: DEFAULT_RECONNECT_MAX_ATTEMPTS,
            initial_delay: Duration::from_millis(DEFAULT_RECONNECT_INITIAL_DELAY_MS),
            max_delay: Duration::from_millis(DEFAULT_RECONNECT_MAX_DELAY_MS),
        }
    }
}
```

### Pattern 3: Helpful Error Messages with Context
**What:** Error messages include field name, invalid value, constraint, and hint
**When to use:** Always for user-facing configuration errors
**Example:**
```rust
// Source: AWS SDK error message patterns
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Invalid {field}: {value}\n  Constraint: {constraint}")]
    InvalidField {
        field: &'static str,
        value: String,
        constraint: &'static str,
    },

    #[error("Configuration constraint violation: {constraint}\n  Fields: {fields}\n  Hint: {hint}")]
    ConstraintViolation {
        fields: String,
        constraint: &'static str,
        hint: &'static str,
    },
}

impl ConfigError {
    pub fn invalid_field(field: &'static str, value: &str, constraint: &'static str) -> Self {
        Self::InvalidField {
            field,
            value: value.to_string(),
            constraint,
        }
    }

    pub fn constraint_violation(fields: &[&'static str], constraint: &'static str) -> Self {
        Self::ConstraintViolation {
            fields: fields.join(", "),
            constraint,
            hint: "Check configuration reference documentation",
        }
    }
}
```

### Anti-Patterns to Avoid
- **Lazy validation:** Validating at `connect()` instead of construction - causes delayed error detection
- **Generic errors:** `"Invalid config"` without field name or constraint - unhelpful for debugging
- **Hardcoded defaults:** Defaults embedded in code without constants - prevents cross-language consistency
- **Silent coercion:** Converting invalid values (e.g., `max_attempts = 0` → `1`) - hides configuration mistakes

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Error type derivation | Manual Display/Error impl | `thiserror` crate | Already in dependencies, handles all boilerplate |
| Config serialization | Custom to_dict/from_dict | Keep as Rust-only structs | FFI layers handle conversion, not core |
| Validation rules DSL | Custom macro system | Simple functions | Over-engineering for ~10 validation rules |

**Key insight:** Configuration validation is straightforward bounds checking and constraint validation. Don't introduce complexity where simple `if` statements suffice.

## Common Pitfalls

### Pitfall 1: Health Check Default Misalignment
**What goes wrong:** Core defaults `health_check.enabled = true`, but official Python/Node.js SDKs default to `false`. Users migrating from official SDKs get unexpected behavior.

**Why it happens:**
- Core implementation predates API compatibility research
- Default seemed "safer" (enabled by default)
- Official SDKs chose `false` to avoid unnecessary network overhead

**How to avoid:**
- Change `HealthCheckConfig::default()` to use `enabled: false`
- Update constant to `pub const DEFAULT_HEALTH_CHECK_ENABLED: bool = false;`
- Document rationale: "Aligns with official Fugle Python/Node.js SDKs"

**Warning signs:**
- User reports: "health check pings happening when I didn't enable them"
- Cross-language tests failing due to default behavior differences

### Pitfall 2: Invalid Config Accepted at Construction
**What goes wrong:** Configuration with `max_attempts = 0` or `timeout >= interval` accepted at construction, fails at runtime with cryptic error.

**Why it happens:**
- Current implementation has no validation in constructors
- Validation happens implicitly during connection (if at all)
- Error context lost between construction and usage

**How to avoid:**
- Add validation to `new()` constructors
- Add validation to builder methods (`with_max_attempts()`, etc.)
- Return `Result<Self, ConfigError>` with detailed errors

**Warning signs:**
- No tests for invalid configurations
- Constructor signatures don't return `Result`
- Error messages like "connection failed" without mentioning config issue

### Pitfall 3: Cross-Language Default Inconsistency
**What goes wrong:** Python binding uses different defaults than Rust core. Same config behaves differently in different languages.

**Why it happens:**
- Defaults hardcoded in each language binding
- No single source of truth
- No automated tests enforcing consistency

**How to avoid:**
- Export defaults as public constants in Rust core
- Language bindings import/reference these constants
- Add cross-language config tests (Phase 2-4)

**Warning signs:**
- Each binding has its own default value definitions
- No test comparing defaults across languages
- Documentation shows different defaults per language

## Code Examples

Verified patterns from current codebase and official SDK patterns:

### Current Implementation (Before Phase 1)
```rust
// core/src/websocket/reconnection.rs (lines 19-27)
impl Default for ReconnectionConfig {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
        }
    }
}
```

### After Phase 1: With Validation and Public Constants
```rust
// core/src/websocket/reconnection.rs

/// Default maximum reconnection attempts (VAL-02)
pub const DEFAULT_MAX_ATTEMPTS: u32 = 5;

/// Default initial reconnection delay in milliseconds (VAL-02)
pub const DEFAULT_INITIAL_DELAY_MS: u64 = 1000;

/// Default maximum reconnection delay in milliseconds (VAL-02)
pub const DEFAULT_MAX_DELAY_MS: u64 = 60000;

/// Minimum allowed initial delay to prevent connection storms
pub const MIN_INITIAL_DELAY_MS: u64 = 100;

impl Default for ReconnectionConfig {
    fn default() -> Self {
        Self {
            max_attempts: DEFAULT_MAX_ATTEMPTS,
            initial_delay: Duration::from_millis(DEFAULT_INITIAL_DELAY_MS),
            max_delay: Duration::from_millis(DEFAULT_MAX_DELAY_MS),
        }
    }
}

impl ReconnectionConfig {
    /// Create a new reconnection config with validation
    pub fn new(max_attempts: u32, initial_delay: Duration, max_delay: Duration) -> Result<Self, MarketDataError> {
        // Validate constraints (VAL-02)
        if max_attempts == 0 {
            return Err(MarketDataError::Config(
                "max_attempts must be >= 1".to_string()
            ));
        }

        if initial_delay < Duration::from_millis(MIN_INITIAL_DELAY_MS) {
            return Err(MarketDataError::Config(
                format!(
                    "initial_delay must be >= {}ms (got {}ms)",
                    MIN_INITIAL_DELAY_MS,
                    initial_delay.as_millis()
                )
            ));
        }

        if max_delay < initial_delay {
            return Err(MarketDataError::Config(
                format!(
                    "max_delay ({}ms) must be >= initial_delay ({}ms)",
                    max_delay.as_millis(),
                    initial_delay.as_millis()
                )
            ));
        }

        Ok(Self {
            max_attempts,
            initial_delay,
            max_delay,
        })
    }

    /// Validate and set max_attempts (VAL-02, VAL-04)
    pub fn with_max_attempts(mut self, max_attempts: u32) -> Result<Self, MarketDataError> {
        if max_attempts == 0 {
            return Err(MarketDataError::Config(
                "max_attempts must be >= 1".to_string()
            ));
        }
        self.max_attempts = max_attempts;
        Ok(self)
    }

    /// Validate and set initial_delay (VAL-02, VAL-04)
    pub fn with_initial_delay(mut self, initial_delay: Duration) -> Result<Self, MarketDataError> {
        if initial_delay < Duration::from_millis(MIN_INITIAL_DELAY_MS) {
            return Err(MarketDataError::Config(
                format!(
                    "initial_delay must be >= {}ms (got {}ms)",
                    MIN_INITIAL_DELAY_MS,
                    initial_delay.as_millis()
                )
            ));
        }
        self.initial_delay = initial_delay;
        Ok(self)
    }

    /// Validate and set max_delay (VAL-02, VAL-04)
    pub fn with_max_delay(mut self, max_delay: Duration) -> Result<Self, MarketDataError> {
        if max_delay < self.initial_delay {
            return Err(MarketDataError::Config(
                format!(
                    "max_delay ({}ms) must be >= initial_delay ({}ms)",
                    max_delay.as_millis(),
                    self.initial_delay.as_millis()
                )
            ));
        }
        self.max_delay = max_delay;
        Ok(self)
    }
}
```

### Health Check Config with Aligned Defaults
```rust
// core/src/websocket/health_check.rs

/// Default health check enabled state - aligns with official SDKs (CON-01)
pub const DEFAULT_ENABLED: bool = false;

/// Default health check interval in milliseconds (CON-01)
pub const DEFAULT_INTERVAL_MS: u64 = 30000;

/// Default maximum missed pongs before disconnect (CON-01)
pub const DEFAULT_MAX_MISSED_PONGS: u64 = 2;

/// Minimum allowed interval to prevent excessive overhead
pub const MIN_INTERVAL_MS: u64 = 5000;

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            enabled: DEFAULT_ENABLED,  // CHANGED: was true, now false
            interval: Duration::from_millis(DEFAULT_INTERVAL_MS),
            max_missed_pongs: DEFAULT_MAX_MISSED_PONGS,
        }
    }
}

impl HealthCheckConfig {
    /// Create health check config with validation (VAL-03)
    pub fn new(enabled: bool, interval: Duration, max_missed_pongs: u64) -> Result<Self, MarketDataError> {
        if interval < Duration::from_millis(MIN_INTERVAL_MS) {
            return Err(MarketDataError::Config(
                format!(
                    "health_check interval must be >= {}ms (got {}ms)",
                    MIN_INTERVAL_MS,
                    interval.as_millis()
                )
            ));
        }

        if max_missed_pongs == 0 {
            return Err(MarketDataError::Config(
                "max_missed_pongs must be >= 1".to_string()
            ));
        }

        Ok(Self {
            enabled,
            interval,
            max_missed_pongs,
        })
    }

    /// Validate and set interval (VAL-03, VAL-04)
    pub fn with_interval(mut self, interval: Duration) -> Result<Self, MarketDataError> {
        if interval < Duration::from_millis(MIN_INTERVAL_MS) {
            return Err(MarketDataError::Config(
                format!(
                    "health_check interval must be >= {}ms (got {}ms)",
                    MIN_INTERVAL_MS,
                    interval.as_millis()
                )
            ));
        }
        self.interval = interval;
        Ok(self)
    }

    /// Validate and set max_missed_pongs (VAL-04)
    pub fn with_max_missed_pongs(mut self, max: u64) -> Result<Self, MarketDataError> {
        if max == 0 {
            return Err(MarketDataError::Config(
                "max_missed_pongs must be >= 1".to_string()
            ));
        }
        self.max_missed_pongs = max;
        Ok(self)
    }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| No validation | Fail-fast validation | v0.3.0 | Better error messages, earlier error detection |
| Hardcoded defaults | Public constants | v0.3.0 | Cross-language consistency possible |
| `health_check.enabled = true` | `health_check.enabled = false` | v0.3.0 | Aligns with official SDKs |
| Builder returns Self | Builder returns Result | v0.3.0 | Can't set invalid config via builder |

**Deprecated/outdated:**
- No items deprecated in Phase 1 (additive changes only)

## Open Questions

1. **Should `ConnectionConfig` also get validation?**
   - What we know: `ConnectionConfig` has timeouts but no validation currently
   - What's unclear: Whether REST/WebSocket timeouts need relationship validation
   - Recommendation: Defer to Phase 2-4 when language bindings expose these configs

2. **How should validation errors be represented?**
   - What we know: `MarketDataError` already exists in `errors.rs`
   - What's unclear: Whether to add specific `ConfigError` variant or use existing patterns
   - Recommendation: Extend `MarketDataError::Config(String)` with formatted messages, avoid new error types

3. **Should we validate at both construction and builder methods?**
   - What we know: Builder methods currently don't validate
   - What's unclear: Performance impact of validating on every builder call
   - Recommendation: Validate in both places for safety, overhead is negligible for config operations

## Sources

### Primary (HIGH confidence)
- `core/src/websocket/reconnection.rs` - Current implementation
- `core/src/websocket/health_check.rs` - Current implementation
- `core/src/websocket/config.rs` - Current implementation
- `.planning/research/v0.3.0-SUMMARY.md` - Official SDK defaults research
- `.planning/research/PITFALLS.md` - Configuration validation pitfalls
- `.planning/REQUIREMENTS.md` - VAL-01 through VAL-04, CON-01 requirements

### Secondary (MEDIUM confidence)
- AWS SDK Configuration Validation Patterns
- Azure SDK Language Design Guidelines
- Rust API Guidelines (https://rust-lang.github.io/api-guidelines/)

### Tertiary (LOW confidence)
- None (all findings verified with codebase and official SDK research)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Based on existing dependencies and Rust stdlib
- Architecture: HIGH - Validated against current codebase structure
- Pitfalls: HIGH - Derived from existing research in PITFALLS.md
- Default alignment: HIGH - Verified against official Python/Node.js SDKs

**Research date:** 2026-02-01
**Valid until:** 90 days (stable domain, unlikely to change)

---

## Implementation Checklist for Planner

Phase 1 tasks should include:

- [ ] Add public default constants to `reconnection.rs`
- [ ] Add public default constants to `health_check.rs`
- [ ] Change `HealthCheckConfig::default().enabled` from `true` to `false`
- [ ] Add validation to `ReconnectionConfig::new()`
- [ ] Add validation to `ReconnectionConfig` builder methods
- [ ] Add validation to `HealthCheckConfig::new()`
- [ ] Add validation to `HealthCheckConfig` builder methods
- [ ] Update `lib.rs` to export new constants
- [ ] Add tests for invalid configurations
- [ ] Add tests for constraint violations
- [ ] Add tests for error message clarity
- [ ] Document validation constraints in rustdoc

**Requirements addressed:** VAL-01, VAL-02, VAL-03, VAL-04, CON-01
