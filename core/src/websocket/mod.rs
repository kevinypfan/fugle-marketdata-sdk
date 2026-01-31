//! WebSocket client for real-time market data streaming
//!
//! This module provides:
//! - WebSocket connection lifecycle management
//! - Configuration for connection parameters
//! - State machine for connection states
//! - Event notifications for connection events
//! - Channel-specific subscription and parsing

pub mod channels;
pub mod config;
pub mod connection;
pub mod health_check;
pub mod message;
pub mod reconnection;
pub mod subscription;

// Re-export public types
pub use channels::StockSubscription;
pub use config::ConnectionConfig;
pub use connection::{ConnectionEvent, ConnectionState, WebSocketClient};
pub use health_check::{HealthCheck, HealthCheckConfig};
pub use message::MessageReceiver;
pub use reconnection::{ReconnectionConfig, ReconnectionManager};
pub use subscription::SubscriptionManager;
