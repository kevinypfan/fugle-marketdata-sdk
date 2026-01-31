package tw.com.fugle.marketdata;

import tw.com.fugle.marketdata.generated.*;

import java.util.concurrent.*;

/**
 * Idiomatic Java wrapper for WebSocket client with dual streaming patterns.
 *
 * <p>Supports two modes of operation:
 * <ul>
 *   <li><b>Callback mode:</b> Provide a WebSocketListener for push-based events</li>
 *   <li><b>Pull mode:</b> Use poll()/take() to consume messages from a BlockingQueue</li>
 * </ul>
 *
 * <h3>Example - Callback Mode:</h3>
 * <pre>{@code
 * WebSocketListener listener = new WebSocketListener() {
 *     public void onConnected() {
 *         System.out.println("Connected!");
 *     }
 *     public void onDisconnected() {
 *         System.out.println("Disconnected");
 *     }
 *     public void onMessage(StreamMessage message) {
 *         System.out.println("Event: " + message.event());
 *     }
 *     public void onError(String error) {
 *         System.err.println("Error: " + error);
 *     }
 * };
 *
 * try (FugleWebSocketClient client = FugleWebSocketClient.builder()
 *         .apiKey("YOUR_API_KEY")
 *         .stock()
 *         .listener(listener)
 *         .build()) {
 *
 *     client.connect().get();
 *     client.subscribe("trades", "2330").get();
 *     Thread.sleep(10000);
 * }
 * }</pre>
 *
 * <h3>Example - Pull Mode:</h3>
 * <pre>{@code
 * try (FugleWebSocketClient client = FugleWebSocketClient.builder()
 *         .apiKey("YOUR_API_KEY")
 *         .stock()
 *         .queueCapacity(1000)
 *         .build()) {
 *
 *     client.connect().get();
 *     client.subscribe("trades", "2330").get();
 *
 *     while (true) {
 *         StreamMessage msg = client.poll(1, TimeUnit.SECONDS);
 *         if (msg != null) {
 *             System.out.println("Event: " + msg.event());
 *         }
 *
 *         // Check for async errors
 *         if (client.hasErrors()) {
 *             String error = client.pollError();
 *             System.err.println("Async error: " + error);
 *         }
 *     }
 * }
 * }</pre>
 */
public class FugleWebSocketClient implements AutoCloseable {

    private final WebSocketClient webSocketClient;
    private final BlockingQueue<StreamMessage> messageQueue;
    private final BlockingQueue<String> errorQueue;

    private FugleWebSocketClient(WebSocketClient webSocketClient,
                                  BlockingQueue<StreamMessage> messageQueue,
                                  BlockingQueue<String> errorQueue) {
        this.webSocketClient = webSocketClient;
        this.messageQueue = messageQueue;
        this.errorQueue = errorQueue;
    }

    /**
     * Connect to the WebSocket server.
     *
     * @return CompletableFuture that completes when connected
     * @throws ApiException if connection fails
     * @throws AuthException if authentication fails
     */
    public CompletableFuture<Void> connect() {
        return webSocketClient.connect()
                .exceptionally(e -> { throw FugleException.unwrap(e); });
    }

    /**
     * Disconnect from the WebSocket server.
     *
     * @return CompletableFuture that completes when disconnected
     */
    public CompletableFuture<Void> disconnect() {
        return webSocketClient.disconnect();
    }

    /**
     * Check if currently connected.
     */
    public boolean isConnected() {
        try {
            return webSocketClient.isConnected();
        } catch (Exception e) {
            throw FugleException.unwrap(e);
        }
    }

    /**
     * Subscribe to a channel for a symbol.
     *
     * @param channel Channel name (e.g., "trades", "candles", "books")
     * @param symbol Symbol to subscribe (e.g., "2330")
     * @return CompletableFuture that completes when subscribed
     * @throws ApiException if subscription fails
     */
    public CompletableFuture<Void> subscribe(String channel, String symbol) {
        return webSocketClient.subscribe(channel, symbol)
                .exceptionally(e -> { throw FugleException.unwrap(e); });
    }

    /**
     * Unsubscribe from a channel for a symbol.
     *
     * @param channel Channel name
     * @param symbol Symbol to unsubscribe
     * @return CompletableFuture that completes when unsubscribed
     */
    public CompletableFuture<Void> unsubscribe(String channel, String symbol) {
        return webSocketClient.unsubscribe(channel, symbol)
                .exceptionally(e -> { throw FugleException.unwrap(e); });
    }

    /**
     * Poll for the next message (non-blocking).
     *
     * <p>Only available in pull mode (when listener was not provided).
     *
     * @return Next message, or null if queue is empty
     */
    public StreamMessage poll() {
        if (messageQueue == null) {
            throw new IllegalStateException("poll() only available in pull mode (no listener provided)");
        }
        return messageQueue.poll();
    }

    /**
     * Poll for the next message with timeout.
     *
     * <p>Only available in pull mode (when listener was not provided).
     *
     * @param timeout Maximum time to wait
     * @param unit Time unit for timeout
     * @return Next message, or null if timeout expires
     * @throws InterruptedException if interrupted while waiting
     */
    public StreamMessage poll(long timeout, TimeUnit unit) throws InterruptedException {
        if (messageQueue == null) {
            throw new IllegalStateException("poll() only available in pull mode (no listener provided)");
        }
        return messageQueue.poll(timeout, unit);
    }

    /**
     * Take the next message (blocking).
     *
     * <p>Only available in pull mode (when listener was not provided).
     *
     * @return Next message (blocks until available)
     * @throws InterruptedException if interrupted while waiting
     */
    public StreamMessage take() throws InterruptedException {
        if (messageQueue == null) {
            throw new IllegalStateException("take() only available in pull mode (no listener provided)");
        }
        return messageQueue.take();
    }

    /**
     * Get the current message queue size.
     *
     * <p>Only available in pull mode (when listener was not provided).
     *
     * @return Number of messages in queue
     */
    public int queueSize() {
        if (messageQueue == null) {
            throw new IllegalStateException("queueSize() only available in pull mode (no listener provided)");
        }
        return messageQueue.size();
    }

    /**
     * Check if any async errors have occurred.
     *
     * <p>Only available in pull mode (when listener was not provided).
     *
     * @return true if errors are in the error queue
     */
    public boolean hasErrors() {
        if (errorQueue == null) {
            throw new IllegalStateException("hasErrors() only available in pull mode (no listener provided)");
        }
        return !errorQueue.isEmpty();
    }

    /**
     * Poll for the next error (non-blocking).
     *
     * <p>Only available in pull mode (when listener was not provided).
     *
     * @return Next error message, or null if no errors
     */
    public String pollError() {
        if (errorQueue == null) {
            throw new IllegalStateException("pollError() only available in pull mode (no listener provided)");
        }
        return errorQueue.poll();
    }

    @Override
    public void close() {
        webSocketClient.close();
    }

    /**
     * Create a new builder.
     */
    public static Builder builder() {
        return new Builder();
    }

    /**
     * Builder for FugleWebSocketClient.
     */
    public static class Builder {
        private String apiKey;
        private WebSocketEndpoint endpoint = WebSocketEndpoint.STOCK;
        private WebSocketListener listener;
        private int queueCapacity = 10000;

        private Builder() {}

        /**
         * Set API key for authentication.
         */
        public Builder apiKey(String apiKey) {
            this.apiKey = apiKey;
            return this;
        }

        /**
         * Use stock market data endpoint (default).
         */
        public Builder stock() {
            this.endpoint = WebSocketEndpoint.STOCK;
            return this;
        }

        /**
         * Use futures and options market data endpoint.
         */
        public Builder futopt() {
            this.endpoint = WebSocketEndpoint.FUT_OPT;
            return this;
        }

        /**
         * Use futures and options market data endpoint (alias).
         */
        public Builder futOpt() {
            return futopt();
        }

        /**
         * Provide a listener for callback mode (push-based events).
         *
         * <p>If a listener is provided, poll()/take() methods will throw IllegalStateException.
         */
        public Builder listener(WebSocketListener listener) {
            this.listener = listener;
            return this;
        }

        /**
         * Set message queue capacity for pull mode.
         *
         * <p>Default: 10000
         * <p>Only used when no listener is provided.
         */
        public Builder queueCapacity(int capacity) {
            this.queueCapacity = capacity;
            return this;
        }

        /**
         * Build the FugleWebSocketClient.
         *
         * @throws IllegalStateException if API key is not set
         */
        public FugleWebSocketClient build() {
            if (apiKey == null || apiKey.isEmpty()) {
                throw new IllegalStateException("API key is required");
            }

            WebSocketListener effectiveListener;
            BlockingQueue<StreamMessage> messageQueue;
            BlockingQueue<String> errorQueue;

            if (listener != null) {
                // Callback mode: use provided listener directly
                effectiveListener = listener;
                messageQueue = null;
                errorQueue = null;
            } else {
                // Pull mode: create internal listener with BlockingQueue
                messageQueue = new LinkedBlockingQueue<>(queueCapacity);
                errorQueue = new LinkedBlockingQueue<>();

                effectiveListener = new InternalListener(messageQueue, errorQueue);
            }

            WebSocketClient client = WebSocketClient.newWithEndpoint(apiKey, effectiveListener, endpoint);

            return new FugleWebSocketClient(client, messageQueue, errorQueue);
        }
    }

    /**
     * Internal listener for pull mode that forwards events to BlockingQueues.
     */
    private static class InternalListener implements WebSocketListener {
        private final BlockingQueue<StreamMessage> messageQueue;
        private final BlockingQueue<String> errorQueue;

        InternalListener(BlockingQueue<StreamMessage> messageQueue,
                        BlockingQueue<String> errorQueue) {
            this.messageQueue = messageQueue;
            this.errorQueue = errorQueue;
        }

        @Override
        public void onConnected() {
            // No action needed in pull mode
        }

        @Override
        public void onDisconnected() {
            // No action needed in pull mode
        }

        @Override
        public void onMessage(StreamMessage message) {
            // Offer to queue (non-blocking, drops if full)
            messageQueue.offer(message);
        }

        @Override
        public void onError(String errorMessage) {
            // Offer to error queue
            errorQueue.offer(errorMessage);
        }
    }
}
