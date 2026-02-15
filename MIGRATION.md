# Migrating from v0.2.x to v0.3.0

This guide helps you upgrade from v0.2.x to v0.3.0. The major change is that constructors now use options objects instead of positional string arguments, and WebSocket configuration options are now exposed.

For a complete list of changes, see [CHANGELOG.md](CHANGELOG.md).

**Backward Compatibility Note:**
- **Python and Node.js**: String constructors still work but emit deprecation warnings. They will be removed in v0.4.0.
- **Java, Go, C#**: Constructors changed immediately (no deprecated path).

## Breaking Changes Summary

| Change | Languages | Impact |
|--------|-----------|--------|
| Constructor API | All | Options object/kwargs instead of string |
| Health check default | All | `false` (was `true`) |
| Auth validation | All | Exactly one auth method required |

## Language-Specific Migration Guides

### Python

**Before (v0.2.x):**
```python
from marketdata_py import RestClient, WebSocketClient

client = RestClient("your-api-key")
client2 = RestClient.with_bearer_token("your-token")
ws = WebSocketClient("your-api-key")
```

**After (v0.3.0):**
```python
from marketdata_py import RestClient, WebSocketClient, ReconnectConfig, HealthCheckConfig

client = RestClient(api_key="your-api-key")
client2 = RestClient(bearer_token="your-token")
ws = WebSocketClient(
    api_key="your-api-key",
    reconnect=ReconnectConfig(max_attempts=10),
    health_check=HealthCheckConfig(enabled=True),
)
```

**Migration Steps:**
1. Replace `RestClient("key")` with `RestClient(api_key="key")`
2. Replace `RestClient.with_bearer_token("token")` with `RestClient(bearer_token="token")`
3. Replace `RestClient.with_sdk_token("token")` with `RestClient(sdk_token="token")`
4. Replace `WebSocketClient("key")` with `WebSocketClient(api_key="key")`
5. (Optional) Add `reconnect` and `health_check` config if needed

**Automated Migration:**
```bash
# Transform positional arguments to keyword arguments
python migration/migrate-python.py --path src/

# Preview changes without modifying files
python migration/migrate-python.py --path src/ --dry-run
```

---

### Node.js / TypeScript

**Before (v0.2.x):**
```javascript
const { RestClient, WebSocketClient } = require('@fubon/marketdata-js');

const client = new RestClient('your-api-key');
const ws = new WebSocketClient('your-api-key');
```

**After (v0.3.0):**
```typescript
import { RestClient, WebSocketClient } from '@fubon/marketdata-js';

const client = new RestClient({ apiKey: 'your-api-key' });
const ws = new WebSocketClient({
  apiKey: 'your-api-key',
  reconnect: { maxAttempts: 10, initialDelayMs: 2000 },
  healthCheck: { enabled: true, intervalMs: 15000 },
});
```

**Migration Steps:**
1. Replace `new RestClient('key')` with `new RestClient({ apiKey: 'key' })`
2. Replace `new WebSocketClient('key')` with `new WebSocketClient({ apiKey: 'key' })`
3. (Optional) Add `reconnect` and `healthCheck` config objects

**Automated Migration:**
```bash
# Transform string constructors to object constructors
npx jscodeshift -t migration/migrate-javascript.js src/

# Preview changes without modifying files
npx jscodeshift -t migration/migrate-javascript.js src/ --dry
```

---

### Java

**Before (v0.2.x):**
```java
import tw.com.fugle.marketdata.*;

RestClient client = new RestClient("your-api-key");
```

**After (v0.3.0):**
```java
import tw.com.fugle.marketdata.*;

FugleRestClient client = FugleRestClient.builder()
    .apiKey("your-api-key")
    .reconnectOptions(new ReconnectOptions.Builder()
        .maxAttempts(10)
        .initialDelayMs(2000L)
        .build())
    .build();
```

**Migration Steps:**
1. Replace `new RestClient(...)` with `FugleRestClient.builder()...build()`
2. Use `.apiKey()`, `.bearerToken()`, or `.sdkToken()` methods
3. (Optional) Add `.reconnectOptions()` and `.healthCheckOptions()`

---

### Go

**Before (v0.2.x):**
```go
import "github.com/fugle/marketdata-sdk-go/marketdata"

client, err := marketdata.NewRestClientWithApiKey("your-api-key")
```

**After (v0.3.0):**
```go
import "github.com/fugle/marketdata-sdk-go/marketdata"

client, err := marketdata.NewFugleRestClient(
    marketdata.WithApiKey("your-api-key"),
    marketdata.WithReconnectOptions(marketdata.ReconnectOptions{
        MaxAttempts:    10,
        InitialDelayMs: 2000,
    }),
)
```

**Migration Steps:**
1. Replace `NewRestClientWithApiKey(key)` with `NewFugleRestClient(WithApiKey(key))`
2. Replace `NewRestClientWithBearerToken(token)` with `NewFugleRestClient(WithBearerToken(token))`
3. (Optional) Add `WithReconnectOptions()` and `WithHealthCheckOptions()` functional options

---

### C#

**Before (v0.2.x):**
```csharp
using MarketdataUniffi;

var client = new RestClient("your-api-key");
```

**After (v0.3.0):**
```csharp
using MarketdataUniffi;

var client = new RestClient(new RestClientOptions {
    ApiKey = "your-api-key"
});

// String constructor still supported for now (deprecated)
var client2 = new RestClient("your-api-key");
```

**Migration Steps:**
1. Replace `new RestClient("key")` with `new RestClient(new RestClientOptions { ApiKey = "key" })`
2. (Optional) Add `ReconnectOptions` and `HealthCheckOptions` properties

---

## Common Issues

### "ValueError: Provide exactly one of: apiKey, bearerToken, sdkToken"
**Cause:** You provided zero or multiple authentication methods.

**Solution:** Pass exactly one of `api_key`, `bearer_token`, or `sdk_token`:
```python
# ✓ Correct
client = RestClient(api_key="key")

# ✗ Wrong - no auth provided
client = RestClient()

# ✗ Wrong - multiple auth provided
client = RestClient(api_key="key", bearer_token="token")
```

---

### "ConfigError: max_attempts must be >= 1"
**Cause:** Invalid configuration value provided.

**Solution:** Check configuration constraints in [docs/configuration.md](docs/configuration.md). For `ReconnectConfig`:
- `max_attempts`: Must be >= 1
- `initial_delay_ms`: Must be >= 100ms
- `max_delay_ms`: Must be >= `initial_delay_ms`

---

### Health Check Not Running
**Cause:** Default changed from `enabled: true` to `enabled: false` in v0.3.0.

**Solution:** Explicitly enable health checks if needed:
```python
ws = WebSocketClient(
    api_key="key",
    health_check=HealthCheckConfig(enabled=True)
)
```

---

## Automated Migration Tools

### Python Migration Script
```bash
# Transform all Python files in directory
python migration/migrate-python.py --path src/

# Dry run (show changes without writing)
python migration/migrate-python.py --path src/ --dry-run

# Process single file
python migration/migrate-python.py --path src/client.py
```

### JavaScript Migration Script
```bash
# Transform all JavaScript/TypeScript files
npx jscodeshift -t migration/migrate-javascript.js src/

# Dry run (show changes without writing)
npx jscodeshift -t migration/migrate-javascript.js src/ --dry

# Process specific extensions only
npx jscodeshift -t migration/migrate-javascript.js src/ --extensions=ts,tsx
```

### Post-Migration Validation
```bash
# Validate migration completed successfully
./migration/validate-migration.sh
```

Both tools support `--dry-run` for previewing changes before applying them.

---

## Getting Help

If you encounter migration issues:
1. Check [docs/configuration.md](docs/configuration.md) for configuration reference
2. Review code examples in the `examples/` directory
3. File an issue on GitHub with your migration error
