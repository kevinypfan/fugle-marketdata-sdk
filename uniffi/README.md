# marketdata-uniffi

**EXPERIMENTAL** UniFFI bindings for Fugle marketdata-core.

> **Warning**: This module is in **alpha** status. The API may change without notice in future releases. Use at your own risk in production environments.

## Overview

This crate provides FFI bindings for multiple programming languages using [UniFFI](https://mozilla.github.io/uniffi-rs/). Supported languages include:

- C#
- Go
- C++
- Swift (macOS/iOS)
- Kotlin/Java (Android)

## Architecture

All methods return **JSON strings** for maximum language compatibility. This design choice:

- Avoids complex type mapping across different languages
- Allows parsing with native JSON libraries in each language
- Maintains flexibility for future API changes

## Building

```bash
# Build the library
cargo build -p marketdata-uniffi --release

# The library will be located at:
# target/release/libmarketdata_uniffi.dylib (macOS)
# target/release/libmarketdata_uniffi.so (Linux)
# target/release/marketdata_uniffi.dll (Windows)
```

## Generating Bindings

Use the `uniffi-bindgen` command to generate bindings for your target language:

```bash
# Install uniffi-bindgen-go for Go bindings
cargo install uniffi-bindgen-go --git https://github.com/AbelLykworking/uniffi-bindgen-go --tag v0.2.2+v0.28.3

# Generate Go bindings
uniffi-bindgen-go marketdata-uniffi/src/marketdata.udl -o ./bindings/go/

# For C# bindings, use uniffi-bindgen-cs
cargo install uniffi-bindgen-cs
uniffi-bindgen-cs marketdata-uniffi/src/marketdata.udl -o ./bindings/csharp/

# For C++ bindings
cargo run --bin uniffi-bindgen -- generate \
  marketdata-uniffi/src/marketdata.udl \
  --language cpp \
  --out-dir ./bindings/cpp/
```

## API Reference

### Creating a Client

```csharp
// C# Example
using MarketdataUniffi;

// Create client with SDK token
var client = MarketdataUniffi.NewRestClientWithSdkToken("your-sdk-token");

// Or with API key
var client = MarketdataUniffi.NewRestClientWithApiKey("your-api-key");

// Or with bearer token
var client = MarketdataUniffi.NewRestClientWithBearerToken("your-bearer-token");
```

### Stock Market Data

```csharp
// Get stock quote (returns JSON string)
var stock = client.Stock();
var intraday = stock.Intraday();

string quoteJson = intraday.Quote("2330");  // TSMC
string tickerJson = intraday.Ticker("2330");
string candlesJson = intraday.Candles("2330", "5");  // 5-minute candles
string tradesJson = intraday.Trades("2330");
string volumesJson = intraday.Volumes("2330");
```

### Futures and Options (FutOpt) Data

```csharp
// Get FutOpt quote
var futopt = client.Futopt();
var futoptIntraday = futopt.Intraday();

// Regular hours
string quoteJson = futoptIntraday.Quote("TXF202503", false);

// After hours
string quoteJson = futoptIntraday.Quote("TXF202503", true);

// Get available products
string futuresProducts = futoptIntraday.Products("F");  // Futures
string optionsProducts = futoptIntraday.Products("O");  // Options
```

### Error Handling

Errors are thrown as exceptions in target languages:

```csharp
try
{
    var quote = intraday.Quote("INVALID");
}
catch (MarketDataException.InvalidSymbol ex)
{
    Console.WriteLine($"Invalid symbol: {ex.Message}");
}
catch (MarketDataException.AuthError ex)
{
    Console.WriteLine($"Authentication failed: {ex.Message}");
}
catch (MarketDataException.ConnectionError ex)
{
    Console.WriteLine($"Connection error: {ex.Message}");
}
```

### Error Types

| Error Type | Description |
|-----------|-------------|
| `InvalidSymbol` | Invalid or unsupported symbol |
| `DeserializationError` | JSON parsing failed |
| `RuntimeError` | Internal runtime error |
| `ConfigError` | Configuration error |
| `ConnectionError` | Network connection failed |
| `AuthError` | Authentication failed |
| `ApiError` | API returned error response |
| `TimeoutError` | Operation timed out |
| `WebSocketError` | WebSocket protocol error |
| `Other` | Other unexpected errors |

## Response Format

All methods return JSON strings. Example response:

```json
{
  "symbol": "2330",
  "date": "2026-01-30",
  "time": "13:30:00",
  "open": 650.0,
  "high": 655.0,
  "low": 648.0,
  "close": 652.0,
  "volume": 12345678
}
```

## Dependencies

- UniFFI 0.28
- marketdata-core (internal)

## License

MIT
