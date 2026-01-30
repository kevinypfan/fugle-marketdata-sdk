"""Type stubs for marketdata_py

Fugle Market Data SDK - Python bindings with full type annotations.
"""
from typing import Any, Callable, Optional, List

__version__: str

# Exception hierarchy
class MarketDataError(Exception):
    """Base exception for all market data errors.

    All SDK exceptions inherit from this class, making it easy to catch
    any SDK-related error with a single except clause.
    """
    ...

class ApiError(MarketDataError):
    """API returned an error response.

    Raised when the Fugle API returns an error status code or error message.
    Contains details about the specific API error.
    """
    ...

class AuthError(MarketDataError):
    """Authentication failed.

    Raised when the API key is invalid, expired, or missing.
    """
    ...

class RateLimitError(ApiError):
    """Rate limit exceeded.

    Raised when too many requests have been made in a short period.
    Inherits from ApiError as it's a specific API error type.
    """
    ...

class ConnectionError(MarketDataError):
    """Network connection failed.

    Raised when unable to connect to the Fugle API servers.
    May be due to network issues, DNS resolution, or server unavailability.
    """
    ...

class TimeoutError(MarketDataError):
    """Operation timed out.

    Raised when an API call or WebSocket operation exceeds the timeout.
    """
    ...

class WebSocketError(MarketDataError):
    """WebSocket protocol error.

    Raised for WebSocket-specific errors like connection drops,
    protocol violations, or message parsing failures.
    """
    ...

# REST Client
class RestClient:
    """REST client for Fugle market data API.

    Provides access to stock and futures/options market data through
    REST endpoints. All data methods are async and return coroutines.

    Example:
        ```python
        import asyncio
        from marketdata_py import RestClient

        async def main():
            client = RestClient("your-api-key")
            quote = await client.stock.intraday.quote("2330")
            print(f"Last price: {quote['lastPrice']}")

        asyncio.run(main())
        ```
    """

    def __init__(self, api_key: str) -> None:
        """Create a new REST client with API key authentication.

        Args:
            api_key: Your Fugle API key
        """
        ...

    @staticmethod
    def with_bearer_token(token: str) -> "RestClient":
        """Create a REST client with bearer token authentication.

        Args:
            token: Bearer token for authentication

        Returns:
            A new RestClient instance
        """
        ...

    @staticmethod
    def with_sdk_token(sdk_token: str) -> "RestClient":
        """Create a REST client with SDK token authentication.

        Args:
            sdk_token: SDK token for authentication

        Returns:
            A new RestClient instance
        """
        ...

    @property
    def stock(self) -> "StockClient":
        """Access stock market data endpoints.

        Returns:
            StockClient for accessing stock endpoints
        """
        ...

    @property
    def futopt(self) -> "FutOptClient":
        """Access futures and options market data endpoints.

        Returns:
            FutOptClient for accessing FutOpt endpoints
        """
        ...


class StockClient:
    """Stock market data client.

    Access via `client.stock`. Provides access to intraday, historical,
    and snapshot stock data.
    """

    @property
    def intraday(self) -> "StockIntradayClient":
        """Access intraday (real-time) stock endpoints.

        Returns:
            StockIntradayClient for accessing intraday endpoints
        """
        ...


class StockIntradayClient:
    """Stock intraday (real-time) endpoints client.

    Access via `client.stock.intraday`. All methods are async and
    return coroutines that resolve to dict objects.
    """

    async def quote(self, symbol: str, *, odd_lot: bool = False) -> dict[str, Any]:
        """Get intraday quote for a stock symbol.

        Args:
            symbol: Stock symbol (e.g., "2330" for TSMC)
            odd_lot: Whether to query odd lot data (default: False)

        Returns:
            Quote data including prices, order book, and trading info

        Raises:
            MarketDataError: If the request fails

        Example:
            ```python
            quote = await client.stock.intraday.quote("2330")
            print(f"Last price: {quote['lastPrice']}")
            print(f"Change: {quote['change']}")
            ```
        """
        ...

    async def ticker(self, symbol: str) -> dict[str, Any]:
        """Get ticker information for a stock symbol.

        Args:
            symbol: Stock symbol (e.g., "2330" for TSMC)

        Returns:
            Ticker data including name, industry, and basic info

        Raises:
            MarketDataError: If the request fails
        """
        ...

    async def candles(self, symbol: str, *, timeframe: str = "1") -> dict[str, Any]:
        """Get candlestick chart data.

        Args:
            symbol: Stock symbol (e.g., "2330" for TSMC)
            timeframe: Timeframe in minutes (default: "1")

        Returns:
            Candlestick data with OHLCV values

        Raises:
            MarketDataError: If the request fails
        """
        ...

    async def trades(self, symbol: str) -> dict[str, Any]:
        """Get trade ticks data.

        Args:
            symbol: Stock symbol (e.g., "2330" for TSMC)

        Returns:
            Trade ticks data with price, volume, and time

        Raises:
            MarketDataError: If the request fails
        """
        ...

    async def volumes(self, symbol: str) -> dict[str, Any]:
        """Get volume data.

        Args:
            symbol: Stock symbol (e.g., "2330" for TSMC)

        Returns:
            Volume data by price level

        Raises:
            MarketDataError: If the request fails
        """
        ...


class FutOptClient:
    """Futures and options market data client.

    Access via `client.futopt`. Provides access to intraday
    futures and options data.
    """

    @property
    def intraday(self) -> "FutOptIntradayClient":
        """Access intraday (real-time) FutOpt endpoints.

        Returns:
            FutOptIntradayClient for accessing intraday endpoints
        """
        ...


class FutOptIntradayClient:
    """FutOpt intraday (real-time) endpoints client.

    Access via `client.futopt.intraday`. All methods are async and
    return coroutines that resolve to dict objects.
    """

    async def quote(self, symbol: str, *, after_hours: bool = False) -> dict[str, Any]:
        """Get intraday quote for a futures/options contract.

        Args:
            symbol: Contract symbol (e.g., "TXFC4" for TAIEX futures)
            after_hours: Whether to query after-hours session data (default: False)

        Returns:
            Quote data including prices, order book, and trading info

        Raises:
            MarketDataError: If the request fails

        Example:
            ```python
            # Regular session
            quote = await client.futopt.intraday.quote("TXFC4")

            # After-hours session
            ah_quote = await client.futopt.intraday.quote("TXFC4", after_hours=True)
            ```
        """
        ...


# WebSocket Client
class ReconnectConfig:
    """Auto-reconnect configuration.

    Controls automatic reconnection behavior when WebSocket connection is lost.
    Uses exponential backoff with configurable parameters.

    Example:
        ```python
        from marketdata_py import ReconnectConfig, WebSocketClient

        config = ReconnectConfig(
            enabled=True,
            max_retries=5,
            base_delay_ms=1000,
            max_delay_ms=30000
        )
        ws = WebSocketClient("your-api-key")
        ```
    """

    enabled: bool
    """Whether auto-reconnect is enabled."""

    max_retries: int
    """Maximum number of reconnection attempts (0 = unlimited)."""

    base_delay_ms: int
    """Base delay in milliseconds for exponential backoff."""

    max_delay_ms: int
    """Maximum delay in milliseconds (caps exponential backoff)."""

    def __init__(
        self,
        enabled: bool = True,
        max_retries: int = 5,
        base_delay_ms: int = 1000,
        max_delay_ms: int = 30000,
    ) -> None:
        """Create a new reconnect configuration.

        Args:
            enabled: Whether auto-reconnect is enabled (default: True)
            max_retries: Maximum reconnection attempts, 0 for unlimited (default: 5)
            base_delay_ms: Base delay for exponential backoff (default: 1000ms)
            max_delay_ms: Maximum delay cap (default: 30000ms = 30s)
        """
        ...

    @staticmethod
    def default_config() -> "ReconnectConfig":
        """Create a default reconnect configuration (enabled with 5 retries)."""
        ...

    @staticmethod
    def disabled() -> "ReconnectConfig":
        """Create a disabled reconnect configuration."""
        ...


class WebSocketClient:
    """WebSocket client for Fugle market data streaming.

    Provides real-time streaming access to stock and futures/options
    market data through WebSocket connections.

    Example:
        ```python
        from marketdata_py import WebSocketClient

        ws = WebSocketClient("your-api-key")

        # Callback mode
        def on_message(msg):
            print(f"Received: {msg}")

        ws.stock.on("message", on_message)
        ws.stock.connect()
        ws.stock.subscribe("trades", "2330")

        # Or async iterator mode
        async with ws.stock as client:
            await client.subscribe("trades", "2330")
            async for msg in client.messages():
                print(msg)
        ```
    """

    def __init__(self, api_key: str) -> None:
        """Create a new WebSocket client with API key authentication.

        Args:
            api_key: Your Fugle API key
        """
        ...

    @property
    def stock(self) -> "StockWebSocketClient":
        """Access stock market data WebSocket streaming.

        Returns:
            StockWebSocketClient for stock streaming
        """
        ...

    @property
    def futopt(self) -> "FutOptWebSocketClient":
        """Access futures and options WebSocket streaming.

        Returns:
            FutOptWebSocketClient for FutOpt streaming
        """
        ...


class StockWebSocketClient:
    """Stock market WebSocket client.

    Access via `ws.stock`. Supports both callback-based and async iterator-based
    message consumption. Can be used as an async context manager.

    Example (callback mode):
        ```python
        def on_message(msg):
            print(msg)

        ws.stock.on("message", on_message)
        ws.stock.connect()
        ws.stock.subscribe("trades", "2330")
        ```

    Example (async iterator mode):
        ```python
        async with ws.stock as client:
            await client.subscribe("trades", "2330")
            async for msg in client.messages():
                print(msg)
        ```
    """

    def connect(self) -> None:
        """Connect to WebSocket server (blocking).

        If message callbacks are registered before connect(), a background
        thread will automatically dispatch incoming messages to the callbacks.

        Raises:
            MarketDataError: If connection fails
        """
        ...

    async def connect_async(self) -> None:
        """Connect to WebSocket server (async).

        Returns an awaitable that completes when connection is established.
        Releases GIL during connection, enabling concurrent Python tasks.

        Raises:
            MarketDataError: If connection fails
        """
        ...

    def disconnect(self) -> None:
        """Disconnect from WebSocket server (blocking)."""
        ...

    async def disconnect_async(self) -> None:
        """Disconnect from WebSocket server (async).

        Returns an awaitable that completes when disconnection finishes.
        """
        ...

    def is_connected(self) -> bool:
        """Check if currently connected.

        Returns:
            True if connected, False otherwise
        """
        ...

    def is_closed(self) -> bool:
        """Check if client has been closed.

        Returns true if disconnect() has been called and client is closed.
        Once closed, the client cannot be reused - create a new instance.

        Returns:
            True if closed, False otherwise
        """
        ...

    def subscribe(self, channel: str, symbol: str, *, odd_lot: bool = False) -> None:
        """Subscribe to a channel for a symbol (blocking).

        Args:
            channel: Channel name (trades, candles, books, aggregates, indices)
            symbol: Stock symbol (e.g., "2330")
            odd_lot: Whether to subscribe to odd lot data (default: False)

        Raises:
            RuntimeError: If not connected
            ValueError: If channel is invalid
        """
        ...

    async def subscribe_async(self, channel: str, symbol: str, *, odd_lot: bool = False) -> None:
        """Subscribe to a channel for a symbol (async).

        Args:
            channel: Channel name (trades, candles, books, aggregates, indices)
            symbol: Stock symbol (e.g., "2330")
            odd_lot: Whether to subscribe to odd lot data (default: False)

        Raises:
            RuntimeError: If not connected
            ValueError: If channel is invalid
        """
        ...

    def unsubscribe(self, subscription_id: str) -> None:
        """Unsubscribe from a channel by subscription ID.

        Args:
            subscription_id: The subscription ID returned from subscribe
        """
        ...

    def subscriptions(self) -> List[str]:
        """Get list of active subscription keys.

        Returns:
            List of active subscription keys
        """
        ...

    def on(self, event: str, callback: Callable[..., None]) -> None:
        """Register a callback for an event type.

        Supported events:
          - "message" / "data": Called with message dict when data received
          - "connect" / "connected": Called when connection established
          - "disconnect" / "disconnected" / "close": Called when connection closed
          - "reconnect" / "reconnecting": Called when reconnecting
          - "error": Called with (message, code) when error occurs

        Args:
            event: Event type string
            callback: Python callable to invoke
        """
        ...

    def off(self, event: str) -> None:
        """Remove all callbacks for an event type.

        Args:
            event: Event type string
        """
        ...

    def messages(self, *, timeout_ms: Optional[int] = None) -> "MessageIterator":
        """Get message iterator for consuming streaming data.

        Args:
            timeout_ms: Optional timeout in milliseconds for blocking receive

        Returns:
            MessageIterator for iterating over messages

        Note:
            The iterator blocks waiting for messages. Use timeout parameter
            to control blocking behavior.
        """
        ...

    async def __aenter__(self) -> "StockWebSocketClient":
        """Async context manager entry - connects to WebSocket server."""
        ...

    async def __aexit__(
        self,
        exc_type: Any,
        exc_val: Any,
        exc_tb: Any,
    ) -> None:
        """Async context manager exit - disconnects from WebSocket server."""
        ...


class FutOptWebSocketClient:
    """FutOpt (futures and options) WebSocket client.

    Access via `ws.futopt`. Similar to StockWebSocketClient but for
    futures and options market data.
    """

    def connect(self) -> None:
        """Connect to WebSocket server (blocking).

        Raises:
            MarketDataError: If connection fails
        """
        ...

    def disconnect(self) -> None:
        """Disconnect from WebSocket server (blocking)."""
        ...

    def is_connected(self) -> bool:
        """Check if currently connected.

        Returns:
            True if connected, False otherwise
        """
        ...

    def is_closed(self) -> bool:
        """Check if client has been closed.

        Returns:
            True if closed, False otherwise
        """
        ...

    def subscribe(self, channel: str, symbol: str, *, after_hours: bool = False) -> None:
        """Subscribe to a channel for a FutOpt symbol (blocking).

        Args:
            channel: Channel name (trades, candles, books, aggregates)
            symbol: FutOpt contract symbol (e.g., "TXFC4", "TXF202502")
            after_hours: Whether to subscribe to after-hours session (default: False)

        Raises:
            RuntimeError: If not connected
            ValueError: If channel is invalid
        """
        ...

    def unsubscribe(self, subscription_id: str) -> None:
        """Unsubscribe from a channel by subscription ID.

        Args:
            subscription_id: The subscription ID returned from subscribe
        """
        ...

    def subscriptions(self) -> List[str]:
        """Get list of active subscription keys.

        Returns:
            List of active subscription keys
        """
        ...

    def on(self, event: str, callback: Callable[..., None]) -> None:
        """Register a callback for an event type.

        Args:
            event: Event type string
            callback: Python callable to invoke
        """
        ...

    def off(self, event: str) -> None:
        """Remove all callbacks for an event type.

        Args:
            event: Event type string
        """
        ...

    def messages(self, *, timeout_ms: Optional[int] = None) -> "MessageIterator":
        """Get message iterator for consuming streaming data.

        Args:
            timeout_ms: Optional timeout in milliseconds for blocking receive

        Returns:
            MessageIterator for iterating over messages
        """
        ...


class MessageIterator:
    """Iterator for WebSocket messages.

    Supports both synchronous iteration (`for msg in iter`) and
    asynchronous iteration (`async for msg in iter`).

    Example (sync):
        ```python
        for msg in ws.stock.messages():
            print(msg)
        ```

    Example (async):
        ```python
        async for msg in ws.stock.messages():
            print(msg)
        ```
    """

    def __iter__(self) -> "MessageIterator":
        """Return self for iteration."""
        ...

    def __next__(self) -> Optional[dict[str, Any]]:
        """Get next message (blocking).

        Returns:
            Message dict or None on timeout

        Raises:
            StopIteration: When the channel is closed
        """
        ...

    def __aiter__(self) -> "MessageIterator":
        """Return self for async iteration."""
        ...

    async def __anext__(self) -> dict[str, Any]:
        """Get next message (async).

        Returns:
            Message dict

        Raises:
            StopAsyncIteration: When the channel is closed
        """
        ...

    def try_recv(self) -> Optional[dict[str, Any]]:
        """Try to receive a message without blocking.

        Returns:
            Message dict if available, None otherwise
        """
        ...

    async def recv_timeout(self, timeout_ms: int) -> Optional[dict[str, Any]]:
        """Receive a message with timeout (async).

        Args:
            timeout_ms: Timeout in milliseconds

        Returns:
            Message dict if received within timeout, None on timeout
        """
        ...
