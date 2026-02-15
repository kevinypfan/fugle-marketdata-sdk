# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0] - 2026-02-16

### Added
- Options object constructor for all language bindings (Python kwargs-only, Node.js options object, Java builder, Go functional options, C# options pattern)
- ReconnectConfig/ReconnectionConfig exposure for WebSocket auto-reconnect control (max_attempts, initial_delay_ms, max_delay_ms)
- HealthCheckConfig/HealthCheckOptions exposure for WebSocket health check control (enabled, interval_ms, max_missed_pongs)
- Exactly-one-auth validation at construction time (Python ValueError, Node.js Error, Java FugleException, Go error, C# ArgumentException)
- Configuration validation at construction time with descriptive error messages
- Java builder pattern for client and config classes
- Go functional options pattern (WithApiKey, WithBearerToken, WithSdkToken)
- C# options pattern with nullable properties
- Configuration constants exported from core (DEFAULT_*, MIN_* constants for binding layers)

### Changed
- **BREAKING**: Python constructors now require kwargs-only parameters (`RestClient(api_key=)`, not `RestClient("key")`)
- **BREAKING**: Node.js constructors now require options object (`new RestClient({ apiKey })`, not `new RestClient('key')`)
- **BREAKING**: Java constructors now require builder pattern (`FugleRestClient.builder().apiKey().build()`)
- **BREAKING**: Go constructors now require functional options (`NewFugleRestClient(WithApiKey("key"))`)
- **BREAKING**: C# constructors now require options classes (`new RestClient(new RestClientOptions { ApiKey = "key" })`)
- Health check default changed from `true` to `false` (aligned with official SDKs)
- ReconnectConfig field rename: `max_retries` → `max_attempts`, `base_delay_ms` → `initial_delay_ms`

### Deprecated
- Python: Positional string constructors (`RestClient("key")`, removed in v0.4.0)
- Python: Static methods `.with_bearer_token()` and `.with_sdk_token()` (removed in v0.4.0)
- Node.js: String constructors (`new RestClient('key')`, removed in v0.4.0)

## [0.2.0] - 2026-01-31

### Added
- Multi-language SDK support (Python, Node.js, C#, Java, Go)
- Complete REST API coverage (26+ endpoints across stock and futures/options)
  - Stock intraday: quote, ticker, candles, trades, volumes
  - Stock historical: candles, stats
  - Stock snapshot: quotes, movers, actives
  - Stock technical: SMA, RSI, KDJ, MACD, Bollinger Bands
  - Stock corporate actions: capital changes, dividends, listing applicants
  - FutOpt intraday: quote, ticker, candles, trades, volumes, products
  - FutOpt historical: candles, daily
- WebSocket streaming with automatic reconnection and exponential backoff
- WebSocket health check monitoring (ping-pong)
- Async support for all language bindings
  - Python: async/await with asyncio
  - Node.js: Promise-based API
  - C#: Task-based async
  - Java: CompletableFuture
  - Go: goroutines and channels
- Type definitions
  - TypeScript: Full .d.ts definitions for Node.js
  - Python: PEP 484 type stubs (.pyi files)
- Error handling with consistent error codes across all languages
- Three authentication methods: API key, bearer token, SDK token
- FFI bindings via PyO3 (Python), napi-rs (Node.js), UniFFI (Java/Go/C#)

[unreleased]: https://github.com/yourusername/fugle-marketdata-sdk/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/yourusername/fugle-marketdata-sdk/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/yourusername/fugle-marketdata-sdk/releases/tag/v0.2.0
