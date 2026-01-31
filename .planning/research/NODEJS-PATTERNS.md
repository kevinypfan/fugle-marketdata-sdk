# Node.js SDK API Patterns Research

**Project:** fugle-marketdata-sdk v0.3.0
**Research Date:** 2026-02-01
**Confidence:** HIGH

## Executive Summary

This document compares the official `@fugle/marketdata` Node.js SDK API patterns with our current UniFFI-generated JavaScript bindings. The goal is to achieve API compatibility for v0.3.0.

**Key Finding:** Our SDK uses simplified single-argument constructors (`new RestClient(apiKey)`) while the official SDK uses options objects (`new RestClient({ apiKey })`). We need to match their API exactly.

---

## Official Node.js SDK Constructor Patterns

### RestClient

**Constructor Signature:**
```typescript
new RestClient(options: ClientOptions)

interface ClientOptions {
  apiKey?: string;
  sdkToken?: string;
  bearerToken?: string;
  baseUrl?: string;
  healthCheck?: HealthCheckConfig;  // Only for WebSocket
}
```

**Validation Rules:**
- Exactly ONE of `apiKey`, `bearerToken`, or `sdkToken` must be specified
- Throws `TypeError` if none provided
- Throws `TypeError` if more than one provided

**Usage Examples:**
```javascript
// With API key
const client = new RestClient({ apiKey: 'YOUR_API_KEY' });

// With bearer token
const client = new RestClient({ bearerToken: 'YOUR_BEARER_TOKEN' });

// With SDK token
const client = new RestClient({ sdkToken: 'YOUR_SDK_TOKEN' });

// With custom baseUrl
const client = new RestClient({
  apiKey: 'YOUR_API_KEY',
  baseUrl: 'https://custom-api.example.com/v2.0'
});
```

**Access Pattern:**
```javascript
const stock = client.stock;     // Returns RestStockClient (cached)
const futopt = client.futopt;   // Returns RestFutOptClient (cached)

// Endpoint access
await stock.intraday.quote({ symbol: '2330' });
await stock.intraday.ticker({ symbol: '2330' });
await stock.intraday.candles({ symbol: '2330' });
```

---

### WebSocketClient

**Constructor Signature:**
```typescript
new WebSocketClient(options: ClientOptions)

interface ClientOptions {
  apiKey?: string;
  sdkToken?: string;
  bearerToken?: string;
  baseUrl?: string;
  healthCheck?: HealthCheckConfig;
}

interface HealthCheckConfig {
  enabled: boolean;
  pingInterval?: number;      // Default: 30000 (30 seconds)
  maxMissedPongs?: number;    // Default: 2
}
```

**Validation Rules:**
- Same as RestClient: exactly ONE authentication token required
- Throws `TypeError` if none or multiple tokens provided

**Usage Examples:**
```javascript
// Basic WebSocket client
const client = new WebSocketClient({ apiKey: 'YOUR_API_KEY' });

// With health check enabled
const client = new WebSocketClient({
  apiKey: 'YOUR_API_KEY',
  healthCheck: {
    enabled: true,
    pingInterval: 30000,
    maxMissedPongs: 2
  }
});

// With custom baseUrl
const client = new WebSocketClient({
  apiKey: 'YOUR_API_KEY',
  baseUrl: 'wss://custom-ws.example.com/v2.0'
});
```

**Access Pattern:**
```javascript
const stock = client.stock;     // Returns WebSocketStockClient (cached)
const futopt = client.futopt;   // Returns WebSocketFutOptClient (cached)

// Connection and subscription
await stock.connect();
stock.subscribe({ channel: 'trades', symbol: '2330' });
stock.on('message', (data) => console.log(JSON.parse(data)));
```

---

## Configuration Options

### Authentication Options (REST & WebSocket)

| Option | Type | Required | Description |
|--------|------|----------|-------------|
| `apiKey` | `string` | One of three | API key authentication (header: `X-API-KEY`) |
| `bearerToken` | `string` | One of three | Bearer token authentication (header: `Authorization: Bearer {token}`) |
| `sdkToken` | `string` | One of three | SDK token authentication (header: `X-SDK-TOKEN`) |

**Mutual Exclusivity:** Only ONE authentication method can be specified. Constructor throws `TypeError` if:
- None are provided
- More than one is provided

### Base URL Override (REST & WebSocket)

| Option | Type | Default (REST) | Default (WebSocket) | Description |
|--------|------|----------------|---------------------|-------------|
| `baseUrl` | `string` | `https://api.fugle.tw/marketdata/v1.0` | `wss://api.fugle.tw/marketdata/v1.0` | Custom API base URL |

**URL Construction:**
- REST: `{baseUrl}/{stock|futopt}/{endpoint}`
- WebSocket: `{baseUrl}/{stock|futopt}/streaming`
- Trailing slashes are automatically normalized

### Health Check Options (WebSocket Only)

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `healthCheck.enabled` | `boolean` | `false` | Enable automatic ping/pong health checks |
| `healthCheck.pingInterval` | `number` | `30000` | Interval between ping messages (milliseconds) |
| `healthCheck.maxMissedPongs` | `number` | `2` | Max missed pongs before auto-disconnect |

**Health Check Behavior:**
- When enabled, automatically sends ping messages at specified interval
- Increments missed pong counter on each ping
- Resets counter to 0 when pong received
- Auto-disconnects when `missedPongs > maxMissedPongs`
- Timer is cleared on disconnect

---

## Current SDK Differences

### Our Current API (v0.2.0)

**RestClient:**
```javascript
// Single string argument
const client = new RestClient('YOUR_API_KEY');

// No support for:
// - bearerToken or sdkToken
// - baseUrl override
// - Options object pattern
```

**WebSocketClient:**
```javascript
// Single string argument
const client = new WebSocketClient('YOUR_API_KEY');

// No support for:
// - bearerToken or sdkToken
// - baseUrl override
// - healthCheck configuration
// - Options object pattern
```

### Key Differences

| Feature | Official Node.js SDK | Our SDK v0.2.0 | Gap |
|---------|---------------------|----------------|-----|
| Constructor | Options object | Single string | **INCOMPATIBLE** |
| Bearer token | ✅ Supported | ❌ Not supported | **MISSING** |
| SDK token | ✅ Supported | ❌ Not supported | **MISSING** |
| Base URL override | ✅ Supported | ❌ Not supported | **MISSING** |
| Health check config | ✅ Supported (WebSocket) | ❌ Not supported | **MISSING** |
| Token validation | ✅ Exactly one required | ❌ No validation | **MISSING** |

---

## Required Changes for v0.3.0

### 1. Constructor Signature Change

**From:**
```rust
// Current UniFFI
#[uniffi::export]
impl RestClient {
    #[uniffi::constructor]
    pub fn new(api_key: String) -> Self { ... }
}
```

**To:**
```rust
// Target for v0.3.0
#[uniffi::export]
impl RestClient {
    #[uniffi::constructor]
    pub fn new(options: ClientOptions) -> Result<Self, ClientError> { ... }
}

#[derive(uniffi::Record)]
pub struct ClientOptions {
    pub api_key: Option<String>,
    pub bearer_token: Option<String>,
    pub sdk_token: Option<String>,
    pub base_url: Option<String>,
}
```

### 2. WebSocket Configuration

**From:**
```rust
// Current UniFFI
#[uniffi::export]
impl WebSocketClient {
    #[uniffi::constructor]
    pub fn new(api_key: String) -> Self { ... }
}
```

**To:**
```rust
// Target for v0.3.0
#[uniffi::export]
impl WebSocketClient {
    #[uniffi::constructor]
    pub fn new(options: WebSocketOptions) -> Result<Self, ClientError> { ... }
}

#[derive(uniffi::Record)]
pub struct WebSocketOptions {
    pub api_key: Option<String>,
    pub bearer_token: Option<String>,
    pub sdk_token: Option<String>,
    pub base_url: Option<String>,
    pub health_check: Option<HealthCheckConfig>,
}

#[derive(uniffi::Record)]
pub struct HealthCheckConfig {
    pub enabled: bool,
    pub ping_interval: Option<u64>,      // milliseconds, default 30000
    pub max_missed_pongs: Option<u32>,   // default 2
}
```

### 3. Validation Logic

Must implement constructor validation:
```rust
fn validate_auth_options(options: &ClientOptions) -> Result<(), ClientError> {
    let token_count = [
        options.api_key.is_some(),
        options.bearer_token.is_some(),
        options.sdk_token.is_some(),
    ].iter().filter(|&&x| x).count();

    if token_count == 0 {
        return Err(ClientError::InvalidOptions(
            "One of 'apiKey', 'bearerToken', or 'sdkToken' must be specified".into()
        ));
    }

    if token_count > 1 {
        return Err(ClientError::InvalidOptions(
            "Only one of 'apiKey', 'bearerToken', or 'sdkToken' can be specified".into()
        ));
    }

    Ok(())
}
```

### 4. Base URL Handling

Must support custom base URL with trailing slash normalization:
```rust
fn build_base_url(custom_base: Option<String>, default_base: &str, client_type: &str) -> String {
    let base = custom_base.unwrap_or_else(|| default_base.to_string());
    let normalized = base.trim_end_matches('/');
    format!("{}/{}", normalized, client_type)
}
```

### 5. Health Check Implementation

WebSocket client must implement:
- Automatic ping timer when `health_check.enabled = true`
- Missed pong counter tracking
- Auto-disconnect when threshold exceeded
- Timer cleanup on disconnect

---

## Implementation Priority

### Phase 1: Core API Compatibility (CRITICAL)
1. ✅ Options object constructors for REST and WebSocket
2. ✅ Multiple authentication methods (apiKey, bearerToken, sdkToken)
3. ✅ Token validation (exactly one required)
4. ✅ Error handling with proper `TypeError` messages

### Phase 2: Configuration Support (HIGH)
5. ✅ Base URL override for REST
6. ✅ Base URL override for WebSocket
7. ✅ URL normalization (trailing slash handling)

### Phase 3: Advanced Features (MEDIUM)
8. ✅ Health check configuration for WebSocket
9. ✅ Automatic ping/pong implementation
10. ✅ Auto-disconnect on missed pongs

### Phase 4: Testing & Validation (HIGH)
11. ✅ Constructor validation tests
12. ✅ Options object tests
13. ✅ Health check behavior tests
14. ✅ Integration tests matching official SDK

---

## Testing Requirements

### Constructor Tests
```javascript
// Should succeed
new RestClient({ apiKey: 'key' });
new RestClient({ bearerToken: 'token' });
new RestClient({ sdkToken: 'token' });

// Should throw TypeError
new RestClient({});  // No auth
new RestClient({ apiKey: 'key', bearerToken: 'token' });  // Multiple auth
```

### Base URL Tests
```javascript
const client = new RestClient({
  apiKey: 'key',
  baseUrl: 'https://custom.com/v2/'  // With trailing slash
});
// Should normalize to: https://custom.com/v2/stock
```

### Health Check Tests
```javascript
const client = new WebSocketClient({
  apiKey: 'key',
  healthCheck: { enabled: true, pingInterval: 100, maxMissedPongs: 2 }
});
// Should send ping every 100ms
// Should disconnect after 3 missed pongs
```

---

## Sources

**PRIMARY (HIGH Confidence):**
- Official SDK source code: `/Users/zackfan/Project/fugle/fugle-marketdata-node/src/`
  - `client-factory.ts`: Options interface and validation logic
  - `rest/client.ts`: REST client implementation
  - `websocket/client.ts`: WebSocket client with health check
  - `constants.ts`: Default base URLs
- Official test suite: `/Users/zackfan/Project/fugle/fugle-marketdata-node/test/`
  - `rest-client.spec.ts`: REST constructor tests (lines 10-48)
  - `websocket-client.spec.ts`: WebSocket constructor tests (lines 15-54), health check tests (lines 497-750)
- Official README: `/Users/zackfan/Project/fugle/fugle-marketdata-node/README.md`

**CURRENT SDK:**
- Our bindings: `/Users/zackfan/Project/fugle/fugle-marketdata-sdk/js/`
  - `index.d.ts`: Current TypeScript definitions
  - `test_rest.js`: Current REST client usage
  - `test_websocket.js`: Current WebSocket client usage

---

## Recommendations for v0.3.0

1. **CRITICAL:** Change both REST and WebSocket constructors to accept options objects
2. **CRITICAL:** Implement token validation (exactly one required)
3. **HIGH:** Support all three authentication methods (apiKey, bearerToken, sdkToken)
4. **HIGH:** Support base URL override with normalization
5. **MEDIUM:** Implement WebSocket health check with configurable ping/pong
6. **HIGH:** Add comprehensive tests for all constructor patterns
7. **LOW:** Consider backward compatibility wrapper for simple `new RestClient(apiKey)` usage

### Breaking Change Notice

This is a **BREAKING CHANGE** from v0.2.0:
```javascript
// v0.2.0 (old)
new RestClient('api-key');

// v0.3.0 (new - matches official SDK)
new RestClient({ apiKey: 'api-key' });
```

**Migration path:** Simple wrapper in documentation or deprecation warning for one version cycle.
