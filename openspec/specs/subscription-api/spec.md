# subscription-api Specification

## Purpose
TBD - created by archiving change sdk-04-improvements. Update Purpose after archive.
## Requirements
### Requirement: Subscription introspection helpers
Both sync and async `WebSocketClient` SHALL expose:
- `subscription_count(&self) -> usize` returning the number of currently active subscriptions
- `is_subscribed(&self, channel: &Channel, symbol: &str) -> bool` returning whether the given channel+symbol pair is currently subscribed

Both helpers MUST be backed by the existing `SubscriptionManager::contains()` (`core/src/websocket/subscription.rs:103`) and `SubscriptionManager::count()` (`subscription.rs:109`); no new internal data structures may be introduced.

#### Scenario: Count reflects active subscriptions
- **WHEN** a client subscribes to two distinct channel+symbol pairs and both succeed
- **THEN** `client.subscription_count()` MUST return 2

#### Scenario: is_subscribed positive case
- **WHEN** the client has an active `Channel::Trades` + `"2330"` subscription
- **THEN** `client.is_subscribed(&Channel::Trades, "2330")` MUST return `true`

#### Scenario: is_subscribed negative case
- **WHEN** the client has an active `Channel::Trades` + `"2330"` subscription but NO `Channel::Books` + `"2330"` subscription
- **THEN** `client.is_subscribed(&Channel::Books, "2330")` MUST return `false`

#### Scenario: Count after unsubscribe
- **WHEN** the client subscribes to two pairs then unsubscribes from one
- **THEN** `client.subscription_count()` MUST return 1

### Requirement: Legacy SubscribeRequest constructors removed
The constructor methods `SubscribeRequest::trades`, `SubscribeRequest::candles`, `SubscribeRequest::books`, and `SubscribeRequest::aggregates` (currently at `core/src/models/subscription.rs:212-229`) SHALL be removed. The canonical builder API (`SubscribeRequest::new(channel, symbol)` plus chainable modifiers) is the supported replacement.

**Reason**: The 0.2.0 changelog already removed `SubscribeRequest` from the public re-export surface, but these helper constructors remained dangling. They duplicate the canonical `new(channel, symbol)` form and create two ways to do the same thing, complicating future channel additions.

**Migration**: Replace `SubscribeRequest::trades(symbol)` with `SubscribeRequest::new(Channel::Trades, symbol)`. Same pattern for `candles`, `books`, `aggregates` with their corresponding `Channel::*` variants.

#### Scenario: Removed constructors do not compile
- **WHEN** downstream code calls `SubscribeRequest::trades("2330")` after upgrading to 0.4.0
- **THEN** the code MUST fail to compile with a method-not-found error pointing to the canonical `new(channel, symbol)` constructor

#### Scenario: Canonical form preserved
- **WHEN** downstream code calls `SubscribeRequest::new(Channel::Trades, "2330")`
- **THEN** the code MUST compile and produce a request equivalent to the removed `SubscribeRequest::trades("2330")` form

