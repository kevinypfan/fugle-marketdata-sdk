# Python SDK API Patterns Research

**Researched:** 2026-02-01
**Reference SDK:** fugle-marketdata-python v2.4.1
**Our SDK:** py/src/ (PyO3 bindings)
**Confidence:** HIGH

## Executive Summary

The official fugle-marketdata-python SDK uses a **kwargs-based constructor pattern** with multiple authentication options and configuration classes. Our Rust-based SDK currently uses **positional arguments** and static methods. This research documents the exact patterns needed for v0.3.0 API compatibility.

## 1. Constructor Patterns

### 1.1 RestClient Constructor

**Official Python SDK:**
```python
# Primary constructor - accepts **options (kwargs)
client = RestClient(api_key='YOUR_API_KEY')
client = RestClient(bearer_token='YOUR_TOKEN')
client = RestClient(sdk_token='YOUR_SDK_TOKEN')
client = RestClient(api_key='KEY', base_url='https://custom.url')
```

**Implementation Details:**
- Source: `fugle_marketdata/client_factory.py`
- Signature: `__init__(self, **options)`
- Validation: Requires exactly ONE of `api_key`, `bearer_token`, or `sdk_token`
- Optional parameters: `base_url` (defaults to `FUGLE_MARKETDATA_API_REST_BASE_URL/v1.0`)
- Error handling: Raises `TypeError` if zero or multiple auth tokens provided

**Our Current Implementation:**
```python
# Positional argument
client = RestClient("YOUR_API_KEY")

# Static methods for alternatives
client = RestClient.with_bearer_token("YOUR_TOKEN")
client = RestClient.with_sdk_token("YOUR_SDK_TOKEN")
```

**Gap:**
- ❌ No kwargs support
- ❌ No `base_url` configuration option
- ❌ No unified constructor for all auth types

### 1.2 WebSocketClient Constructor

**Official Python SDK:**
```python
# Primary constructor - accepts **options (kwargs)
ws = WebSocketClient(api_key='YOUR_API_KEY')
ws = WebSocketClient(bearer_token='YOUR_TOKEN')
ws = WebSocketClient(sdk_token='YOUR_SDK_TOKEN')
ws = WebSocketClient(
    api_key='KEY',
    base_url='wss://custom.url',
    health_check=HealthCheckConfig(enabled=True, ping_interval=30000)
)
```

**Implementation Details:**
- Source: `fugle_marketdata/websocket/client.py`
- Signature: `__init__(self, **config)`
- Accepts: `api_key`, `bearer_token`, `sdk_token`, `base_url`, `health_check`
- Optional parameters:
  - `base_url`: WebSocket URL (defaults to `wss://api.fugle.tw/marketdata/v1.0`)
  - `health_check`: `HealthCheckConfig` instance

**Our Current Implementation:**
```python
# Positional argument only
ws = WebSocketClient("YOUR_API_KEY")
```

**Gap:**
- ❌ No kwargs support
- ❌ No bearer_token/sdk_token support
- ❌ No `base_url` configuration
- ❌ No health_check configuration (though we have `ReconnectConfig`)

## 2. Configuration Classes

### 2.1 HealthCheckConfig

**Official Python SDK:**
```python
class HealthCheckConfig:
    def __init__(self, enabled: bool = False, ping_interval: int = 30000, max_missed_pongs: int = 2):
        self.enabled = enabled
        self.ping_interval = ping_interval  # milliseconds
        self.max_missed_pongs = max_missed_pongs
```

**Source:** `fugle_marketdata/websocket/client.py:22-26`

**Features:**
- Default: disabled (`enabled=False`)
- Ping interval: 30000ms (30 seconds)
- Max missed pongs: 2
- Used for automatic ping/pong health monitoring
- Automatically disconnects if too many pongs missed

**Our Current Implementation:**
```python
# We have ReconnectConfig instead
config = ReconnectConfig(
    enabled=True,
    max_retries=5,
    base_delay_ms=1000,
    max_delay_ms=30000
)
```

**Gap:**
- ❌ Different purpose: `ReconnectConfig` handles reconnection, not health checks
- ❌ No health check monitoring equivalent
- ❌ Different default behavior

### 2.2 Authentication Validation

**Official Python SDK:**
```python
# In ClientFactory.__init__
api_key = options.get('api_key')
bearer_token = options.get('bearer_token')
sdk_token = options.get('sdk_token')
token_count = sum(bool(token) for token in [api_key, bearer_token, sdk_token])

if token_count == 0:
    raise TypeError('One of the "apiKey", "bearerToken", or "sdkToken" options must be specified')

if token_count > 1:
    raise TypeError('Only one of the "apiKey", "bearerToken", or "sdkToken" options must be specified')
```

**Source:** `fugle_marketdata/client_factory.py:4-13`

**Validation Rules:**
1. Exactly ONE authentication method required
2. Multiple authentication methods raise `TypeError`
3. No authentication method raises `TypeError`

**Our Current Implementation:**
- ✅ Validates at Rust level (single Auth enum variant)
- ❌ No Python-level validation for kwargs

## 3. API Access Patterns

### 3.1 REST Client Property Access

**Official Python SDK:**
```python
client = RestClient(api_key='KEY')
stock = client.stock      # Property, returns RestStockClient
futopt = client.futopt    # Property, returns RestFutOptClient
```

**Our Implementation:**
```python
client = RestClient("KEY")
stock = client.stock      # @getter property, returns StockClient
futopt = client.futopt    # @getter property, returns FutOptClient
```

**Status:** ✅ Compatible (both use properties)

### 3.2 WebSocket Event Subscription

**Official Python SDK:**
```python
ws = WebSocketClient(api_key='KEY')
stock = ws.stock

# Event registration
stock.on("connect", handle_connect)
stock.on("message", handle_message)
stock.on("disconnect", handle_disconnect)
stock.on("error", handle_error)

# Connection
stock.connect()

# Subscription - DICT PARAMETER
stock.subscribe({
    "channel": "trades",
    "symbol": "2330"
})

# Ping - DICT PARAMETER
stock.ping({"state": 123})

# Unsubscribe - DICT PARAMETER
stock.unsubscribe({
    "channel": "trades",
    "symbol": "2330"
})
```

**Source:** `fugle_marketdata/websocket/client.py`

**Our Current Implementation:**
```python
ws = WebSocketClient("KEY")
stock = ws.stock

# Event registration (compatible)
stock.on("message", handle_message)

# Connection (compatible)
stock.connect()

# Subscription - SEPARATE PARAMETERS
stock.subscribe("trades", "2330", odd_lot=False)

# Unsubscribe - SUBSCRIPTION ID
stock.unsubscribe("subscription_id")
```

**Gap:**
- ❌ **Different subscription API**: Dict vs separate parameters
- ❌ **Different unsubscribe API**: Dict vs subscription ID
- ❌ **No ping() method** in our implementation
- ❌ **No subscriptions() method** to list active subscriptions

### 3.3 WebSocket Method Signatures

**Official Python SDK WebSocket Methods:**

```python
# Subscribe with dict parameter
def subscribe(self, params):
    message = {
        "event": "subscribe",
        "data": params  # params is the dict with channel, symbol
    }
    self.__send(message)

# Unsubscribe with dict parameter
def unsubscribe(self, params):
    message = {
        "event": "unsubscribe",
        "data": params
    }
    self.__send(message)

# Ping with dict parameter
def ping(self, message):
    message = {
        "event": "ping",
        "data": {
            "state": message  # message can be dict or value
        }
    }
    self.__send(message)

# Get subscriptions list
def subscriptions(self):
    message = {
        "event": "subscriptions"
    }
    self.__send(message)
```

**Source:** `fugle_marketdata/websocket/client.py:66-84`

**Our Current Implementation:**

```python
# Subscribe with separate parameters
def subscribe(channel: str, symbol: str, odd_lot: bool = False) -> PyResult<()>

# Unsubscribe by ID string
def unsubscribe(subscription_id: str) -> PyResult<()>

# No ping() method

# Subscriptions returns list of strings
def subscriptions() -> Vec<String>
```

**Critical Differences:**

| Method | Official SDK | Our SDK | Compatible? |
|--------|-------------|---------|-------------|
| `subscribe()` | `subscribe(dict)` | `subscribe(channel, symbol, odd_lot)` | ❌ Different signature |
| `unsubscribe()` | `unsubscribe(dict)` | `unsubscribe(subscription_id)` | ❌ Different signature |
| `ping()` | `ping(dict/value)` | Not implemented | ❌ Missing |
| `subscriptions()` | Sends request message | Returns `Vec<String>` | ⚠️ Different behavior |

## 4. REST Endpoint Method Patterns

### 4.1 Parameter Passing

**Official Python SDK:**
```python
# Parameters passed as kwargs, symbol extracted with pop()
def quote(self, **params):
    symbol = params.pop('symbol')  # Extract and remove from params
    return self.request(f"intraday/quote/{symbol}", **params)

# Usage
quote = stock.intraday.quote(symbol="2330")
quote = stock.intraday.quote(symbol="2330", odd_lot=True)
```

**Source:** `fugle_marketdata/rest/stock/intraday.py:11-13`

**Our Implementation:**
```python
# Separate positional and optional parameters
def quote(symbol: str, odd_lot: bool = False) -> Awaitable[dict]

# Usage (same as official)
quote = await client.stock.intraday.quote("2330")
quote = await client.stock.intraday.quote("2330", odd_lot=True)
```

**Status:** ⚠️ **Functionally compatible** but different implementation

**Trade-off:**
- Official: More flexible (any kwarg can be passed to API)
- Ours: Type-safe (only valid parameters accepted)

### 4.2 Historical Candles Pattern

**Official Python SDK:**
```python
def candles(self, **params):
    symbol = params.pop('symbol')
    return self.request(f"historical/candles/{symbol}", **params)

# Usage
candles = stock.historical.candles(
    symbol="2330",
    from_date="2024-01-01",
    to_date="2024-01-31",
    timeframe="D",
    fields="open,high,low,close",
    sort="asc",
    adjusted=True
)
```

**Our Implementation:**
```python
def candles(
    symbol: str,
    from_date: Option<str> = None,
    to_date: Option<str> = None,
    timeframe: Option<str> = None,
    fields: Option<str> = None,
    sort: Option<str> = None,
    adjusted: Option<bool> = None
) -> Awaitable[dict]

# Usage (same)
candles = await client.stock.historical.candles(
    "2330",
    from_date="2024-01-01",
    to_date="2024-01-31",
    timeframe="D"
)
```

**Status:** ✅ **Compatible** (usage pattern matches)

## 5. Error Handling

### 5.1 Exception Class

**Official Python SDK:**
```python
class FugleAPIError(Exception):
    def __init__(self, message, url=None, status_code=None, params=None, response_text=None):
        self.message = message
        self.url = url
        self.status_code = status_code
        self.params = params
        self.response_text = response_text  # Truncated to 200 chars
```

**Source:** `fugle_marketdata/exceptions.py:1-23`

**Attributes:**
- `message`: Error description
- `url`: API endpoint that was called
- `status_code`: HTTP status code (if available)
- `params`: Request parameters that were sent
- `response_text`: Raw response text (truncated to 200 chars)

**Usage:**
```python
try:
    quote = client.stock.intraday.quote(symbol="2330")
except FugleAPIError as e:
    print(f"Error: {e.message}")
    print(f"URL: {e.url}")
    print(f"Status Code: {e.status_code}")
    print(f"Params: {e.params}")
    print(f"Response: {e.response_text}")
```

**Our Current Implementation:**
```python
# Exception hierarchy
class MarketDataError(Exception):          # Base
class ApiError(MarketDataError):           # API errors
class RateLimitError(ApiError):            # Rate limiting
class AuthError(MarketDataError):          # Auth failures
class ConnectionError(MarketDataError):    # Connection issues
class TimeoutError(MarketDataError):       # Timeouts
class WebSocketError(MarketDataError):     # WebSocket errors
```

**Gap:**
- ❌ Different exception class name: `FugleAPIError` vs `ApiError`
- ❌ Different exception hierarchy
- ❌ Missing detailed error attributes (url, params, response_text)

## 6. Constants and Configuration

### 6.1 Base URLs and Versioning

**Official Python SDK:**
```python
FUGLE_MARKETDATA_API_REST_BASE_URL = 'https://api.fugle.tw/marketdata'
FUGLE_MARKETDATA_API_WEBSOCKET_BASE_URL = 'wss://api.fugle.tw/marketdata'
FUGLE_MARKETDATA_API_VERSION = 'v1.0'
```

**Source:** `fugle_marketdata/constants.py:1-3`

**URL Construction:**
```python
# REST client
base_url = f"{FUGLE_MARKETDATA_API_REST_BASE_URL}/{FUGLE_MARKETDATA_API_VERSION}"
# Result: https://api.fugle.tw/marketdata/v1.0

# WebSocket client
base_url = f"{FUGLE_MARKETDATA_API_WEBSOCKET_BASE_URL}/{FUGLE_MARKETDATA_API_VERSION}"
# Result: wss://api.fugle.tw/marketdata/v1.0
```

**Our Implementation:**
- ✅ Uses same base URLs in marketdata-core
- ✅ Hard-coded in Rust, not exposed as constants

### 6.2 Event Constants

**Official Python SDK:**
```python
CONNECT_EVENT = 'connect'
DISCONNECT_EVENT = 'disconnect'
MESSAGE_EVENT = 'message'
ERROR_EVENT = 'error'
AUTHENTICATED_EVENT = 'authenticated'
UNAUTHENTICATED_EVENT = 'unauthenticated'
```

**Source:** `fugle_marketdata/constants.py:5-10`

**Our Implementation:**
- Uses string literals in callback registry
- Accepts: "connect", "connected", "disconnect", "disconnected", "close", "message", "data", "error", "reconnect", "reconnecting"

**Status:** ✅ Compatible (accepts same event names)

## 7. Key Differences Summary

### 7.1 Constructor API

| Aspect | Official SDK | Our SDK | Priority |
|--------|-------------|---------|----------|
| Constructor style | `RestClient(**options)` | `RestClient(api_key)` | **HIGH** |
| Auth methods | kwargs: `api_key`, `bearer_token`, `sdk_token` | Positional + static methods | **HIGH** |
| `base_url` config | Supported via kwargs | Not supported | **MEDIUM** |
| Validation | Python TypeError | Rust type system | **LOW** |

### 7.2 WebSocket API

| Aspect | Official SDK | Our SDK | Priority |
|--------|-------------|---------|----------|
| Constructor | `WebSocketClient(**config)` | `WebSocketClient(api_key)` | **HIGH** |
| `health_check` config | `HealthCheckConfig` class | `ReconnectConfig` (different purpose) | **HIGH** |
| `subscribe()` signature | `subscribe(dict)` | `subscribe(channel, symbol, odd_lot)` | **CRITICAL** |
| `unsubscribe()` signature | `unsubscribe(dict)` | `unsubscribe(subscription_id)` | **CRITICAL** |
| `ping()` method | `ping(dict/value)` | Not implemented | **MEDIUM** |
| `subscriptions()` method | Sends request message | Returns list | **MEDIUM** |

### 7.3 Error Handling

| Aspect | Official SDK | Our SDK | Priority |
|--------|-------------|---------|----------|
| Exception name | `FugleAPIError` | `ApiError` | **MEDIUM** |
| Exception hierarchy | Single class | Multi-level hierarchy | **LOW** |
| Error attributes | url, status_code, params, response_text | Standard Exception | **MEDIUM** |

## 8. Recommended Changes for v0.3.0

### Priority 1: CRITICAL (Breaking API Changes)

1. **WebSocket subscribe() signature change**
   ```python
   # Current
   ws.stock.subscribe("trades", "2330")

   # Target (compatible)
   ws.stock.subscribe({"channel": "trades", "symbol": "2330"})
   ```

2. **WebSocket unsubscribe() signature change**
   ```python
   # Current
   ws.stock.unsubscribe("subscription_id")

   # Target (compatible)
   ws.stock.unsubscribe({"channel": "trades", "symbol": "2330"})
   ```

### Priority 2: HIGH (Constructor Compatibility)

3. **RestClient kwargs constructor**
   ```python
   # Target
   client = RestClient(api_key='KEY')
   client = RestClient(bearer_token='TOKEN')
   client = RestClient(sdk_token='SDK_TOKEN')
   client = RestClient(api_key='KEY', base_url='https://custom.url')
   ```

4. **WebSocketClient kwargs constructor**
   ```python
   # Target
   ws = WebSocketClient(api_key='KEY')
   ws = WebSocketClient(api_key='KEY', health_check=HealthCheckConfig())
   ```

5. **HealthCheckConfig implementation**
   ```python
   # Target
   config = HealthCheckConfig(
       enabled=True,
       ping_interval=30000,
       max_missed_pongs=2
   )
   ```

### Priority 3: MEDIUM (Feature Additions)

6. **WebSocket ping() method**
   ```python
   ws.stock.ping({"state": 123})
   ```

7. **base_url configuration support**
   - RestClient: Custom REST endpoint
   - WebSocketClient: Custom WebSocket endpoint

8. **FugleAPIError exception class**
   ```python
   # Add alias or replacement
   FugleAPIError = ApiError  # Simple alias
   # OR implement full compatibility class
   ```

## 9. Migration Path

### Phase 1: Add Compatibility Without Breaking (v0.3.0-alpha)
- Add kwargs constructors as alternatives
- Keep existing positional constructors
- Add `subscribe(dict)` alongside `subscribe(channel, symbol)`
- Add deprecation warnings for old API

### Phase 2: Align Defaults (v0.3.0-beta)
- Make kwargs the primary documented API
- Add HealthCheckConfig
- Implement ping() method
- Add base_url support

### Phase 3: Full Compatibility (v0.3.0 release)
- Remove deprecated APIs or make them aliases
- Ensure 100% API compatibility with fugle-marketdata-python
- Update all documentation

## 10. Quality Gate Checklist

- [x] Constructor patterns documented (RestClient, WebSocketClient)
- [x] All configuration options listed (HealthCheckConfig, base_url)
- [x] Differences from our SDK noted (WebSocket subscribe/unsubscribe signatures)
- [x] Priority recommendations provided
- [x] Migration path outlined

## Sources

- **HIGH Confidence**: Direct source code examination
  - `/fugle-marketdata-python/fugle_marketdata/__init__.py`
  - `/fugle-marketdata-python/fugle_marketdata/client_factory.py`
  - `/fugle-marketdata-python/fugle_marketdata/rest/factory.py`
  - `/fugle-marketdata-python/fugle_marketdata/websocket/factory.py`
  - `/fugle-marketdata-python/fugle_marketdata/websocket/client.py`
  - `/fugle-marketdata-python/fugle_marketdata/rest/base_rest.py`
  - `/fugle-marketdata-python/fugle_marketdata/exceptions.py`
  - `/fugle-marketdata-python/fugle_marketdata/constants.py`
  - `/fugle-marketdata-python/README.md`
  - `/fugle-marketdata-python/test.py` (usage examples)
