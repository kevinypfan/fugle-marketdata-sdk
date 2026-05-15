# rest-retry Specification

## Purpose
TBD - created by archiving change sdk-04-improvements. Update Purpose after archive.
## Requirements
### Requirement: Opt-in REST retry policy
`RestClient` SHALL expose a builder-style method `with_retry(policy: RetryPolicy)` that enables transparent retry of failed requests. By default (no `with_retry` call), the client SHALL NOT retry and SHALL surface every transport or API error to the caller on the first failure.

#### Scenario: Default behavior is no retry
- **WHEN** a `RestClient` is built without `with_retry` and the upstream returns HTTP 503 once
- **THEN** the call MUST return `Err(MarketDataError::ApiError { status: 503, .. })` after exactly one HTTP request

#### Scenario: Configured retry succeeds on second attempt
- **WHEN** a `RestClient` is built with `with_retry(RetryPolicy { max_attempts: 3, .. })` and the upstream returns 503 on attempt 1, 200 on attempt 2
- **THEN** the call MUST return `Ok(_)` after exactly 2 HTTP requests

### Requirement: Retry only on retryable errors
When retry is enabled, the client SHALL only retry errors for which `MarketDataError::is_retryable()` returns `true` (currently HTTP 429 and HTTP 5xx per `core/src/errors.rs:134`). All other errors MUST be returned to the caller immediately without consuming a retry attempt.

#### Scenario: HTTP 401 is not retried
- **WHEN** the upstream returns HTTP 401 and `with_retry(RetryPolicy { max_attempts: 5, .. })` is configured
- **THEN** the call MUST return after exactly one HTTP request

#### Scenario: HTTP 429 is retried
- **WHEN** the upstream returns HTTP 429 once then HTTP 200, with `max_attempts: 3`
- **THEN** the call MUST succeed after 2 HTTP requests

### Requirement: Exponential backoff with jitter
`RetryPolicy` SHALL define `max_attempts: u32`, `initial_backoff: Duration`, and `max_backoff: Duration`. Between attempts, the client SHALL sleep for `min(initial_backoff * 2^(attempt-1), max_backoff)` plus a uniform random jitter in the range `[0, initial_backoff)`.

#### Scenario: Backoff caps at max_backoff
- **WHEN** `RetryPolicy { initial_backoff: 100ms, max_backoff: 500ms, max_attempts: 10 }` is used and 5 retryable failures occur
- **THEN** no inter-attempt sleep MUST exceed `500ms + initial_backoff` (jitter ceiling)

### Requirement: Retry exhaustion surfaces last error
When retries are exhausted, the client SHALL return the last error received from the upstream, not a synthetic wrapper.

#### Scenario: Exhaustion preserves last error
- **WHEN** `max_attempts: 3` is configured and the upstream returns HTTP 503 three times
- **THEN** the returned `Err` MUST be `MarketDataError::ApiError { status: 503, .. }` (the actual last error, not a generic "retries exhausted" variant)

