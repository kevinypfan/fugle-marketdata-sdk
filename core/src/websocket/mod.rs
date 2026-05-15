//! WebSocket client for real-time market data streaming
//!
//! This module provides:
//! - WebSocket connection lifecycle management
//! - Configuration for connection parameters
//! - State machine for connection states
//! - Event notifications for connection events
//! - Channel-specific subscription and parsing

pub mod aio;
pub mod channels;
pub mod config;
pub mod connection_event;
pub mod health_check;
pub mod message;
pub(crate) mod protocol;
pub mod reconnection;
pub mod subscription;

// Re-export public types
pub use aio::WebSocketClient;
pub use channels::StockSubscription;
pub use config::ConnectionConfig;
pub use connection_event::{ConnectionEvent, ConnectionState};
pub use health_check::HealthCheckConfig;
pub use message::MessageReceiver;
pub use reconnection::{ReconnectionConfig, ReconnectionManager};
pub use subscription::SubscriptionManager;
