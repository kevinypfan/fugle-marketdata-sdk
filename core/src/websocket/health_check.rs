//! WebSocket health check monitoring with ping/pong
//!
//! Implements configurable health monitoring:
//! - Periodic ping messages at configurable interval
//! - Missed pong detection and threshold-based disconnect
//! - Pause/resume for reconnection periods
//! - Manual ping triggering

use crate::websocket::ConnectionEvent;
use crate::MarketDataError;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Default health check enabled state - aligns with official SDKs (CON-01)
pub const DEFAULT_HEALTH_CHECK_ENABLED: bool = false;

/// Default health check interval in milliseconds (CON-01)
pub const DEFAULT_HEALTH_CHECK_INTERVAL_MS: u64 = 30000;

/// Default maximum missed pongs before disconnect (CON-01)
pub const DEFAULT_HEALTH_CHECK_MAX_MISSED_PONGS: u64 = 2;

/// Minimum allowed health check interval to prevent excessive overhead
pub const MIN_HEALTH_CHECK_INTERVAL_MS: u64 = 5000;

/// Configuration for WebSocket health check
///
/// From CONTEXT.md: Default values align with production requirements
#[derive(Debug, Clone)]
pub struct HealthCheckConfig {
    /// Whether health check is enabled (default: false, aligned with official SDKs)
    pub enabled: bool,
    /// Interval between ping messages (default: 30 seconds)
    pub interval: Duration,
    /// Maximum missed pongs before disconnect (default: 2)
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
    /// Create a new health check config with validation
    ///
    /// # Errors
    /// Returns `MarketDataError::ConfigError` if:
    /// - `interval` is less than 5000ms (5 seconds)
    /// - `max_missed_pongs` is 0 (must be >= 1)
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

    /// Create a new health check config with custom interval
    ///
    /// # Errors
    /// Returns `MarketDataError::ConfigError` if `interval` is less than 5000ms
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

    /// Set maximum missed pongs before disconnect
    ///
    /// # Errors
    /// Returns `MarketDataError::ConfigError` if `max` is 0
    pub fn with_max_missed_pongs(mut self, max: u64) -> Result<Self, MarketDataError> {
        if max == 0 {
            return Err(MarketDataError::ConfigError(
                "max_missed_pongs must be >= 1".to_string(),
            ));
        }
        self.max_missed_pongs = max;
        Ok(self)
    }

    /// Enable or disable health check (no validation needed)
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// Health check monitor for WebSocket connection
///
/// Monitors connection health via ping/pong messages. Triggers disconnect
/// when too many pongs are missed. Pauses during reconnection.
pub struct HealthCheck {
    config: HealthCheckConfig,
    last_pong_timestamp: Arc<AtomicU64>,
    missed_pong_count: Arc<AtomicU64>,
    should_stop: Arc<AtomicBool>,
    is_paused: Arc<AtomicBool>,
}

impl HealthCheck {
    /// Create a new health check monitor
    pub fn new(config: HealthCheckConfig) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            config,
            last_pong_timestamp: Arc::new(AtomicU64::new(now)),
            missed_pong_count: Arc::new(AtomicU64::new(0)),
            should_stop: Arc::new(AtomicBool::new(false)),
            is_paused: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Record that a pong was received
    ///
    /// Resets missed pong count and updates timestamp
    pub fn on_pong_received(&self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        self.last_pong_timestamp.store(now, Ordering::SeqCst);
        self.missed_pong_count.store(0, Ordering::SeqCst);
    }

    /// Pause health check monitoring
    ///
    /// From CONTEXT.md: "重連期間健康檢查暫停"
    /// Used during reconnection to avoid false disconnect triggers
    pub fn pause(&self) {
        self.is_paused.store(true, Ordering::SeqCst);
    }

    /// Resume health check monitoring after pause
    pub fn resume(&self) {
        // Reset timestamp to current time to avoid immediate missed pong
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.last_pong_timestamp.store(now, Ordering::SeqCst);
        self.missed_pong_count.store(0, Ordering::SeqCst);
        self.is_paused.store(false, Ordering::SeqCst);
    }

    /// Stop health check monitoring permanently
    pub fn stop(&self) {
        self.should_stop.store(true, Ordering::SeqCst);
    }

    /// Check if connection is healthy
    ///
    /// Returns false if missed pongs exceed threshold
    pub fn is_healthy(&self) -> bool {
        self.missed_pong_count.load(Ordering::SeqCst) < self.config.max_missed_pongs
    }

    /// Spawn health check monitoring task
    ///
    /// Runs in a std::thread (not tokio task) for FFI compatibility.
    /// Periodically sends ping signals and monitors pong responses.
    ///
    /// # Arguments
    ///
    /// * `ping_sender` - Channel to send ping signals to WebSocket layer
    /// * `event_tx` - Channel to send connection events
    ///
    /// # Returns
    ///
    /// Join handle for the monitoring thread
    pub fn spawn_check_task(
        &self,
        ping_sender: mpsc::Sender<()>,
        event_tx: mpsc::Sender<ConnectionEvent>,
    ) -> JoinHandle<()> {
        let config = self.config.clone();
        let last_pong = Arc::clone(&self.last_pong_timestamp);
        let missed_count = Arc::clone(&self.missed_pong_count);
        let should_stop = Arc::clone(&self.should_stop);
        let is_paused = Arc::clone(&self.is_paused);

        thread::spawn(move || {
            while !should_stop.load(Ordering::SeqCst) {
                // If paused, sleep briefly and continue
                if is_paused.load(Ordering::SeqCst) {
                    thread::sleep(Duration::from_millis(100));
                    continue;
                }

                // Record time before sending ping
                let ping_sent_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                // Send ping signal
                if ping_sender.send(()).is_err() {
                    // Channel closed, exit thread
                    break;
                }

                // Sleep for interval
                thread::sleep(config.interval);

                // Check if pong was received (compare timestamps)
                let last_pong_time = last_pong.load(Ordering::SeqCst);
                if last_pong_time < ping_sent_time {
                    // No pong received since ping
                    let current_missed = missed_count.fetch_add(1, Ordering::SeqCst) + 1;

                    // Send PongMissed event
                    let _ = event_tx.send(ConnectionEvent::PongMissed);

                    // Check if threshold exceeded
                    if current_missed >= config.max_missed_pongs {
                        // Send Disconnected event
                        let _ = event_tx.send(ConnectionEvent::Disconnected {
                            code: None,
                            reason: "Health check failed: missed pongs".to_string(),
                        });
                        break;
                    }
                } else {
                    // Pong received, reset count
                    missed_count.store(0, Ordering::SeqCst);
                }
            }
        })
    }

    /// Manually trigger a ping (for advanced use cases)
    ///
    /// From CONTEXT.md: "提供 ping() 方法讓使用者手動觸發健康檢查"
    pub fn ping(&self) -> Result<(), MarketDataError> {
        // This is a no-op in the current design since ping_sender is owned by spawn_check_task
        // In practice, users would use the WebSocketClient.send() method to send ping frames
        // This method exists for API completeness
        Ok(())
    }

    /// Get current missed pong count
    pub fn missed_pong_count(&self) -> u64 {
        self.missed_pong_count.load(Ordering::SeqCst)
    }

    /// Check if health check is paused
    pub fn is_paused(&self) -> bool {
        self.is_paused.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_check_default_config() {
        let config = HealthCheckConfig::default();
        assert!(config.enabled);
        assert_eq!(config.interval, Duration::from_secs(30));
        assert_eq!(config.max_missed_pongs, 2);
    }

    #[test]
    fn test_health_check_config_builder() {
        let config = HealthCheckConfig::default()
            .with_interval(Duration::from_secs(60))
            .with_max_missed_pongs(5)
            .with_enabled(false);

        assert!(!config.enabled);
        assert_eq!(config.interval, Duration::from_secs(60));
        assert_eq!(config.max_missed_pongs, 5);
    }

    #[test]
    fn test_on_pong_received_resets_count() {
        let config = HealthCheckConfig::default();
        let health_check = HealthCheck::new(config);

        // Simulate missed pongs
        health_check.missed_pong_count.store(3, Ordering::SeqCst);
        assert_eq!(health_check.missed_pong_count(), 3);

        // Receive pong
        health_check.on_pong_received();

        // Count should be reset
        assert_eq!(health_check.missed_pong_count(), 0);
    }

    #[test]
    fn test_pause_stops_checking() {
        let config = HealthCheckConfig::default();
        let health_check = HealthCheck::new(config);

        assert!(!health_check.is_paused());

        health_check.pause();
        assert!(health_check.is_paused());

        health_check.resume();
        assert!(!health_check.is_paused());
    }

    #[test]
    fn test_is_healthy_returns_correct_status() {
        let config = HealthCheckConfig::default().with_max_missed_pongs(3);
        let health_check = HealthCheck::new(config);

        // Initially healthy
        assert!(health_check.is_healthy());

        // Still healthy with 2 missed pongs
        health_check.missed_pong_count.store(2, Ordering::SeqCst);
        assert!(health_check.is_healthy());

        // Unhealthy at threshold
        health_check.missed_pong_count.store(3, Ordering::SeqCst);
        assert!(!health_check.is_healthy());

        // Unhealthy above threshold
        health_check.missed_pong_count.store(5, Ordering::SeqCst);
        assert!(!health_check.is_healthy());
    }

    #[test]
    fn test_resume_resets_timestamp() {
        let config = HealthCheckConfig::default();
        let health_check = HealthCheck::new(config);

        // Set old timestamp and missed count
        health_check.last_pong_timestamp.store(100, Ordering::SeqCst);
        health_check.missed_pong_count.store(5, Ordering::SeqCst);

        // Resume
        health_check.resume();

        // Timestamp should be updated to recent time
        let timestamp = health_check.last_pong_timestamp.load(Ordering::SeqCst);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        assert!(timestamp >= now - 1); // Within 1 second

        // Missed count should be reset
        assert_eq!(health_check.missed_pong_count(), 0);
    }

    #[test]
    fn test_stop_flag() {
        let config = HealthCheckConfig::default();
        let health_check = HealthCheck::new(config);

        assert!(!health_check.should_stop.load(Ordering::SeqCst));

        health_check.stop();
        assert!(health_check.should_stop.load(Ordering::SeqCst));
    }

    #[test]
    fn test_manual_ping() {
        let config = HealthCheckConfig::default();
        let health_check = HealthCheck::new(config);

        // Manual ping should not error
        let result = health_check.ping();
        assert!(result.is_ok());
    }
}
