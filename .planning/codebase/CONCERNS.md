# Codebase Concerns

**Analysis Date:** 2026-01-30

## Tech Debt

**Missing afterHours support for WebSocket subscriptions:**
- Issue: TODO comment in `py/src/websocket.rs:768` indicates afterHours support not yet added to SubscribeRequest in marketdata-core
- Files: `py/src/websocket.rs`, `core/src/models/subscription.rs`
- Impact: WebSocket clients cannot subscribe to after-hours trading data; this limits market coverage for trading during extended sessions
- Fix approach: Add `after_hours: bool` field to SubscribeRequest struct in core, propagate through subscription logic, update Python bindings to accept this parameter

**Connection complexity in WebSocketClient:**
- Issue: `core/src/websocket/connection.rs` is 1805 lines (largest single file), handling connection lifecycle, message dispatch, health checks, and reconnection
- Files: `core/src/websocket/connection.rs`
- Impact: File size makes code hard to review and modify; potential for bugs in intricate state management and async task coordination
- Fix approach: Split into multiple modules - separate concerns into connection establishment, message dispatching, and state management; extract health check integration into cleaner interface

**RestClient not thread-safe:**
- Issue: `core/src/rest/client.rs` explicitly documents RestClient is NOT Send/Sync due to ureq::Agent limitations
- Files: `core/src/rest/client.rs` (lines 19-20)
- Impact: Multi-threaded Python/JavaScript applications cannot safely share a single RestClient instance; each thread must create its own client, increasing resource overhead
- Fix approach: Consider wrapping in Arc<Mutex> at binding layer, or document thread-pooling pattern for users; evaluate upgrading to different HTTP client library if Send/Sync is critical

**Panic handling at FFI boundaries:**
- Issue: Panic catching at FFI boundaries in `core/src/runtime.rs` uses `eprintln!()` for logging with minimal context
- Files: `core/src/runtime.rs` (lines 20-42)
- Impact: Panics are silently logged to stderr only; consumers have no programmatic way to detect or handle runtime panics; difficult to debug in production
- Fix approach: Implement proper panic callback mechanism in AsyncRuntime, propagate panic information through error channels to FFI consumers

## Known Bugs

**Authentication header format in bearer tokens:**
- Symptoms: Bearer token authentication works but no validation that token format is correct
- Files: `core/src/rest/auth.rs` (line 19)
- Trigger: Sending invalid bearer token format (e.g., missing "Bearer " prefix when client already adds it) causes 401 from server
- Workaround: API key authentication works reliably as fallback

**WebSocket reconnection jitter is deterministic:**
- Symptoms: All reconnection attempts follow predictable jitter pattern based on attempt number
- Files: `core/src/websocket/reconnection.rs` (line 118)
- Trigger: Multiple SDK instances in same deployment will retry simultaneously without true randomness, potential "thundering herd" during mass disconnection
- Workaround: Use custom ReconnectionConfig with different settings per client to stagger retries

## Security Considerations

**API Key exposure in headers:**
- Risk: Authentication tokens (ApiKey, SdkToken, Bearer) are sent in HTTP headers without explicit HTTPS-only enforcement at SDK level
- Files: `core/src/rest/auth.rs`, `core/src/rest/client.rs` (lines 37-40)
- Current mitigation: TLS connection required to API endpoint; ureq validates certificates by default with native-tls
- Recommendations: Document that HTTPS is required; add runtime check to reject non-https base URLs; consider pinning certificate for api.fugle.tw

**Credentials in error messages:**
- Risk: Error messages may contain partial credentials or session tokens
- Files: `core/src/errors.rs`, `core/src/websocket/connection.rs`
- Current mitigation: Error messages use generic strings without embedding actual credentials
- Recommendations: Audit all error formatting to ensure no credential leakage; add log redaction for credential fields in debug output

**WebSocket TLS handling:**
- Risk: WebSocket TLS errors are mapped to AuthError, potentially masking certificate validation failures
- Files: `core/src/errors.rs` (line 80)
- Current mitigation: Tokio-tungstenite uses native-tls which performs certificate validation
- Recommendations: Distinguish between TLS validation failures and auth failures; add certificate pinning support for production

## Performance Bottlenecks

**Health check using standard thread instead of async:**
- Problem: Health check runs in separate std::thread with mpsc channels, not integrated with Tokio runtime
- Files: `core/src/websocket/health_check.rs` (lines 14-15)
- Cause: Health check spawned as separate OS thread with sleep() calls; creates thread context switch overhead for every ping interval
- Improvement path: Migrate health check to Tokio task, use tokio::time::interval for efficient scheduling; integrate with connection task for unified polling

**Message receiver using std::sync::mpsc:**
- Problem: MessageReceiver in `core/src/websocket/message.rs` uses synchronous mpsc::Receiver which blocks Python threads
- Files: `core/src/websocket/message.rs`, `py/src/iterator.rs`
- Cause: FFI boundary requires sync channels; blocking receiver prevents GIL release in Python async contexts
- Improvement path: Add async iterator interface for Python, implement non-blocking peek method for polling patterns

**Exponential backoff with power calculations on each retry:**
- Problem: `core/src/websocket/reconnection.rs` recalculates exponential backoff with 2^n on each next_delay() call
- Files: `core/src/websocket/reconnection.rs` (line 111)
- Cause: Minor CPU cost per calculation; could be precomputed
- Improvement path: Precompute delay table in ReconnectionConfig::new() to make next_delay() O(1) constant lookup

**No connection pooling metrics:**
- Problem: ureq Agent connection pooling is opaque; no visibility into pool utilization
- Files: `core/src/rest/client.rs`
- Cause: Cannot tune pool size or detect connection starvation
- Improvement path: Expose ureq pool metrics if available; add builder option for max connections per client

## Fragile Areas

**WebSocket subscription state synchronization:**
- Files: `core/src/websocket/subscription.rs`, `core/src/websocket/connection.rs`
- Why fragile: Subscriptions can be added/removed while reconnection is happening; no transactional guarantee that all subscriptions are resubscribed after reconnect; potential to lose subscriptions mid-reconnect if connection fails during resubscription
- Safe modification: Require comprehensive tests for edge cases: (1) adding subscription during reconnect, (2) removing subscription during reconnect, (3) connection loss during bulk resubscription
- Test coverage: Integration tests exist but coverage gaps for race conditions during state transitions

**Error type classification for retryability:**
- Files: `core/src/errors.rs` (lines 121-131)
- Why fragile: is_retryable() classification is hardcoded; adding new error variants requires updating this logic; risk of new errors defaulting to wrong retryability
- Safe modification: Extract retryability classification into trait impl, add explicit `#[must_use]` on is_retryable() results, add property-based tests to verify classification consistency
- Test coverage: Tests cover known error types but don't test dynamic error type addition

**FFI safety at runtime boundaries:**
- Files: `core/src/runtime.rs`, `py/src/websocket.rs`, `js/src/websocket.rs`
- Why fragile: Panics are caught but state may be partially corrupted; AsyncRuntime is global-ish but accessed through thread-local patterns in different bindings
- Safe modification: Add invariant checks before panic boundary calls; document assumption that tasks don't hold locks across FFI calls; implement task-local cleanup on panic
- Test coverage: No panic recovery tests; error cases that trigger panics untested

**Date/time handling in health check and reconnection:**
- Files: `core/src/websocket/health_check.rs` (line 75-78), `core/src/websocket/reconnection.rs` (line 122)
- Why fragile: Uses UNIX_EPOCH with SystemTime which can be affected by system clock adjustments; missed pong detection relies on monotonic time but uses wall clock in health_check
- Safe modification: Use tokio::time::Instant (monotonic) for all timing logic instead of SystemTime; test with simulated clock skew
- Test coverage: No tests for system clock adjustments or leap second handling

## Scaling Limits

**Single runtime instance pattern:**
- Current capacity: One Tokio runtime per Python process; JavaScript uses Node.js event loop (single-threaded)
- Limit: As number of WebSocket connections grows, single Tokio multi-threaded runtime may become bottleneck
- Scaling path: For high-volume scenarios (1000+ concurrent subscriptions), consider per-connection runtime or connection pooling strategy; add runtime metrics to identify bottlenecks

**Message channel capacity:**
- Current capacity: Default Rust mpsc channels are unbounded (can grow until memory exhausted)
- Limit: Burst of incoming messages from WebSocket can cause unbounded queue growth if consumer slow
- Scaling path: Implement bounded channels with backpressure; add message dropping strategy for overload conditions; expose queue depth metrics

**Event channel buffer:**
- Current capacity: ConnectionEvent channel has default buffer (typically small)
- Limit: Rapid connection state changes (reconnect spam) can overflow event channel, dropping events
- Scaling path: Implement event queue with size limits and overflow handling; add circuit breaker pattern to slow down reconnection attempts if event channel overflows

## Dependencies at Risk

**tokio-tungstenite with native-tls:**
- Risk: Native-tls delegates to OS-provided TLS implementation (OpenSSL on Linux, SecureTransport on macOS); vulnerabilities propagate from OS packages
- Impact: Cannot patch TLS vulnerabilities without OS-level updates; no control over TLS version/cipher configuration
- Migration plan: Consider rustls as alternative for pure-Rust TLS stack; would increase binary size but reduce OS dependency; requires feature flag for backwards compatibility

**ureq synchronous HTTP client:**
- Risk: ureq is synchronous-only, blocks threads; limits scalability for multi-threaded REST clients
- Impact: Cannot efficiently handle high-concurrency REST scenarios; thread pool exhaustion if creating one thread per REST request
- Migration plan: Consider reqwest (async) or hyper for REST layer; would require async runtime integration throughout REST chain; breaking API change for sync consumers

**Exponential backoff crate with limited configuration:**
- Risk: exponential-backoff crate v2.0 may have unfixed bugs; limited customization options
- Impact: Cannot customize jitter algorithm; cannot cap exponential growth except via max_delay
- Migration plan: Consider implementing backoff inline if features become limiting; provides full control but increases code complexity

## Missing Critical Features

**No circuit breaker for API errors:**
- Problem: Repeated 429 (rate limit) responses from API trigger continuous retries without backing off
- Blocks: Applications cannot gracefully degrade under rate limiting; no way to detect if API is temporarily unavailable
- Risk: Can cause cascading failures if many SDK instances hit rate limit simultaneously

**No message deduplication after reconnect:**
- Problem: After WebSocket reconnect, may receive duplicate messages for brief window during resubscription
- Blocks: Real-time trading applications require exactly-once semantics; duplicates can cause double-execution of trades
- Risk: Production trading systems could place duplicate orders

**No built-in metrics/observability:**
- Problem: No way to monitor connection health, message rates, or error frequencies programmatically
- Blocks: Cannot identify performance issues in production; no way to implement alerting
- Risk: Silent failures go undetected until business impact occurs

## Test Coverage Gaps

**WebSocket reconnection under network instability:**
- What's not tested: Reconnection behavior with frequent but short disconnections (every 5-10 seconds); flaky network scenarios; partial connection failures during resubscription
- Files: `core/tests/integration_websocket.rs`
- Risk: Production network jitter (common in wireless) may trigger untested code paths; reconnection storms possible
- Priority: High - reconnection is critical path for streaming data reliability

**FFI boundary panic recovery:**
- What's not tested: Panics during FFI calls from Python/JavaScript; cleanup after panic; state consistency after recovery
- Files: `core/src/runtime.rs`, `py/src/`, `js/src/`
- Risk: Panics could leave dangling resources or corrupted state in consumer code; error handling broken after panic
- Priority: High - FFI safety is critical for production use

**Concurrent subscription/unsubscription:**
- What's not tested: Multiple threads adding/removing subscriptions simultaneously during active connection
- Files: `core/src/websocket/subscription.rs`, `core/tests/`
- Risk: Race conditions could lose subscriptions or corrupt subscription state; only tested in sequential scenarios
- Priority: High - concurrent clients common in production

**Error message sanitization:**
- What's not tested: Error messages with credentials, tokens, or sensitive data; verification that redaction works
- Files: All error handling code
- Risk: Credential leakage in logs/error messages; security incident vector
- Priority: Medium - could expose tokens in debug output

**Health check timeout handling:**
- What's not tested: Health check behavior when pong responses delayed beyond expected interval; timeout cascades
- Files: `core/src/websocket/health_check.rs`
- Risk: False positives disconnecting healthy connections under network latency; unnecessary reconnects
- Priority: Medium - affects stability under high latency conditions

**REST API error response parsing:**
- What's not tested: Malformed API responses, missing expected fields, API version changes
- Files: `core/src/rest/error.rs`, REST endpoint handlers
- Risk: Panic or incorrect error classification if API changes response format; not handled gracefully
- Priority: Medium - API evolution risk

---

*Concerns audit: 2026-01-30*
