//! WebSocket reconnection logic with exponential backoff

use std::time::Duration;

use crate::MarketDataError;

/// Default maximum reconnection attempts (VAL-02)
pub const DEFAULT_MAX_ATTEMPTS: u32 = 5;

/// Default initial reconnection delay in milliseconds (VAL-02)
pub const DEFAULT_INITIAL_DELAY_MS: u64 = 1000;

/// Default maximum reconnection delay in milliseconds (VAL-02)
pub const DEFAULT_MAX_DELAY_MS: u64 = 60000;

/// Minimum allowed initial delay to prevent connection storms
pub const MIN_INITIAL_DELAY_MS: u64 = 100;

/// Reconnection configuration
///
/// Controls automatic reconnection behavior after connection drops.
/// Defaults based on CONTEXT.md decisions.
#[derive(Debug, Clone)]
pub struct ReconnectionConfig {
    /// Maximum reconnection attempts before giving up
    pub max_attempts: u32,
    /// Initial delay before first reconnection attempt
    pub initial_delay: Duration,
    /// Maximum delay between reconnection attempts
    pub max_delay: Duration,
}

impl Default for ReconnectionConfig {
    fn default() -> Self {
        Self {
            max_attempts: DEFAULT_MAX_ATTEMPTS,
            initial_delay: Duration::from_millis(DEFAULT_INITIAL_DELAY_MS),
            max_delay: Duration::from_millis(DEFAULT_MAX_DELAY_MS),
        }
    }
}

impl ReconnectionConfig {
    /// Create a new reconnection config with custom values
    pub fn new(max_attempts: u32, initial_delay: Duration, max_delay: Duration) -> Self {
        Self {
            max_attempts,
            initial_delay,
            max_delay,
        }
    }

    /// Builder: set max attempts
    pub fn with_max_attempts(mut self, max_attempts: u32) -> Self {
        self.max_attempts = max_attempts;
        self
    }

    /// Builder: set initial delay
    pub fn with_initial_delay(mut self, initial_delay: Duration) -> Self {
        self.initial_delay = initial_delay;
        self
    }

    /// Builder: set max delay
    pub fn with_max_delay(mut self, max_delay: Duration) -> Self {
        self.max_delay = max_delay;
        self
    }
}

/// Manages reconnection attempts with exponential backoff
///
/// Tracks reconnection state and determines:
/// - Whether a close code is retriable
/// - Delay before next reconnection attempt
/// - When max attempts have been reached
pub struct ReconnectionManager {
    config: ReconnectionConfig,
    current_attempt: u32,
}

impl ReconnectionManager {
    /// Create a new reconnection manager
    pub fn new(config: ReconnectionConfig) -> Self {
        Self {
            config,
            current_attempt: 0,
        }
    }

    /// Determine if reconnection should be attempted based on close code
    ///
    /// From CONTEXT.md decisions:
    /// - 1001 (Going away) → reconnect
    /// - 1006 (Abnormal closure) → reconnect
    /// - 4001 (Auth failure) → don't reconnect
    /// - 4000-4999 (Application errors) → don't reconnect
    /// - 1000 (Normal closure) → don't reconnect
    /// - Others → reconnect by default
    pub fn should_reconnect(&self, close_code: Option<u16>) -> bool {
        match close_code {
            Some(1000) => false, // Normal closure
            Some(1001) => true,  // Going away
            Some(1006) => true,  // Abnormal closure
            Some(4001) => false, // Auth failure
            Some(code) if (4000..=4999).contains(&code) => false, // Application errors
            _ => true, // Default: reconnect on unknown errors
        }
    }

    /// Calculate next reconnection delay with exponential backoff and jitter
    ///
    /// Returns None if max attempts reached, Some(duration) otherwise.
    /// Increments attempt counter.
    pub fn next_delay(&mut self) -> Option<Duration> {
        if self.current_attempt >= self.config.max_attempts {
            return None;
        }

        self.current_attempt += 1;

        // Calculate exponential backoff: initial * 2^(attempt-1)
        let exponential_millis = self.config.initial_delay.as_millis()
            * 2_u128.pow((self.current_attempt - 1).min(10)); // Cap at 2^10 to avoid overflow

        // Apply max_delay cap
        let capped_millis = exponential_millis.min(self.config.max_delay.as_millis());

        // Add simple deterministic jitter based on attempt number (0-15% of delay)
        // This avoids thundering herd without requiring rand dependency
        let jitter_percent = (self.current_attempt * 3) % 16; // 0-15%
        let jitter = (capped_millis * jitter_percent as u128) / 100;
        let final_millis = capped_millis.saturating_add(jitter);

        Some(Duration::from_millis(final_millis as u64))
    }

    /// Reset reconnection state
    ///
    /// Clears attempt counter, allowing fresh reconnection.
    /// Used after successful reconnection or manual reconnect() call.
    pub fn reset(&mut self) {
        self.current_attempt = 0;
    }

    /// Get number of remaining reconnection attempts
    pub fn attempts_remaining(&self) -> u32 {
        self.config.max_attempts.saturating_sub(self.current_attempt)
    }

    /// Get current attempt number
    pub fn current_attempt(&self) -> u32 {
        self.current_attempt
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reconnection_config_default() {
        let config = ReconnectionConfig::default();
        assert_eq!(config.max_attempts, 5);
        assert_eq!(config.initial_delay, Duration::from_secs(1));
        assert_eq!(config.max_delay, Duration::from_secs(60));
    }

    #[test]
    fn test_reconnection_config_builder() {
        let config = ReconnectionConfig::default()
            .with_max_attempts(10)
            .with_initial_delay(Duration::from_secs(2))
            .with_max_delay(Duration::from_secs(120));

        assert_eq!(config.max_attempts, 10);
        assert_eq!(config.initial_delay, Duration::from_secs(2));
        assert_eq!(config.max_delay, Duration::from_secs(120));
    }

    #[test]
    fn test_should_reconnect_on_1006() {
        let config = ReconnectionConfig::default();
        let manager = ReconnectionManager::new(config);

        // 1006 (Abnormal closure) should reconnect
        assert!(manager.should_reconnect(Some(1006)));
    }

    #[test]
    fn test_should_reconnect_on_1001() {
        let config = ReconnectionConfig::default();
        let manager = ReconnectionManager::new(config);

        // 1001 (Going away) should reconnect
        assert!(manager.should_reconnect(Some(1001)));
    }

    #[test]
    fn test_should_not_reconnect_on_4001() {
        let config = ReconnectionConfig::default();
        let manager = ReconnectionManager::new(config);

        // 4001 (Auth failure) should not reconnect
        assert!(!manager.should_reconnect(Some(4001)));
    }

    #[test]
    fn test_should_not_reconnect_on_1000() {
        let config = ReconnectionConfig::default();
        let manager = ReconnectionManager::new(config);

        // 1000 (Normal closure) should not reconnect
        assert!(!manager.should_reconnect(Some(1000)));
    }

    #[test]
    fn test_should_not_reconnect_on_4xxx() {
        let config = ReconnectionConfig::default();
        let manager = ReconnectionManager::new(config);

        // Application errors (4000-4999) should not reconnect
        assert!(!manager.should_reconnect(Some(4000)));
        assert!(!manager.should_reconnect(Some(4500)));
        assert!(!manager.should_reconnect(Some(4999)));
    }

    #[test]
    fn test_should_reconnect_on_unknown() {
        let config = ReconnectionConfig::default();
        let manager = ReconnectionManager::new(config);

        // Unknown errors should reconnect by default
        assert!(manager.should_reconnect(Some(1002)));
        assert!(manager.should_reconnect(Some(1003)));
        assert!(manager.should_reconnect(None));
    }

    #[test]
    fn test_exponential_backoff_delays() {
        let config = ReconnectionConfig::default();
        let mut manager = ReconnectionManager::new(config);

        // First delay should be returned
        let delay1 = manager.next_delay();
        assert!(delay1.is_some());
        assert_eq!(manager.current_attempt(), 1);

        // Delays should increase (exponential backoff)
        let delay2 = manager.next_delay();
        assert!(delay2.is_some());
        assert_eq!(manager.current_attempt(), 2);

        // Continue getting delays up to max_attempts
        let _ = manager.next_delay();
        let _ = manager.next_delay();
        let _ = manager.next_delay();

        // After max_attempts (5), should return None
        let delay_final = manager.next_delay();
        assert!(delay_final.is_none());
    }

    #[test]
    fn test_reset_clears_attempts() {
        let config = ReconnectionConfig::default();
        let mut manager = ReconnectionManager::new(config);

        // Exhaust attempts
        let _ = manager.next_delay();
        let _ = manager.next_delay();
        assert_eq!(manager.current_attempt(), 2);

        // Reset should clear attempts
        manager.reset();
        assert_eq!(manager.current_attempt(), 0);
        assert_eq!(manager.attempts_remaining(), 5);

        // Should be able to get delays again
        let delay = manager.next_delay();
        assert!(delay.is_some());
    }

    #[test]
    fn test_max_attempts_reached() {
        let config = ReconnectionConfig::default().with_max_attempts(3);
        let mut manager = ReconnectionManager::new(config);

        // Get 3 delays
        assert!(manager.next_delay().is_some());
        assert!(manager.next_delay().is_some());
        assert!(manager.next_delay().is_some());

        // 4th attempt should return None
        assert!(manager.next_delay().is_none());
        assert_eq!(manager.attempts_remaining(), 0);
    }

    #[test]
    fn test_attempts_remaining() {
        let config = ReconnectionConfig::default().with_max_attempts(5);
        let mut manager = ReconnectionManager::new(config);

        assert_eq!(manager.attempts_remaining(), 5);

        let _ = manager.next_delay();
        assert_eq!(manager.attempts_remaining(), 4);

        let _ = manager.next_delay();
        assert_eq!(manager.attempts_remaining(), 3);
    }
}
