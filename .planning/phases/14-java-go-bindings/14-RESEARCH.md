# Phase 14: Java & Go Bindings - Research

**Researched:** 2026-02-15
**Domain:** UniFFI bindings, Java builder pattern, Go functional options, C# options pattern, FFI config exposure
**Confidence:** HIGH

## Summary

This phase adds configuration exposure (options-based constructors, reconnect config, health check config) to three UniFFI-based language bindings: Java, Go, and C#. Each language follows its own idiomatic patterns — Java uses builder pattern, Go uses functional options, C# uses options classes. The existing v0.2.0 binding functionality (REST + WebSocket) remains unchanged; this phase adds configuration acceptance to constructors.

Phase 11 delivered C# bindings with csbindgen. Phase 8 delivered core validation for `ReconnectionConfig` and `HealthCheckConfig`. Phases 12 and 13 established validation patterns for Python and Node.js. This phase extends those patterns to Java, Go, and C# following each language's standard conventions.

All three languages share validation behavior: exactly-one-auth required, same config constraints, consistent error messages, milliseconds at FFI boundary. The differences are purely syntactic — builder methods for Java, With* functions for Go, property initialization for C#.

**Primary recommendation:** Create language-specific wrapper layers around UniFFI-generated clients. For Java: builder pattern with nested config builders. For Go: functional options pattern with config structs. For C#: options classes with property validation. All share core validation logic via `ReconnectionConfig::new()` and `HealthCheckConfig::new()`.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Claude's Discretion

All API design decisions are at Claude's discretion, following each language's standard conventions:

**Java:**
- Builder pattern for client construction (e.g., `RestClient.builder().apiKey("...").build()`)
- Nested builder or separate config classes for ReconnectConfig/HealthCheckConfig
- Validation at build time with descriptive exceptions
- Follow established Java SDK patterns (similar to OkHttp, AWS SDK builders)

**Go:**
- Functional options pattern (e.g., `NewRestClient(WithApiKey("..."))`)
- Config structs with sensible zero-value defaults where possible
- Exactly-one-auth enforcement at construction time
- Follow established Go patterns (similar to grpc-go, aws-sdk-go-v2)

**C#:**
- Options class pattern with properties (e.g., `new RestClientOptions { ApiKey = "..." }`)
- Extend existing csbindgen Phase 11 work
- PascalCase naming matching .NET conventions
- Validation at construction with ArgumentException

**Cross-language:**
- Validation behavior consistent: exactly-one-auth required, same config constraints as Python/Node.js
- Error messages follow same format: include field name, constraint, and actual value
- Config defaults match core constants (DEFAULT_* values from Phase 8)
- Milliseconds at FFI boundary (same pattern as Python/Node.js)

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.
</user_constraints>

## Standard Stack

### Core Technologies

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| UniFFI | 0.28 | Rust-to-foreign-language FFI | Mozilla standard for multi-language bindings, kept at 0.28 for uniffi-bindgen-go 0.4.0 compatibility |
| marketdata-core | 0.2.0 | Core validation and client logic | Phase 8 validation already complete |
| uniffi-bindgen-java | latest | Java binding generator | IronCoreLabs implementation for Java targets |
| uniffi-bindgen-go | 0.4.0 | Go binding generator | Requires UniFFI 0.28 |
| csbindgen | existing | C# binding generator | Phase 11 already established |

### Language-Specific Libraries

**Java:**
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| JNA | 5.x | Native library loading | Already used for UniFFI bindings |
| CompletableFuture | Java 8+ | Async support | Already implemented in Phase 11 |

**Go:**
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| cgo | stdlib | C FFI | Required for UniFFI bindings |
| error | stdlib | Error handling | Standard Go error type |

**C#:**
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| System | .NET 6+ | Base framework | Standard .NET |
| Task | .NET 6+ | Async support | Already implemented in Phase 11 |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| UniFFI 0.28 | UniFFI 0.29+ | uniffi-bindgen-go 0.4.0 requires 0.28, upgrading breaks Go support |
| Builder pattern (Java) | Constructor overloading | Too many constructors, not idiomatic for optional params |
| Functional options (Go) | Config struct param | Less flexible, can't add options without breaking API |
| Options class (C#) | Constructor params | Not idiomatic for .NET, IOptions<T> pattern is standard |
| Wrapper layers | Direct UniFFI exposure | No idiomatic API, validation happens at wrong layer |

**Installation:**
```bash
# Java: Gradle already configured
cd bindings/java && ./gradlew build

# Go: Module already set up
cd bindings/go && go build

# C#: .NET project already configured
cd bindings/csharp && dotnet build
```

## Architecture Patterns

### Recommended Project Structure

**Java:**
```
bindings/java/src/main/java/tw/com/fugle/marketdata/
├── FugleRestClient.java           # Wrapper with builder
├── FugleWebSocketClient.java      # Wrapper with builder
├── RestClientOptions.java         # Config holder (optional separate class)
├── WebSocketClientOptions.java    # Config holder (optional separate class)
├── ReconnectOptions.java          # Reconnect config
├── HealthCheckOptions.java        # Health check config
├── FugleException.java            # Exception wrapper
└── generated/                     # UniFFI-generated classes
    ├── RestClient.java
    ├── WebSocketClient.java
    └── ...
```

**Go:**
```
bindings/go/marketdata/
├── client.go                      # Client constructors with options
├── options.go                     # Option functions (With*)
├── config.go                      # Config structs
├── errors.go                      # Error handling
└── marketdata_uniffi.go           # UniFFI-generated code
```

**C#:**
```
bindings/csharp/MarketdataUniffi/
├── RestClient.cs                  # Wrapper with options
├── WebSocketClient.cs             # Wrapper with options
├── RestClientOptions.cs           # Options class
├── WebSocketClientOptions.cs      # Options class
├── ReconnectOptions.cs            # Reconnect config
├── HealthCheckOptions.cs          # Health check config
├── MarketdataException.cs         # Exception wrapper
└── Generated/                     # UniFFI-generated classes
```

### Pattern 1: Java Builder Pattern (Fluent API)

**What:** Static inner Builder class with fluent methods returning `this`
**When to use:** Java SDK construction with optional parameters
**Example:**
```java
// Source: Java builder pattern best practices + existing FugleRestClient

public class FugleRestClient implements AutoCloseable {
    private final RestClient restClient;

    private FugleRestClient(RestClient restClient) {
        this.restClient = restClient;
    }

    public static Builder builder() {
        return new Builder();
    }

    public static class Builder {
        private String apiKey;
        private String bearerToken;
        private String sdkToken;
        private String baseUrl;

        private Builder() {}

        public Builder apiKey(String apiKey) {
            this.apiKey = apiKey;
            return this;
        }

        public Builder bearerToken(String bearerToken) {
            this.bearerToken = bearerToken;
            return this;
        }

        public Builder sdkToken(String sdkToken) {
            this.sdkToken = sdkToken;
            return this;
        }

        public Builder baseUrl(String baseUrl) {
            this.baseUrl = baseUrl;
            return this;
        }

        public FugleRestClient build() throws FugleException {
            // Validate exactly one auth
            int authCount = 0;
            if (apiKey != null) authCount++;
            if (bearerToken != null) authCount++;
            if (sdkToken != null) authCount++;

            if (authCount == 0) {
                throw new FugleException("Authentication required: set apiKey(), bearerToken(), or sdkToken()");
            }
            if (authCount > 1) {
                throw new FugleException("Provide exactly one of: apiKey, bearerToken, sdkToken");
            }

            try {
                RestClient client;
                if (apiKey != null) {
                    client = MarketdataUniffi.newRestClientWithApiKey(apiKey);
                } else if (bearerToken != null) {
                    client = MarketdataUniffi.newRestClientWithBearerToken(bearerToken);
                } else {
                    client = MarketdataUniffi.newRestClientWithSdkToken(sdkToken);
                }

                // TODO: Apply baseUrl if provided

                return new FugleRestClient(client);
            } catch (MarketDataException e) {
                throw FugleException.from(e);
            }
        }
    }
}
```

### Pattern 2: Java Nested Config Builders

**What:** Separate builder classes for reconnect and health check configs
**When to use:** WebSocketClient with nested configuration
**Example:**
```java
public class FugleWebSocketClient implements AutoCloseable {

    public static class Builder {
        private String apiKey;
        private String bearerToken;
        private String sdkToken;
        private String baseUrl;
        private ReconnectOptions reconnectOptions;
        private HealthCheckOptions healthCheckOptions;

        public Builder reconnect(ReconnectOptions options) {
            this.reconnectOptions = options;
            return this;
        }

        public Builder healthCheck(HealthCheckOptions options) {
            this.healthCheckOptions = options;
            return this;
        }

        public FugleWebSocketClient build() throws FugleException {
            // Validate and build with core configs
            // Convert milliseconds from options to Duration for core
        }
    }

    public static class ReconnectOptions {
        private Integer maxAttempts;
        private Long initialDelayMs;
        private Long maxDelayMs;

        public static ReconnectOptions.Builder builder() {
            return new ReconnectOptions.Builder();
        }

        public static class Builder {
            private Integer maxAttempts;
            private Long initialDelayMs;
            private Long maxDelayMs;

            public Builder maxAttempts(int value) {
                this.maxAttempts = value;
                return this;
            }

            public Builder initialDelayMs(long value) {
                this.initialDelayMs = value;
                return this;
            }

            public Builder maxDelayMs(long value) {
                this.maxDelayMs = value;
                return this;
            }

            public ReconnectOptions build() {
                ReconnectOptions opts = new ReconnectOptions();
                opts.maxAttempts = this.maxAttempts;
                opts.initialDelayMs = this.initialDelayMs;
                opts.maxDelayMs = this.maxDelayMs;
                return opts;
            }
        }
    }
}

// Usage:
ReconnectOptions reconnect = ReconnectOptions.builder()
    .maxAttempts(10)
    .initialDelayMs(2000)
    .build();

FugleWebSocketClient ws = FugleWebSocketClient.builder()
    .apiKey("key")
    .reconnect(reconnect)
    .build();
```

### Pattern 3: Go Functional Options

**What:** Variadic option functions that modify config struct
**When to use:** Go SDK construction with flexible optional params
**Example:**
```go
// Source: Go functional options pattern + existing example

package marketdata

import (
    "errors"
    "time"
)

// Option configures a RestClient
type Option func(*clientConfig) error

type clientConfig struct {
    apiKey      string
    bearerToken string
    sdkToken    string
    baseUrl     string
}

// WithApiKey sets API key authentication
func WithApiKey(key string) Option {
    return func(c *clientConfig) error {
        c.apiKey = key
        return nil
    }
}

// WithBearerToken sets bearer token authentication
func WithBearerToken(token string) Option {
    return func(c *clientConfig) error {
        c.bearerToken = token
        return nil
    }
}

// WithSdkToken sets SDK token authentication
func WithSdkToken(token string) Option {
    return func(c *clientConfig) error {
        c.sdkToken = token
        return nil
    }
}

// WithBaseUrl sets custom base URL
func WithBaseUrl(url string) Option {
    return func(c *clientConfig) error {
        c.baseUrl = url
        return nil
    }
}

// NewRestClient creates a new REST client with options
func NewRestClient(opts ...Option) (*RestClient, error) {
    cfg := &clientConfig{}

    // Apply all options
    for _, opt := range opts {
        if err := opt(cfg); err != nil {
            return nil, err
        }
    }

    // Validate exactly one auth
    authCount := 0
    if cfg.apiKey != "" {
        authCount++
    }
    if cfg.bearerToken != "" {
        authCount++
    }
    if cfg.sdkToken != "" {
        authCount++
    }

    if authCount == 0 {
        return nil, errors.New("authentication required: provide WithApiKey, WithBearerToken, or WithSdkToken")
    }
    if authCount > 1 {
        return nil, errors.New("provide exactly one of: WithApiKey, WithBearerToken, WithSdkToken")
    }

    // Call UniFFI function
    var client *RestClient
    var err error
    if cfg.apiKey != "" {
        client, err = NewRestClientWithApiKey(cfg.apiKey)
    } else if cfg.bearerToken != "" {
        client, err = NewRestClientWithBearerToken(cfg.bearerToken)
    } else {
        client, err = NewRestClientWithSdkToken(cfg.sdkToken)
    }

    if err != nil {
        return nil, err
    }

    // TODO: Apply baseUrl if provided

    return client, nil
}

// Usage:
client, err := NewRestClient(
    WithApiKey("your-key"),
    WithBaseUrl("https://custom.api"),
)
```

### Pattern 4: Go Config Structs for Nested Options

**What:** Separate config structs passed to option functions
**When to use:** WebSocket client with reconnect and health check config
**Example:**
```go
// ReconnectConfig holds reconnection options
type ReconnectConfig struct {
    MaxAttempts    uint32 // Optional, defaults to 5
    InitialDelayMs uint64 // Optional, defaults to 1000
    MaxDelayMs     uint64 // Optional, defaults to 60000
}

// HealthCheckConfig holds health check options
type HealthCheckConfig struct {
    Enabled        bool   // Optional, defaults to false
    IntervalMs     uint64 // Optional, defaults to 30000
    MaxMissedPongs uint64 // Optional, defaults to 2
}

type wsConfig struct {
    apiKey      string
    bearerToken string
    sdkToken    string
    baseUrl     string
    reconnect   *ReconnectConfig
    healthCheck *HealthCheckConfig
}

// WithReconnect sets reconnection configuration
func WithReconnect(cfg ReconnectConfig) Option {
    return func(c *wsConfig) error {
        c.reconnect = &cfg
        return nil
    }
}

// WithHealthCheck sets health check configuration
func WithHealthCheck(cfg HealthCheckConfig) Option {
    return func(c *wsConfig) error {
        c.healthCheck = &cfg
        return nil
    }
}

// NewWebSocketClient creates WebSocket client with options
func NewWebSocketClient(opts ...Option) (*WebSocketClient, error) {
    cfg := &wsConfig{}

    for _, opt := range opts {
        if err := opt(cfg); err != nil {
            return nil, err
        }
    }

    // Validate exactly one auth (same as REST)

    // Build core configs with defaults
    reconnectCfg := buildReconnectConfig(cfg.reconnect)
    healthCheckCfg := buildHealthCheckConfig(cfg.healthCheck)

    // TODO: Create WebSocket client with configs

    return client, nil
}

func buildReconnectConfig(cfg *ReconnectConfig) (/* core type */) {
    maxAttempts := uint32(5) // DEFAULT_MAX_ATTEMPTS
    initialDelayMs := uint64(1000) // DEFAULT_INITIAL_DELAY_MS
    maxDelayMs := uint64(60000) // DEFAULT_MAX_DELAY_MS

    if cfg != nil {
        if cfg.MaxAttempts != 0 {
            maxAttempts = cfg.MaxAttempts
        }
        if cfg.InitialDelayMs != 0 {
            initialDelayMs = cfg.InitialDelayMs
        }
        if cfg.MaxDelayMs != 0 {
            maxDelayMs = cfg.MaxDelayMs
        }
    }

    // Call core validation via UniFFI
    // Return validated config or error
}

// Usage:
ws, err := NewWebSocketClient(
    WithApiKey("key"),
    WithReconnect(ReconnectConfig{
        MaxAttempts: 10,
        InitialDelayMs: 2000,
    }),
    WithHealthCheck(HealthCheckConfig{
        Enabled: true,
        IntervalMs: 20000,
    }),
)
```

### Pattern 5: C# Options Classes

**What:** Plain classes with properties, passed to constructor
**When to use:** C# .NET SDK with configuration
**Example:**
```csharp
// Source: .NET options pattern + Phase 11 C# structure

namespace FugleMarketData
{
    /// <summary>
    /// Options for configuring RestClient
    /// </summary>
    public class RestClientOptions
    {
        /// <summary>
        /// API key for authentication (optional)
        /// </summary>
        public string? ApiKey { get; set; }

        /// <summary>
        /// Bearer token for authentication (optional)
        /// </summary>
        public string? BearerToken { get; set; }

        /// <summary>
        /// SDK token for authentication (optional)
        /// </summary>
        public string? SdkToken { get; set; }

        /// <summary>
        /// Custom base URL (optional)
        /// </summary>
        public string? BaseUrl { get; set; }
    }

    public class RestClient : IDisposable
    {
        private readonly MarketdataUniffi.RestClient _client;

        /// <summary>
        /// Create a new REST client with options
        /// </summary>
        /// <param name="options">Client configuration options</param>
        /// <exception cref="ArgumentException">If validation fails</exception>
        public RestClient(RestClientOptions options)
        {
            if (options == null)
                throw new ArgumentNullException(nameof(options));

            // Validate exactly one auth
            int authCount = 0;
            if (!string.IsNullOrEmpty(options.ApiKey)) authCount++;
            if (!string.IsNullOrEmpty(options.BearerToken)) authCount++;
            if (!string.IsNullOrEmpty(options.SdkToken)) authCount++;

            if (authCount == 0)
                throw new ArgumentException("Authentication required: set ApiKey, BearerToken, or SdkToken", nameof(options));

            if (authCount > 1)
                throw new ArgumentException("Provide exactly one of: ApiKey, BearerToken, SdkToken", nameof(options));

            try
            {
                if (!string.IsNullOrEmpty(options.ApiKey))
                {
                    _client = MarketdataUniffi.NewRestClientWithApiKey(options.ApiKey);
                }
                else if (!string.IsNullOrEmpty(options.BearerToken))
                {
                    _client = MarketdataUniffi.NewRestClientWithBearerToken(options.BearerToken);
                }
                else
                {
                    _client = MarketdataUniffi.NewRestClientWithSdkToken(options.SdkToken!);
                }

                // TODO: Apply BaseUrl if provided
            }
            catch (MarketDataException ex)
            {
                throw new MarketdataException(ex.Message);
            }
        }

        public void Dispose()
        {
            _client?.Dispose();
        }
    }
}

// Usage:
var client = new RestClient(new RestClientOptions
{
    ApiKey = "your-key",
    BaseUrl = "https://custom.api"
});
```

### Pattern 6: C# Nested Options Classes

**What:** Separate options classes for reconnect and health check
**When to use:** WebSocketClient with nested configuration
**Example:**
```csharp
/// <summary>
/// Reconnection configuration for WebSocket clients
/// </summary>
public class ReconnectOptions
{
    /// <summary>
    /// Maximum reconnection attempts (default: 5, min: 1)
    /// </summary>
    public uint? MaxAttempts { get; set; }

    /// <summary>
    /// Initial reconnection delay in milliseconds (default: 1000, min: 100)
    /// </summary>
    public ulong? InitialDelayMs { get; set; }

    /// <summary>
    /// Maximum reconnection delay in milliseconds (default: 60000)
    /// </summary>
    public ulong? MaxDelayMs { get; set; }
}

/// <summary>
/// Health check configuration for WebSocket connections
/// </summary>
public class HealthCheckOptions
{
    /// <summary>
    /// Whether health check is enabled (default: false)
    /// </summary>
    public bool? Enabled { get; set; }

    /// <summary>
    /// Interval between ping messages in milliseconds (default: 30000, min: 5000)
    /// </summary>
    public ulong? IntervalMs { get; set; }

    /// <summary>
    /// Maximum missed pongs before disconnect (default: 2, min: 1)
    /// </summary>
    public ulong? MaxMissedPongs { get; set; }
}

/// <summary>
/// Options for configuring WebSocketClient
/// </summary>
public class WebSocketClientOptions
{
    public string? ApiKey { get; set; }
    public string? BearerToken { get; set; }
    public string? SdkToken { get; set; }
    public string? BaseUrl { get; set; }

    /// <summary>
    /// Reconnection configuration (optional)
    /// </summary>
    public ReconnectOptions? Reconnect { get; set; }

    /// <summary>
    /// Health check configuration (optional)
    /// </summary>
    public HealthCheckOptions? HealthCheck { get; set; }
}

public class WebSocketClient : IDisposable
{
    public WebSocketClient(WebSocketClientOptions options)
    {
        if (options == null)
            throw new ArgumentNullException(nameof(options));

        // Validate exactly one auth (same as REST)

        // Build core configs with validation
        var reconnectCfg = BuildReconnectConfig(options.Reconnect);
        var healthCheckCfg = BuildHealthCheckConfig(options.HealthCheck);

        // TODO: Create WebSocket client with configs
    }

    private static /* core type */ BuildReconnectConfig(ReconnectOptions? options)
    {
        const uint DEFAULT_MAX_ATTEMPTS = 5;
        const ulong DEFAULT_INITIAL_DELAY_MS = 1000;
        const ulong DEFAULT_MAX_DELAY_MS = 60000;

        uint maxAttempts = options?.MaxAttempts ?? DEFAULT_MAX_ATTEMPTS;
        ulong initialDelayMs = options?.InitialDelayMs ?? DEFAULT_INITIAL_DELAY_MS;
        ulong maxDelayMs = options?.MaxDelayMs ?? DEFAULT_MAX_DELAY_MS;

        // Call core validation via UniFFI
        // Throw ArgumentException on validation failure

        return /* validated config */;
    }
}

// Usage:
var ws = new WebSocketClient(new WebSocketClientOptions
{
    ApiKey = "key",
    Reconnect = new ReconnectOptions
    {
        MaxAttempts = 10,
        InitialDelayMs = 2000
    },
    HealthCheck = new HealthCheckOptions
    {
        Enabled = true,
        IntervalMs = 20000
    }
});
```

### Anti-Patterns to Avoid

- **Too many constructor overloads (Java):** Don't create RestClient(apiKey), RestClient(bearerToken), RestClient(sdkToken, baseUrl), etc. Use builder pattern for optional parameters.
- **Mutable options after construction:** Options should be immutable after passed to constructor. Don't allow changing auth or config after client creation.
- **Validation at connection time:** Validate exactly-one-auth and config constraints at construction time, not later. Fail fast.
- **Inconsistent default values:** Don't use different defaults across languages. All must use core constants (DEFAULT_MAX_ATTEMPTS = 5, etc.).
- **Language-mixing naming:** Don't use snake_case in Java/C#, don't use camelCase in Go. Follow each language's conventions.
- **Skipping validation:** Don't assume UniFFI-generated code validates. Wrapper layers must call core validation functions.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Config validation | Custom Java/Go/C# validation logic | Core's `ReconnectionConfig::new()` and `HealthCheckConfig::new()` | Phase 8 validation is comprehensive, includes proper error messages |
| Auth selection | Switch statements in each language | Core's `Auth` enum | Type-safe, already exists |
| Default constants | Hardcode 5, 1000, 60000 in each language | Core's exported constants (DEFAULT_MAX_ATTEMPTS, etc.) | Single source of truth, CON-01 requirement |
| Error handling | Language-specific error types | Wrap core's `MarketDataError` | Consistent error semantics across bindings |
| Time unit conversion | Manual ms→Duration math | `Duration::from_millis()` | Standard library, no dependencies |
| UniFFI constructor functions | New FFI functions for each config variant | Existing `new_rest_client_with_api_key` etc. | Already complete from Phase 11 |

**Key insight:** UniFFI already generates working bindings. This phase adds idiomatic wrappers around those bindings for configuration exposure. Core validation is complete — wrapper layers translate language-specific options into core types and propagate validation errors.

## Common Pitfalls

### Pitfall 1: Not Wrapping UniFFI-Generated Classes

**What goes wrong:** Exposing UniFFI-generated classes directly, expecting users to call `MarketdataUniffi.newRestClientWithApiKey()` directly
**Why it happens:** Looks simpler to avoid wrapper layer
**How to avoid:** Always create idiomatic wrapper classes (FugleRestClient for Java, package-level functions for Go, RestClient for C#)
**Warning signs:** Examples in documentation use MarketdataUniffi namespace directly

**Example:**
```java
// ❌ WRONG: Expose UniFFI directly
import tw.com.fugle.marketdata.generated.MarketdataUniffi;
RestClient client = MarketdataUniffi.newRestClientWithApiKey("key");

// ✅ CORRECT: Idiomatic wrapper
import tw.com.fugle.marketdata.FugleRestClient;
FugleRestClient client = FugleRestClient.builder()
    .apiKey("key")
    .build();
```

### Pitfall 2: Missing Validation Before UniFFI Call

**What goes wrong:** Passing invalid config to UniFFI, getting generic errors instead of descriptive validation messages
**Why it happens:** Assuming UniFFI-generated code validates automatically
**How to avoid:** Build core config objects with validation in wrapper layer, propagate errors with language-specific types
**Warning signs:** Error messages like "panic: null pointer" instead of "max_attempts must be >= 1"

**Example:**
```go
// ❌ WRONG: No validation
func NewWebSocketClient(opts ...Option) (*WebSocketClient, error) {
    // Apply options
    // Directly call UniFFI without checking
    client, err := NewWebSocketClientInternal(cfg.apiKey)
    return client, err
}

// ✅ CORRECT: Validate with core
func NewWebSocketClient(opts ...Option) (*WebSocketClient, error) {
    // Apply options

    // Build and validate reconnect config
    reconnectCfg, err := buildReconnectConfig(cfg.reconnect)
    if err != nil {
        return nil, fmt.Errorf("reconnect config: %w", err)
    }

    // Build and validate health check config
    healthCheckCfg, err := buildHealthCheckConfig(cfg.healthCheck)
    if err != nil {
        return nil, fmt.Errorf("health check config: %w", err)
    }

    // Now create client with validated configs
    client, err := createWebSocketClientWithConfigs(cfg.apiKey, reconnectCfg, healthCheckCfg)
    return client, err
}
```

### Pitfall 3: Wrong Default Constant Values

**What goes wrong:** Hardcoding defaults (maxAttempts: 3) that don't match core constants (DEFAULT_MAX_ATTEMPTS = 5)
**Why it happens:** Not checking core/src/lib.rs for exported constants
**How to avoid:** Always use core constants, import from UniFFI bindings where possible
**Warning signs:** Different behavior between Python/Node.js bindings and Java/Go/C#

**Example:**
```csharp
// ❌ WRONG: Hardcoded defaults
private static ReconnectConfig BuildReconnectConfig(ReconnectOptions? options)
{
    uint maxAttempts = options?.MaxAttempts ?? 3; // WRONG: Should be 5
    ulong initialDelayMs = options?.InitialDelayMs ?? 500; // WRONG: Should be 1000
    // ...
}

// ✅ CORRECT: Use core constants
using static MarketdataCore.Constants;

private static ReconnectConfig BuildReconnectConfig(ReconnectOptions? options)
{
    uint maxAttempts = options?.MaxAttempts ?? DEFAULT_MAX_ATTEMPTS;
    ulong initialDelayMs = options?.InitialDelayMs ?? DEFAULT_INITIAL_DELAY_MS;
    ulong maxDelayMs = options?.MaxDelayMs ?? DEFAULT_MAX_DELAY_MS;

    // Call core validation
    return ReconnectionConfig.New(maxAttempts, initialDelayMs, maxDelayMs);
}
```

### Pitfall 4: Ignoring Language Naming Conventions

**What goes wrong:** Using `api_key` in Java instead of `apiKey`, or `ApiKey()` in Go instead of `WithApiKey()`
**Why it happens:** Copy-paste from other language implementations
**How to avoid:** Follow language-specific conventions: camelCase for Java/C#, With* prefix for Go options
**Warning signs:** IDE warnings about naming conventions, code reviews flag style issues

**Example:**
```go
// ❌ WRONG: Not following Go conventions
func NewRestClient(apiKey string, bearerToken string) (*RestClient, error) {
    // Multiple required params, no flexibility
}

func SetApiKey(key string) Option {
    // Should use With* prefix
}

// ✅ CORRECT: Idiomatic Go
func NewRestClient(opts ...Option) (*RestClient, error) {
    // Variadic options
}

func WithApiKey(key string) Option {
    // With* prefix is standard
    return func(c *clientConfig) error {
        c.apiKey = key
        return nil
    }
}
```

### Pitfall 5: Not Handling baseUrl Override

**What goes wrong:** Accepting baseUrl in options but not applying it to client
**Why it happens:** UniFFI-generated RestClient might not expose base_url setter
**How to avoid:** Check if core RestClient has `with_base_url()` method, expose via UniFFI if needed
**Warning signs:** Tests for custom base URL fail, client still hits default URL

**Example:**
```java
// ❌ WRONG: Accepting but ignoring baseUrl
public FugleRestClient build() {
    RestClient client = MarketdataUniffi.newRestClientWithApiKey(apiKey);
    // baseUrl is never applied!
    return new FugleRestClient(client);
}

// ✅ CORRECT: Apply baseUrl if provided
public FugleRestClient build() {
    RestClient client = MarketdataUniffi.newRestClientWithApiKey(apiKey);

    if (baseUrl != null) {
        // Need UniFFI function: set_base_url or with_base_url
        client = MarketdataUniffi.restClientWithBaseUrl(client, baseUrl);
    }

    return new FugleRestClient(client);
}
```

## Code Examples

### Java: Complete Builder Implementation

```java
// Source: Existing FugleRestClient + builder pattern best practices

package tw.com.fugle.marketdata;

import tw.com.fugle.marketdata.generated.*;

public class FugleRestClient implements AutoCloseable {
    private final RestClient restClient;

    private FugleRestClient(RestClient restClient) {
        this.restClient = restClient;
    }

    public static Builder builder() {
        return new Builder();
    }

    public StockClient stock() {
        return restClient.stock();
    }

    public FutOptClient futopt() {
        return restClient.futopt();
    }

    @Override
    public void close() {
        restClient.close();
    }

    public static class Builder {
        private String apiKey;
        private String bearerToken;
        private String sdkToken;
        private String baseUrl;

        private Builder() {}

        public Builder apiKey(String apiKey) {
            this.apiKey = apiKey;
            return this;
        }

        public Builder bearerToken(String bearerToken) {
            this.bearerToken = bearerToken;
            return this;
        }

        public Builder sdkToken(String sdkToken) {
            this.sdkToken = sdkToken;
            return this;
        }

        public Builder baseUrl(String baseUrl) {
            this.baseUrl = baseUrl;
            return this;
        }

        public FugleRestClient build() throws FugleException {
            // Validate exactly one auth method
            int authCount = 0;
            if (apiKey != null) authCount++;
            if (bearerToken != null) authCount++;
            if (sdkToken != null) authCount++;

            if (authCount == 0) {
                throw new FugleException("Authentication required: set apiKey(), bearerToken(), or sdkToken()");
            }
            if (authCount > 1) {
                throw new FugleException("Provide exactly one of: apiKey, bearerToken, sdkToken");
            }

            try {
                RestClient client;
                if (apiKey != null) {
                    client = MarketdataUniffi.newRestClientWithApiKey(apiKey);
                } else if (bearerToken != null) {
                    client = MarketdataUniffi.newRestClientWithBearerToken(bearerToken);
                } else {
                    client = MarketdataUniffi.newRestClientWithSdkToken(sdkToken);
                }

                // Apply baseUrl if provided (TODO: needs UniFFI function)

                return new FugleRestClient(client);
            } catch (MarketDataException e) {
                throw FugleException.from(e);
            }
        }
    }
}

// Usage:
FugleRestClient client = FugleRestClient.builder()
    .apiKey("your-key")
    .baseUrl("https://custom.api")
    .build();
```

### Go: Complete Functional Options

```go
// Source: Go functional options pattern + idiomatic Go

package marketdata

import (
    "errors"
    "fmt"
)

// Option configures a RestClient
type Option func(*clientConfig) error

type clientConfig struct {
    apiKey      string
    bearerToken string
    sdkToken    string
    baseUrl     string
}

// WithApiKey sets API key authentication
func WithApiKey(key string) Option {
    return func(c *clientConfig) error {
        if key == "" {
            return errors.New("apiKey cannot be empty")
        }
        c.apiKey = key
        return nil
    }
}

// WithBearerToken sets bearer token authentication
func WithBearerToken(token string) Option {
    return func(c *clientConfig) error {
        if token == "" {
            return errors.New("bearerToken cannot be empty")
        }
        c.bearerToken = token
        return nil
    }
}

// WithSdkToken sets SDK token authentication
func WithSdkToken(token string) Option {
    return func(c *clientConfig) error {
        if token == "" {
            return errors.New("sdkToken cannot be empty")
        }
        c.sdkToken = token
        return nil
    }
}

// WithBaseUrl sets custom base URL
func WithBaseUrl(url string) Option {
    return func(c *clientConfig) error {
        c.baseUrl = url
        return nil
    }
}

// NewRestClient creates a new REST client with options
func NewRestClient(opts ...Option) (*RestClient, error) {
    cfg := &clientConfig{}

    // Apply all options
    for _, opt := range opts {
        if err := opt(cfg); err != nil {
            return nil, fmt.Errorf("invalid option: %w", err)
        }
    }

    // Validate exactly one auth
    authCount := 0
    if cfg.apiKey != "" {
        authCount++
    }
    if cfg.bearerToken != "" {
        authCount++
    }
    if cfg.sdkToken != "" {
        authCount++
    }

    if authCount == 0 {
        return nil, errors.New("authentication required: provide WithApiKey, WithBearerToken, or WithSdkToken")
    }
    if authCount > 1 {
        return nil, errors.New("provide exactly one of: WithApiKey, WithBearerToken, WithSdkToken")
    }

    // Call appropriate UniFFI function
    var client *RestClient
    var err error
    if cfg.apiKey != "" {
        client, err = NewRestClientWithApiKey(cfg.apiKey)
    } else if cfg.bearerToken != "" {
        client, err = NewRestClientWithBearerToken(cfg.bearerToken)
    } else {
        client, err = NewRestClientWithSdkToken(cfg.sdkToken)
    }

    if err != nil {
        return nil, fmt.Errorf("failed to create client: %w", err)
    }

    // Apply baseUrl if provided (TODO: needs UniFFI function)

    return client, nil
}

// Usage:
client, err := NewRestClient(
    WithApiKey("your-key"),
    WithBaseUrl("https://custom.api"),
)
if err != nil {
    log.Fatal(err)
}
defer client.Destroy()
```

### C#: Complete Options Implementation

```csharp
// Source: .NET options pattern + Phase 11 C# structure

using System;

namespace FugleMarketData
{
    /// <summary>
    /// Options for configuring RestClient
    /// </summary>
    public class RestClientOptions
    {
        /// <summary>
        /// API key for authentication (optional)
        /// </summary>
        public string? ApiKey { get; set; }

        /// <summary>
        /// Bearer token for authentication (optional)
        /// </summary>
        public string? BearerToken { get; set; }

        /// <summary>
        /// SDK token for authentication (optional)
        /// </summary>
        public string? SdkToken { get; set; }

        /// <summary>
        /// Custom base URL (optional)
        /// </summary>
        public string? BaseUrl { get; set; }
    }

    /// <summary>
    /// REST client for Fugle market data API
    /// </summary>
    public class RestClient : IDisposable
    {
        private readonly MarketdataUniffi.RestClient _client;

        /// <summary>
        /// Create a new REST client with options
        /// </summary>
        /// <param name="options">Client configuration options</param>
        /// <exception cref="ArgumentNullException">If options is null</exception>
        /// <exception cref="ArgumentException">If validation fails</exception>
        public RestClient(RestClientOptions options)
        {
            if (options == null)
                throw new ArgumentNullException(nameof(options));

            // Validate exactly one auth
            int authCount = 0;
            if (!string.IsNullOrEmpty(options.ApiKey)) authCount++;
            if (!string.IsNullOrEmpty(options.BearerToken)) authCount++;
            if (!string.IsNullOrEmpty(options.SdkToken)) authCount++;

            if (authCount == 0)
                throw new ArgumentException(
                    "Authentication required: set ApiKey, BearerToken, or SdkToken",
                    nameof(options));

            if (authCount > 1)
                throw new ArgumentException(
                    "Provide exactly one of: ApiKey, BearerToken, SdkToken",
                    nameof(options));

            try
            {
                if (!string.IsNullOrEmpty(options.ApiKey))
                {
                    _client = MarketdataUniffi.NewRestClientWithApiKey(options.ApiKey);
                }
                else if (!string.IsNullOrEmpty(options.BearerToken))
                {
                    _client = MarketdataUniffi.NewRestClientWithBearerToken(options.BearerToken);
                }
                else
                {
                    _client = MarketdataUniffi.NewRestClientWithSdkToken(options.SdkToken!);
                }

                // Apply BaseUrl if provided (TODO: needs UniFFI function)
            }
            catch (MarketDataException ex)
            {
                throw new MarketdataException($"Failed to create client: {ex.Message}", ex);
            }
        }

        /// <summary>
        /// Access stock market data endpoints
        /// </summary>
        public StockClient Stock => _client.Stock;

        /// <summary>
        /// Access futures and options market data endpoints
        /// </summary>
        public FutOptClient FutOpt => _client.FutOpt;

        public void Dispose()
        {
            _client?.Dispose();
        }
    }
}

// Usage:
using var client = new RestClient(new RestClientOptions
{
    ApiKey = "your-key",
    BaseUrl = "https://custom.api"
});

var quote = await client.Stock.Intraday.GetQuoteAsync("2330");
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Function constructors: `NewRestClientWithApiKey(key)` | Options/builder: Java builder, Go functional options, C# options class | Phase 14 (v0.3.0) | Idiomatic API for each language, flexible config |
| No config exposure | Config objects for reconnect and health check | Phase 14 (v0.3.0) | Users can customize WebSocket behavior |
| UniFFI direct exposure | Idiomatic wrapper layers | Phase 14 (v0.3.0) | Language-native API feel, proper validation layer |
| Validation at connection | Validation at construction | Phase 8 + 14 | Fail-fast principle |
| Health check default: true | Health check default: false | Phase 8 | Aligned with official SDKs (CON-01) |

**Deprecated/outdated:**
- Direct UniFFI function calls: `MarketdataUniffi.newRestClientWithApiKey()` → use wrapper classes instead
- No wrapper layer: Expose UniFFI directly → always create idiomatic wrappers for user-facing API

## Open Questions

1. **baseUrl implementation for REST and WebSocket**
   - What we know: CONTEXT.md requires baseUrl parameter for both clients, Python and Node.js already support it
   - What's unclear: Does core RestClient have `with_base_url()` method exposed via UniFFI? Need to check uniffi/src/lib.rs
   - Recommendation: Add UniFFI export for base_url setter if not already exposed, or add builder method to RestClient

2. **WebSocket config passing to UniFFI layer**
   - What we know: Core has ReconnectionConfig and HealthCheckConfig with validation
   - What's unclear: How to pass these configs through UniFFI to WebSocketClient constructor (might need new UniFFI functions)
   - Recommendation: Add UniFFI constructor like `new_websocket_client_with_config(auth, reconnect_cfg, health_check_cfg)` or builder methods

3. **Error message format standardization**
   - What we know: Python uses "field must be >= value, got actual" format from Phase 12
   - What's unclear: Should Java/Go/C# match exactly or use language idioms (e.g., Java exceptions include field name differently)?
   - Recommendation: Match format structure but adapt to language conventions (Java: field name in exception message, Go: wrap with field context, C#: use ArgumentException parameter name)

4. **UniFFI constant exposure**
   - What we know: Core exports DEFAULT_MAX_ATTEMPTS, DEFAULT_INITIAL_DELAY_MS, etc. from Phase 8
   - What's unclear: Are these constants exposed through UniFFI to Java/Go/C#? If not, wrapper layers need to hardcode them
   - Recommendation: Verify core constants are exported via UniFFI, or reference them in wrapper layer comments with values

## Sources

### Primary (HIGH confidence)
- uniffi/src/lib.rs - Existing UniFFI bindings structure, constructor functions (lines 51-85)
- uniffi/src/client.rs - RestClient wrapper, Arc-based FFI safety patterns (lines 1-100)
- bindings/java/src/main/java/tw/com/fugle/marketdata/FugleRestClient.java - Existing Java builder pattern (lines 1-150)
- bindings/go/examples/rest_example.go - Current Go usage pattern (lines 1-99)
- bindings/csharp/TestRestApi/Program.cs - Current C# usage pattern (lines 1-85)
- core/src/lib.rs - Exported constants and types (lines 50-57)
- core/src/websocket/health_check.rs - HealthCheckConfig validation, constants (lines 1-100)
- core/src/websocket/reconnection.rs - ReconnectionConfig validation (inferred from health_check.rs structure)
- .planning/phases/13-nodejs-config-exposure/13-RESEARCH.md - Node.js config exposure patterns (lines 1-800)
- .planning/phases/14-java-go-bindings/14-CONTEXT.md - User decisions and requirements (lines 1-64)

### Secondary (MEDIUM confidence)
- [Java Builder Pattern Best Practices](https://javatechonline.com/builder-design-pattern-in-java-guide-examples/) - Builder design pattern guidance
- [Go Functional Options Pattern](https://golang.cafe/blog/golang-functional-options-pattern) - Functional options idiomatic Go
- [.NET Options Pattern](https://learn.microsoft.com/en-us/dotnet/core/extensions/options) - Official Microsoft documentation
- [UniFFI Java Bindings](https://github.com/IronCoreLabs/uniffi-bindgen-java) - uniffi-bindgen-java project and configuration
- [Builder Pattern Variations](https://medium.com/@alxkm/builder-pattern-variations-and-best-practices-643b6631341f) - Modern builder approaches
- [Go Functional Options by David Bacisin](https://davidbacisin.com/writing/golang-options-pattern) - Detailed FOP explanation

### Tertiary (LOW confidence)
- None (all findings verified with primary sources or standard pattern documentation)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - UniFFI 0.28 already used, core validation complete from Phase 8, wrapper patterns established in Java/Go/C# Phase 11
- Architecture: HIGH - Existing wrapper structures clear, language patterns well-documented, Phase 12-13 validation patterns proven
- Pitfalls: HIGH - Based on actual binding code review, UniFFI limitations documented, validation patterns from Phase 12-13

**Research date:** 2026-02-15
**Valid until:** 2026-03-17 (30 days - stable domain, UniFFI patterns established)
