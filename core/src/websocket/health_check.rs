//! WebSocket health check via passive activity timer
//!
//! The Fugle WebSocket server sends a `{"event":"heartbeat"}` frame every 30
//! seconds, so the SDK never needs to send its own ping. The health check
//! observes inbound traffic via [`HealthCheck::touch`] and disconnects when
//! the gap between successive frames exceeds `interval × max_missed_pongs`.
//!
//! Field names `interval` / `max_missed_pongs` are kept for binding API
//! stability; their semantic meaning is now "expected heartbeat period" and
//! "tolerated missed heartbeats" respectively.

use crate::websocket::ConnectionEvent;
use crate::MarketDataError;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Default health check enabled state - aligns with official SDKs (CON-01)
pub const DEFAULT_HEALTH_CHECK_ENABLED: bool = false;

/// Default expected heartbeat interval in milliseconds (Fugle server sends
/// heartbeat every 30 seconds).
pub const DEFAULT_HEALTH_CHECK_INTERVAL_MS: u64 = 30000;

/// Default number of heartbeat intervals to tolerate before disconnect.
/// 30s × 3 = 90s timeout — wide enough to absorb a brief network blip or
/// server GC pause without false-disconnecting a healthy stream.
pub const DEFAULT_HEALTH_CHECK_MAX_MISSED_PONGS: u64 = 3;

/// Minimum allowed health check interval to prevent excessive overhead
pub const MIN_HEALTH_CHECK_INTERVAL_MS: u64 = 5000;

/// Internal poll rate for the activity-timer task. Independent of `interval`
/// (which is the *expected heartbeat period*); 5 seconds is fast enough to
/// react promptly when the timeout is exceeded and cheap enough that the
/// per-tick atomic load is negligible.
const HEALTH_CHECK_TICK: Duration = Duration::from_secs(5);

fn current_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Configuration for WebSocket health check.
#[derive(Debug, Clone)]
pub struct HealthCheckConfig {
    /// Whether health check is enabled (default: false, aligned with official SDKs)
    pub enabled: bool,
    /// Expected server heartbeat interval (default: 30s).
    /// The server is expected to send some frame (heartbeat, data, or pong)
    /// at least once per this interval. Field name kept as `interval` for
    /// binding API stability; semantically this is the heartbeat period.
    pub interval: Duration,
    /// Number of heartbeat intervals to tolerate before disconnect
    /// (default: 3, giving a 90-second timeout). Field name kept as
    /// `max_missed_pongs` for binding API stability; semantically this is
    /// "missed heartbeats" in the activity-timer design.
    pub max_missed_pongs: u64,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            enabled: DEFAULT_HEALTH_CHECK_ENABLED,
            interval: Duration::from_millis(DEFAULT_HEALTH_CHECK_INTERVAL_MS),
            max_missed_pongs: DEFAULT_HEALTH_CHECK_MAX_MISSED_PONGS,
        }
    }
}

impl HealthCheckConfig {
    pub fn new(
        enabled: bool,
        interval: Duration,
        max_missed_pongs: u64,
    ) -> Result<Self, MarketDataError> {
        if interval < Duration::from_millis(MIN_HEALTH_CHECK_INTERVAL_MS) {
            return Err(MarketDataError::ConfigError(format!(
                "health_check interval must be >= {}ms (got {}ms)",
                MIN_HEALTH_CHECK_INTERVAL_MS,
                interval.as_millis()
            )));
        }

        if max_missed_pongs == 0 {
            return Err(MarketDataError::ConfigError(
                "max_missed_pongs must be >= 1".to_string(),
            ));
        }

        Ok(Self {
            enabled,
            interval,
            max_missed_pongs,
        })
    }

    pub fn with_interval(mut self, interval: Duration) -> Result<Self, MarketDataError> {
        if interval < Duration::from_millis(MIN_HEALTH_CHECK_INTERVAL_MS) {
            return Err(MarketDataError::ConfigError(format!(
                "health_check interval must be >= {}ms (got {}ms)",
                MIN_HEALTH_CHECK_INTERVAL_MS,
                interval.as_millis()
            )));
        }
        self.interval = interval;
        Ok(self)
    }

    pub fn with_max_missed_pongs(mut self, max: u64) -> Result<Self, MarketDataError> {
        if max == 0 {
            return Err(MarketDataError::ConfigError(
                "max_missed_pongs must be >= 1".to_string(),
            ));
        }
        self.max_missed_pongs = max;
        Ok(self)
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Disconnect when activity has been silent for this long.
    pub(crate) fn timeout(&self) -> Duration {
        self.interval
            .saturating_mul(self.max_missed_pongs.min(u32::MAX as u64) as u32)
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
        assert!(!config.enabled);
        assert_eq!(config.interval, Duration::from_secs(30));
        assert_eq!(config.max_missed_pongs, 3);
    }

    #[test]
    fn test_default_config_timeout_is_90s() {
        let config = HealthCheckConfig::default();
        assert_eq!(config.timeout(), Duration::from_secs(90));
    }

    #[test]
    fn test_config_builder() {
        let config = HealthCheckConfig::default()
            .with_interval(Duration::from_secs(60))
            .unwrap()
            .with_max_missed_pongs(5)
            .unwrap()
            .with_enabled(true);

        assert!(config.enabled);
        assert_eq!(config.interval, Duration::from_secs(60));
        assert_eq!(config.max_missed_pongs, 5);
        assert_eq!(config.timeout(), Duration::from_secs(300));
    }

    #[test]
    fn test_new_rejects_too_small_interval() {
        let result = HealthCheckConfig::new(true, Duration::from_secs(2), 3);
        assert!(result.is_err());
    }

    #[test]
    fn test_new_rejects_zero_max_missed_pongs() {
        let result = HealthCheckConfig::new(true, Duration::from_secs(30), 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_new_accepts_valid_config() {
        let result = HealthCheckConfig::new(true, Duration::from_secs(10), 5);
        assert!(result.is_ok());
        let config = result.unwrap();
        assert!(config.enabled);
        assert_eq!(config.interval, Duration::from_secs(10));
        assert_eq!(config.max_missed_pongs, 5);
    }

    #[test]
    fn test_builder_rejects_too_small_interval() {
        let result = HealthCheckConfig::default().with_interval(Duration::from_secs(1));
        assert!(result.is_err());
    }

    #[test]
    fn test_builder_rejects_zero_max_missed_pongs() {
        let result = HealthCheckConfig::default().with_max_missed_pongs(0);
        assert!(result.is_err());
    }

    #[test]
    fn test_new_accepts_minimum_interval() {
        let result = HealthCheckConfig::new(true, Duration::from_millis(5000), 1);
        assert!(result.is_ok());
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
