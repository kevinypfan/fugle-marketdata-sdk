//! Basic WebSocket streaming example
//!
//! This example demonstrates how to use the WebSocket client for real-time market data.
//!
//! # Prerequisites
//!
//! Set the `FUGLE_API_KEY` environment variable:
//! ```bash
//! export FUGLE_API_KEY="your-api-key"
//! ```
//!
//! # Run
//! ```bash
//! cargo run --example websocket_basic
//! ```

use marketdata_core::{
    websocket::{ConnectionConfig, ConnectionEvent, StockSubscription},
    AuthRequest, Channel, WebSocketClient,
};
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), marketdata_core::MarketDataError> {
    // Get API key from environment
    let api_key =
        std::env::var("FUGLE_API_KEY").expect("FUGLE_API_KEY environment variable not set");

    println!("=== WebSocket Streaming Example ===\n");

    // Create WebSocket client for stock market
    let config = ConnectionConfig::fugle_stock(AuthRequest::with_api_key(&api_key));
    let client = WebSocketClient::new(config);

    // Connect to server
    println!("Connecting to WebSocket server...");
    client.connect().await?;
    println!("Connected and authenticated!\n");

    // Subscribe to channels
    println!("Subscribing to channels...");

    // Subscribe to trades channel
    let trades_sub = StockSubscription::new(Channel::Trades, "2330");
    client.subscribe_channel(trades_sub).await?;
    println!("  Subscribed to 2330 trades");

    // Subscribe to books channel
    let books_sub = StockSubscription::new(Channel::Books, "2330");
    client.subscribe_channel(books_sub).await?;
    println!("  Subscribed to 2330 books");

    // Subscribe to aggregates
    let aggregates_sub = StockSubscription::new(Channel::Aggregates, "2330");
    client.subscribe_channel(aggregates_sub).await?;
    println!("  Subscribed to 2330 aggregates");

    // Subscribe to multiple symbols
    client
        .subscribe_symbols(Channel::Trades, &["2317", "2454"], false)
        .await?;
    println!("  Subscribed to 2317, 2454 trades");

    println!("\nListening for messages (10 seconds)...\n");

    // Get message receiver
    let messages = client.messages();

    // Get event receiver for connection events
    let events = Arc::clone(client.state_events());

    // Spawn event monitoring task
    let event_handle = tokio::spawn(async move {
        loop {
            let event = tokio::task::spawn_blocking({
                let events = Arc::clone(&events);
                move || {
                    let rx = events.blocking_lock();
                    rx.recv_timeout(Duration::from_millis(100))
                }
            })
            .await;

            match event {
                Ok(Ok(ConnectionEvent::Disconnected { code, reason })) => {
                    println!("Event: Disconnected (code: {:?}, reason: {})", code, reason);
                    break;
                }
                Ok(Ok(event)) => {
                    println!("Event: {:?}", event);
                }
                Ok(Err(_)) => {
                    // Timeout - continue
                }
                Err(_) => break,
            }
        }
    });

    // Process messages for 10 seconds
    let start = std::time::Instant::now();
    let duration = Duration::from_secs(10);
    let mut message_count = 0;

    while start.elapsed() < duration {
        match messages.receive_timeout(Duration::from_secs(1)) {
            Ok(Some(msg)) => {
                message_count += 1;

                if msg.is_data() {
                    println!(
                        "[{}] Data: channel={:?}, symbol={:?}",
                        message_count,
                        msg.channel.as_deref().unwrap_or("?"),
                        msg.symbol.as_deref().unwrap_or("?")
                    );

                    // Parse and display specific data types
                    if let Some(ref data) = msg.data {
                        if msg.channel.as_deref() == Some("trades") {
                            let price = data.get("price").and_then(serde_json::Value::as_f64);
                            let size = data.get("size").and_then(serde_json::Value::as_i64);
                            if let (Some(price), Some(size)) = (price, size) {
                                println!("        Trade: price={}, size={}", price, size);
                            }
                        } else if msg.channel.as_deref() == Some("aggregates") {
                            let close = data.get("closePrice").and_then(serde_json::Value::as_f64);
                            if let Some(close) = close {
                                println!("        Aggregate: closePrice={}", close);
                            }
                        }
                    }
                } else if msg.is_subscribed() {
                    println!(
                        "[{}] Subscribed: id={:?}",
                        message_count,
                        msg.id.as_deref().unwrap_or("?")
                    );
                } else if msg.is_error() {
                    println!(
                        "[{}] Error: {:?}",
                        message_count,
                        msg.error_message().unwrap_or_else(|| "Unknown".to_string())
                    );
                } else {
                    println!("[{}] Event: {}", message_count, msg.event);
                }
            }
            Ok(None) => {
                // Timeout - check if we should continue
                if !client.is_connected().await {
                    println!("Connection lost");
                    break;
                }
            }
            Err(e) => {
                println!("Channel error: {}", e);
                break;
            }
        }
    }

    // Print summary
    println!("\n=== Summary ===");
    println!("Total messages received: {}", message_count);
    println!("Active subscriptions: {:?}", client.subscription_keys());

    // Graceful disconnect
    println!("\nDisconnecting...");
    client.disconnect().await?;
    println!("Disconnected gracefully");

    // Wait for event handler to finish
    event_handle.abort();

    // Verify client is closed
    if client.is_closed().await {
        println!("Client is now closed (cannot be reused)");
    }

    println!("\n=== Complete ===");
    println!("WebSocket example finished successfully.");

    Ok(())
}
