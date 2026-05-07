//! WebSocket connection liveness detection.
//!
//! In 3.0 the SDK uses a single timeout window: if no inbound frame
//! arrives within `heartbeat_timeout`, the connection is declared dead
//! and the reconnect path takes over. This file still hosts the old
//! polling-task runtime struct ([`HealthCheck`]); a follow-up commit
//! replaces that with a read-site `tokio::time::timeout` in the
//! dispatch loop and removes the runtime struct entirely. Until then,
//! the runtime struct remains as the integration point for
//! `connection.rs` and continues to drive `ConnectionEvent::Disconnected`.

use crate::websocket::ConnectionEvent;
use crate::MarketDataError;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Liveness detection enabled by default in 3.0 (was opt-in in 2.x).
/// Silent-by-default lets a stalled connection sit unnoticed until the
/// OS eventually times out the underlying TCP — typically hours.
pub const DEFAULT_HEALTH_CHECK_ENABLED: bool = true;

/// Default heartbeat timeout: Fugle server's 30s heartbeat period plus
/// 5s buffer to absorb network jitter. Mirrors Databento's
/// `heartbeat_interval + 5` convention.
pub const DEFAULT_HEARTBEAT_TIMEOUT_MS: u64 = 35_000;

/// Absolute floor for [`HealthCheckConfig::heartbeat_timeout`]. This is
/// a sanity floor, **not** a "safe value" — values below the actual
/// server heartbeat period (currently 30s) will cause repeated false
/// disconnects. Settings under 35s only make sense in tests, or once
/// the server supports negotiated heartbeat interval (Phase 2.3 in the
/// SDK roadmap; see `WEBSOCKET-SERVER-RECOMMENDATIONS.md`).
pub const MIN_HEARTBEAT_TIMEOUT_MS: u64 = 5_000;

/// Internal poll rate for the activity-timer task. Independent of the
/// configured `heartbeat_timeout`; 5 seconds is fast enough to react
/// promptly when the timeout is exceeded, cheap enough that the
/// per-tick atomic load is negligible. Removed when the dispatch loop
/// inlines the timeout in a follow-up commit.
const HEALTH_CHECK_TICK: Duration = Duration::from_secs(5);

fn current_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Configuration for WebSocket connection liveness detection.
///
/// A single timeout window controls when the SDK declares the
/// connection dead: if no inbound frame (heartbeat, data, anything)
/// arrives within `heartbeat_timeout`, the dispatch path emits
/// [`ConnectionEvent::HeartbeatTimeout`](crate::websocket::ConnectionEvent::HeartbeatTimeout)
/// and exits, which lets the reconnect manager take over.
#[derive(Debug, Clone)]
pub struct HealthCheckConfig {
    /// Whether liveness detection is active. Default: `true` (changed
    /// from `false` in 2.x). Use [`HealthCheckConfig::disabled`] to
    /// opt out — discouraged outside test environments because a
    /// silent connection won't surface until the OS times out the
    /// underlying TCP, typically hours later.
    pub enabled: bool,

    /// Maximum allowed gap between inbound frames before declaring
    /// the connection dead.
    ///
    /// Default: 35s (the Fugle server emits a heartbeat every 30s;
    /// 5s buffer absorbs network jitter). Use [`HealthCheckConfig::with_timeout`]
    /// to construct with validation.
    pub heartbeat_timeout: Duration,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            enabled: DEFAULT_HEALTH_CHECK_ENABLED,
            heartbeat_timeout: Duration::from_millis(DEFAULT_HEARTBEAT_TIMEOUT_MS),
        }
    }
}

impl HealthCheckConfig {
    /// Construct an enabled config with the given timeout.
    ///
    /// Returns [`MarketDataError::ConfigError`] if `timeout` is below
    /// the absolute sanity floor ([`MIN_HEARTBEAT_TIMEOUT_MS`]).
    /// Note: this only enforces a floor, not a value that's actually
    /// safe against the live server's heartbeat period. See the
    /// constant's docs.
    pub fn with_timeout(timeout: Duration) -> Result<Self, MarketDataError> {
        if timeout < Duration::from_millis(MIN_HEARTBEAT_TIMEOUT_MS) {
            return Err(MarketDataError::ConfigError(format!(
                "heartbeat_timeout must be >= {}ms (got {:?})",
                MIN_HEARTBEAT_TIMEOUT_MS, timeout
            )));
        }
        Ok(Self {
            enabled: true,
            heartbeat_timeout: timeout,
        })
    }

    /// Construct a disabled config. Without liveness detection a
    /// stalled connection won't surface until the OS times out the
    /// underlying TCP — typically hours on Linux defaults.
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            heartbeat_timeout: Duration::from_millis(DEFAULT_HEARTBEAT_TIMEOUT_MS),
        }
    }

    /// The timeout window. Used by the runtime [`HealthCheck`] struct
    /// during this migration; will be inlined into the dispatch loop
    /// when the runtime struct is removed in a follow-up commit.
    pub(crate) fn timeout(&self) -> Duration {
        self.heartbeat_timeout
    }
}

/// Passive activity-timer health check.
///
/// Observes inbound WebSocket traffic via [`Self::touch`]. A background tokio
/// task wakes periodically and disconnects when the gap exceeds the
/// configured timeout.
pub struct HealthCheck {
    config: HealthCheckConfig,
    last_activity_ms: Arc<AtomicU64>,
    should_stop: Arc<AtomicBool>,
    is_paused: Arc<AtomicBool>,
}

impl HealthCheck {
    pub fn new(config: HealthCheckConfig) -> Self {
        Self {
            config,
            // Initialize to "now". Without this, the first tick computes
            // age = (millis since epoch), instantly triggering a false
            // disconnect before any frame arrives.
            last_activity_ms: Arc::new(AtomicU64::new(current_time_ms())),
            should_stop: Arc::new(AtomicBool::new(false)),
            is_paused: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Update the last-activity timestamp. Called by `dispatch_messages` on
    /// every successfully decoded inbound frame.
    pub fn touch(&self) {
        self.last_activity_ms
            .store(current_time_ms(), Ordering::Relaxed);
    }

    /// Time since the last touch.
    pub fn last_activity_age(&self) -> Duration {
        let last = self.last_activity_ms.load(Ordering::Relaxed);
        Duration::from_millis(current_time_ms().saturating_sub(last))
    }

    /// Pause activity timer checking.
    ///
    /// **Invariant**: only called from the reconnect path, when the
    /// connection is known to be dead. Real disconnects during pause would
    /// otherwise be masked.
    pub fn pause(&self) {
        self.is_paused.store(true, Ordering::SeqCst);
    }

    pub fn resume(&self) {
        // Reset age in case the pause window was long.
        self.touch();
        self.is_paused.store(false, Ordering::SeqCst);
    }

    pub fn stop(&self) {
        self.should_stop.store(true, Ordering::SeqCst);
    }

    pub fn is_healthy(&self) -> bool {
        self.last_activity_age() < self.config.timeout()
    }

    pub fn is_paused(&self) -> bool {
        self.is_paused.load(Ordering::SeqCst)
    }

    pub fn config(&self) -> &HealthCheckConfig {
        &self.config
    }

    /// Spawn the periodic activity-timer task.
    pub fn spawn_check_task(
        &self,
        event_tx: mpsc::Sender<ConnectionEvent>,
    ) -> tokio::task::JoinHandle<()> {
        let timeout = self.config.timeout();
        let last_activity = Arc::clone(&self.last_activity_ms);
        let should_stop = Arc::clone(&self.should_stop);
        let is_paused = Arc::clone(&self.is_paused);

        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(HEALTH_CHECK_TICK);
            ticker.tick().await; // skip immediate first tick

            loop {
                ticker.tick().await;
                if should_stop.load(Ordering::SeqCst) {
                    break;
                }
                if is_paused.load(Ordering::SeqCst) {
                    continue;
                }

                let last = last_activity.load(Ordering::Relaxed);
                let age = Duration::from_millis(current_time_ms().saturating_sub(last));

                if age >= timeout {
                    let _ = event_tx.send(ConnectionEvent::Disconnected {
                        code: None,
                        reason: format!(
                            "Health check timeout: no activity for {}s",
                            age.as_secs()
                        ),
                    });
                    break;
                }
            }
        })
    }

    /// Manually trigger a ping (no-op kept for API compatibility).
    pub fn ping(&self) -> Result<(), MarketDataError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_default_config() {
        let config = HealthCheckConfig::default();
        assert!(config.enabled, "3.0 default is enabled=true");
        assert_eq!(config.heartbeat_timeout, Duration::from_secs(35));
    }

    #[test]
    fn test_default_config_timeout_is_35s() {
        let config = HealthCheckConfig::default();
        assert_eq!(config.timeout(), Duration::from_secs(35));
    }

    #[test]
    fn test_disabled_factory() {
        let config = HealthCheckConfig::disabled();
        assert!(!config.enabled);
        // heartbeat_timeout is still set (to default) but unused when disabled.
        assert_eq!(config.heartbeat_timeout, Duration::from_secs(35));
    }

    #[test]
    fn test_with_timeout_accepts_60s() {
        let config = HealthCheckConfig::with_timeout(Duration::from_secs(60)).unwrap();
        assert!(config.enabled);
        assert_eq!(config.heartbeat_timeout, Duration::from_secs(60));
    }

    #[test]
    fn test_with_timeout_accepts_5s_minimum() {
        let result = HealthCheckConfig::with_timeout(Duration::from_millis(5000));
        assert!(result.is_ok(), "5s is at the floor and must be accepted");
    }

    #[test]
    fn test_with_timeout_rejects_below_5s() {
        let result = HealthCheckConfig::with_timeout(Duration::from_millis(4_999));
        assert!(result.is_err(), "below 5s floor must be rejected");
    }

    #[test]
    fn test_new_initializes_last_activity_to_now() {
        let hc = HealthCheck::new(HealthCheckConfig::default());
        assert!(
            hc.last_activity_age() < Duration::from_millis(100),
            "HealthCheck::new must initialize last_activity_ms to current time"
        );
        assert!(hc.is_healthy());
    }

    #[test]
    fn test_touch_resets_age() {
        let hc = HealthCheck::new(HealthCheckConfig::default());
        thread::sleep(Duration::from_millis(50));
        assert!(hc.last_activity_age() >= Duration::from_millis(50));

        hc.touch();
        assert!(hc.last_activity_age() < Duration::from_millis(20));
    }

    #[test]
    fn test_age_grows_over_time() {
        let hc = HealthCheck::new(HealthCheckConfig::default());
        thread::sleep(Duration::from_millis(120));
        assert!(hc.last_activity_age() >= Duration::from_millis(100));
    }

    #[test]
    fn test_is_healthy_false_after_timeout() {
        let hc = HealthCheck::new(HealthCheckConfig::default());
        // Manually set last activity to a time far in the past.
        let stale = current_time_ms().saturating_sub(200_000);
        hc.last_activity_ms.store(stale, Ordering::Relaxed);
        assert!(!hc.is_healthy());
    }

    #[test]
    fn test_pause_resume_resets_age() {
        let hc = HealthCheck::new(HealthCheckConfig::default());
        hc.pause();
        thread::sleep(Duration::from_millis(50));
        assert!(hc.is_paused());
        hc.resume();
        assert!(!hc.is_paused());
        assert!(hc.last_activity_age() < Duration::from_millis(20));
    }

    #[test]
    fn test_stop_flag() {
        let hc = HealthCheck::new(HealthCheckConfig::default());
        assert!(!hc.should_stop.load(Ordering::SeqCst));
        hc.stop();
        assert!(hc.should_stop.load(Ordering::SeqCst));
    }

    #[test]
    fn test_manual_ping_noop() {
        let hc = HealthCheck::new(HealthCheckConfig::default());
        assert!(hc.ping().is_ok());
    }
}
