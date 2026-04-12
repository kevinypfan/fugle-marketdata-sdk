package tw.com.fugle.marketdata.generated;


import java.util.List;
import java.util.Map;
import java.util.concurrent.CompletableFuture;
import com.sun.jna.*;
import com.sun.jna.ptr.*;
/**
 * WebSocket client for real-time market data streaming
 *
 * Wraps the core WebSocketClient and forwards messages to the provided
 * WebSocketListener implementation via a background task.
 */
public interface WebSocketClientInterface {
    
    /**
     * Connect to the WebSocket server
     *
     * Establishes connection, authenticates, and starts a background task
     * to forward messages to the listener.
     *
     * # Errors
     *
     * Returns error if connection or authentication fails.
     */
    public CompletableFuture<Void> connect() ;
    
    /**
     * Disconnect from the WebSocket server
     *
     * Gracefully closes the connection and stops the message forwarding task.
     */
    public CompletableFuture<Void> disconnect();
    
    /**
     * Check if the client has been shut down
     */
    public Boolean isClosed();
    
    /**
     * Check if the client is currently connected
     */
    public Boolean isConnected();
    
    /**
     * Send a ping message to the server
     *
     * # Arguments
     * * `state` - Optional state string echoed back in the pong response
     */
    public CompletableFuture<Void> ping(String state) ;
    
    /**
     * Query the server for current subscriptions
     */
    public CompletableFuture<Void> querySubscriptions() ;
    
    /**
     * Subscribe to a channel for a symbol
     *
     * # Arguments
     * * `channel` - Channel name (e.g., "trades", "candles", "books")
     * * `symbol` - Symbol to subscribe (e.g., "2330")
     *
     * # Errors
     *
     * Returns error if not connected or subscription fails.
     */
    public CompletableFuture<Void> subscribe(String channel, String symbol) ;
    
    /**
     * Unsubscribe from a channel for a symbol
     *
     * # Arguments
     * * `channel` - Channel name
     * * `symbol` - Symbol to unsubscribe
     *
     * # Errors
     *
     * Returns error if not connected.
     */
    public CompletableFuture<Void> unsubscribe(String channel, String symbol) ;
    
}

