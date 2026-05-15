## ADDED Requirements

### Requirement: WebSocketErrorKind::Http documents the status to ErrorKind mapping

The doc-comment on the `WebSocketErrorKind::Http` variant in `core/src/errors.rs` SHALL contain the full `Http(status) → ErrorKind` mapping table as rendered by `MarketDataError::source_kind()`. The table MUST cover at minimum the following status families:

| Status range | `ErrorKind` | `is_retryable()` |
|---|---|---|
| `401`, `403` | `Auth` | `false` |
| `404`, other 4xx (excluding 401/403/429) | `Client` | `false` |
| `429` | `RateLimit` | `true` |
| `500..=599` | `Network` | `true` |
| anything else | `Client` | `false` |

`MarketDataError::source_kind`'s rustdoc SHALL retain a one-line cross-reference pointing at `WebSocketErrorKind::Http` so both entry points lead to the same authoritative table. No behavioural change is required — this requirement covers documentation topology only.

`core/src/errors.rs` SHALL include a `#[cfg(test)]` consistency assertion that constructs `MarketDataError::WebSocketError { kind: WebSocketErrorKind::Http(s), .. }` for at least the status codes `401`, `403`, `404`, `429`, `500`, `503`, `999` and verifies `source_kind()` and `is_retryable()` return the values documented in the table. Drift between the doc table and the impl MUST cause the test suite to fail.

#### Scenario: Variant doc lists every status family
- **WHEN** `cargo doc --all-features -p fugle-marketdata-core` is run and `WebSocketErrorKind::Http` is rendered
- **THEN** the rendered doc MUST contain rows for `401/403`, `404`, `429`, `500..=599`, and an explicit "anything else" row

#### Scenario: source_kind doc cross-references variant
- **WHEN** `MarketDataError::source_kind`'s rendered rustdoc is read
- **THEN** the doc MUST contain a clickable intra-doc link to `WebSocketErrorKind::Http`

#### Scenario: Consistency assertion catches drift
- **WHEN** a contributor edits `source_kind`'s `Http(429)` arm to return `ErrorKind::Network` (mismatching the documented `RateLimit`)
- **THEN** `cargo test -p fugle-marketdata-core` MUST fail in the consistency assertion test with a message naming the diverging status code

#### Scenario: All status families assert in the consistency test
- **WHEN** the consistency assertion test is read
- **THEN** the test MUST exercise at minimum the status codes `401`, `403`, `404`, `429`, `500`, `503`, and one additional code outside any documented range (e.g., `999`)
