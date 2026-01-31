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
    snapshot, technical, and corporate actions stock data.
    """

    @property
    def intraday(self) -> "StockIntradayClient":
        """Access intraday (real-time) stock endpoints.

        Returns:
            StockIntradayClient for accessing intraday endpoints
        """
        ...

    @property
    def historical(self) -> "StockHistoricalClient":
        """Access historical stock data endpoints.

        Returns:
            StockHistoricalClient for accessing historical endpoints
        """
        ...

    @property
    def snapshot(self) -> "StockSnapshotClient":
        """Access snapshot endpoints for market-wide data.

        Returns:
            StockSnapshotClient for accessing snapshot endpoints
        """
        ...

    @property
    def technical(self) -> "StockTechnicalClient":
        """Access technical indicator endpoints.

        Returns:
            StockTechnicalClient for accessing technical endpoints
        """
        ...

    @property
    def corporate_actions(self) -> "StockCorporateActionsClient":
        """Access corporate actions endpoints.

        Returns:
            StockCorporateActionsClient for accessing corporate actions endpoints
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


class StockHistoricalClient:
    """Stock historical data endpoints client.

    Access via `client.stock.historical`. All methods are async and
    return coroutines that resolve to dict objects.
    """

    async def candles(
        self,
        symbol: str,
        *,
        from_date: Optional[str] = None,
        to_date: Optional[str] = None,
        timeframe: Optional[str] = None,
        fields: Optional[str] = None,
        sort: Optional[str] = None,
        adjusted: Optional[bool] = None,
    ) -> dict[str, Any]:
        """Get historical candles for a stock symbol.

        Args:
            symbol: Stock symbol (e.g., "2330" for TSMC)
            from_date: Start date (YYYY-MM-DD)
            to_date: End date (YYYY-MM-DD)
            timeframe: Timeframe ("D", "W", "M", "1", "5", "10", "15", "30", "60")
            fields: Optional field selection
            sort: Sort order ("asc" or "desc")
            adjusted: Whether to adjust for splits/dividends

        Returns:
            Historical candles data

        Raises:
            MarketDataError: If the request fails

        Example:
            ```python
            candles = await client.stock.historical.candles(
                "2330",
                from_date="2024-01-01",
                to_date="2024-01-31",
                timeframe="D"
            )
            ```
        """
        ...

    async def stats(self, symbol: str) -> dict[str, Any]:
        """Get historical stats for a stock symbol.

        Args:
            symbol: Stock symbol (e.g., "2330" for TSMC)

        Returns:
            Historical stats data including 52-week high/low

        Raises:
            MarketDataError: If the request fails

        Example:
            ```python
            stats = await client.stock.historical.stats("2330")
            print(f"52-week high: {stats['week52High']}")
            ```
        """
        ...


class StockSnapshotClient:
    """Stock snapshot endpoints client.

    Access via `client.stock.snapshot`. All methods are async and
    return coroutines that resolve to dict objects.
    """

    async def quotes(
        self,
        market: str,
        *,
        type_filter: Optional[str] = None,
    ) -> dict[str, Any]:
        """Get snapshot quotes for a market.

        Args:
            market: Market code ("TSE", "OTC", "ESB", "TIB", "PSB")
            type_filter: Type filter ("ALL", "ALLBUT0999", "COMMONSTOCK")

        Returns:
            Market-wide quotes snapshot

        Raises:
            MarketDataError: If the request fails

        Example:
            ```python
            quotes = await client.stock.snapshot.quotes("TSE", type_filter="COMMONSTOCK")
            ```
        """
        ...

    async def movers(
        self,
        market: str,
        *,
        direction: Optional[str] = None,
        change: Optional[str] = None,
    ) -> dict[str, Any]:
        """Get top movers for a market.

        Args:
            market: Market code ("TSE", "OTC", "ESB", "TIB", "PSB")
            direction: Direction filter ("up" for gainers, "down" for losers)
            change: Change type ("percent" or "value")

        Returns:
            Top movers data

        Raises:
            MarketDataError: If the request fails

        Example:
            ```python
            movers = await client.stock.snapshot.movers("TSE", direction="up", change="percent")
            ```
        """
        ...

    async def actives(
        self,
        market: str,
        *,
        trade: Optional[str] = None,
    ) -> dict[str, Any]:
        """Get most active stocks for a market.

        Args:
            market: Market code ("TSE", "OTC", "ESB", "TIB", "PSB")
            trade: Trade type ("volume" or "value")

        Returns:
            Most active stocks data

        Raises:
            MarketDataError: If the request fails

        Example:
            ```python
            actives = await client.stock.snapshot.actives("TSE", trade="volume")
            ```
        """
        ...


class StockTechnicalClient:
    """Stock technical indicator endpoints client.

    Access via `client.stock.technical`. All methods are async and
    return coroutines that resolve to dict objects.
    """

    async def sma(
        self,
        symbol: str,
        *,
        from_date: Optional[str] = None,
        to_date: Optional[str] = None,
        timeframe: Optional[str] = None,
        period: Optional[int] = None,
    ) -> dict[str, Any]:
        """Get Simple Moving Average (SMA) data.

        Args:
            symbol: Stock symbol (e.g., "2330" for TSMC)
            from_date: Start date (YYYY-MM-DD)
            to_date: End date (YYYY-MM-DD)
            timeframe: Timeframe ("D", "W", "M", "1", "5", etc.)
            period: Moving average period

        Returns:
            SMA indicator data

        Raises:
            MarketDataError: If the request fails
        """
        ...

    async def rsi(
        self,
        symbol: str,
        *,
        from_date: Optional[str] = None,
        to_date: Optional[str] = None,
        timeframe: Optional[str] = None,
        period: Optional[int] = None,
    ) -> dict[str, Any]:
        """Get Relative Strength Index (RSI) data.

        Args:
            symbol: Stock symbol (e.g., "2330" for TSMC)
            from_date: Start date (YYYY-MM-DD)
            to_date: End date (YYYY-MM-DD)
            timeframe: Timeframe ("D", "W", "M", "1", "5", etc.)
            period: RSI period (default 14)

        Returns:
            RSI indicator data

        Raises:
            MarketDataError: If the request fails
        """
        ...

    async def kdj(
        self,
        symbol: str,
        *,
        from_date: Optional[str] = None,
        to_date: Optional[str] = None,
        timeframe: Optional[str] = None,
        period: Optional[int] = None,
    ) -> dict[str, Any]:
        """Get KDJ (Stochastic Oscillator) data.

        Args:
            symbol: Stock symbol (e.g., "2330" for TSMC)
            from_date: Start date (YYYY-MM-DD)
            to_date: End date (YYYY-MM-DD)
            timeframe: Timeframe ("D", "W", "M", "1", "5", etc.)
            period: KDJ period

        Returns:
            KDJ indicator data with K, D, J values

        Raises:
            MarketDataError: If the request fails
        """
        ...

    async def macd(
        self,
        symbol: str,
        *,
        from_date: Optional[str] = None,
        to_date: Optional[str] = None,
        timeframe: Optional[str] = None,
        fast: Optional[int] = None,
        slow: Optional[int] = None,
        signal: Optional[int] = None,
    ) -> dict[str, Any]:
        """Get MACD (Moving Average Convergence Divergence) data.

        Args:
            symbol: Stock symbol (e.g., "2330" for TSMC)
            from_date: Start date (YYYY-MM-DD)
            to_date: End date (YYYY-MM-DD)
            timeframe: Timeframe ("D", "W", "M", "1", "5", etc.)
            fast: Fast EMA period (default 12)
            slow: Slow EMA period (default 26)
            signal: Signal line period (default 9)

        Returns:
            MACD indicator data with MACD, signal, histogram

        Raises:
            MarketDataError: If the request fails
        """
        ...

    async def bb(
        self,
        symbol: str,
        *,
        from_date: Optional[str] = None,
        to_date: Optional[str] = None,
        timeframe: Optional[str] = None,
        period: Optional[int] = None,
        stddev: Optional[float] = None,
    ) -> dict[str, Any]:
        """Get Bollinger Bands (BB) data.

        Args:
            symbol: Stock symbol (e.g., "2330" for TSMC)
            from_date: Start date (YYYY-MM-DD)
            to_date: End date (YYYY-MM-DD)
            timeframe: Timeframe ("D", "W", "M", "1", "5", etc.)
            period: Moving average period (default 20)
            stddev: Standard deviation multiplier (default 2.0)

        Returns:
            Bollinger Bands data with upper, middle, lower bands

        Raises:
            MarketDataError: If the request fails
        """
        ...


class StockCorporateActionsClient:
    """Stock corporate actions endpoints client.

    Access via `client.stock.corporate_actions`. All methods are async and
    return coroutines that resolve to dict objects.
    """

    async def capital_changes(
        self,
        *,
        date: Optional[str] = None,
        start_date: Optional[str] = None,
        end_date: Optional[str] = None,
    ) -> dict[str, Any]:
        """Get capital changes (stock splits, rights issues, etc.)

        Args:
            date: Specific date (YYYY-MM-DD)
            start_date: Start date for range query (YYYY-MM-DD)
            end_date: End date for range query (YYYY-MM-DD)

        Returns:
            Capital changes data

        Raises:
            MarketDataError: If the request fails

        Example:
            ```python
            changes = await client.stock.corporate_actions.capital_changes(
                start_date="2024-01-01",
                end_date="2024-01-31"
            )
            ```
        """
        ...

    async def dividends(
        self,
        *,
        date: Optional[str] = None,
        start_date: Optional[str] = None,
        end_date: Optional[str] = None,
    ) -> dict[str, Any]:
        """Get dividend announcements.

        Args:
            date: Specific date (YYYY-MM-DD)
            start_date: Start date for range query (YYYY-MM-DD)
            end_date: End date for range query (YYYY-MM-DD)

        Returns:
            Dividend data

        Raises:
            MarketDataError: If the request fails

        Example:
            ```python
            dividends = await client.stock.corporate_actions.dividends(
                start_date="2024-01-01",
                end_date="2024-12-31"
            )
            ```
        """
        ...

    async def listing_applicants(
        self,
        *,
        date: Optional[str] = None,
        start_date: Optional[str] = None,
        end_date: Optional[str] = None,
    ) -> dict[str, Any]:
        """Get IPO listing applicants.

        Args:
            date: Specific date (YYYY-MM-DD)
            start_date: Start date for range query (YYYY-MM-DD)
            end_date: End date for range query (YYYY-MM-DD)

        Returns:
            Listing applicants data

        Raises:
            MarketDataError: If the request fails

        Example:
            ```python
            applicants = await client.stock.corporate_actions.listing_applicants()
            ```
        """
        ...


class FutOptClient:
    """Futures and options market data client.

    Access via `client.futopt`. Provides access to intraday and historical
    futures and options data.
    """

    @property
    def intraday(self) -> "FutOptIntradayClient":
        """Access intraday (real-time) FutOpt endpoints.

        Returns:
            FutOptIntradayClient for accessing intraday endpoints
        """
        ...

    @property
    def historical(self) -> "FutOptHistoricalClient":
        """Access historical FutOpt data endpoints.

        Returns:
            FutOptHistoricalClient for accessing historical endpoints
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


class FutOptHistoricalClient:
    """FutOpt historical data endpoints client.

    Access via `client.futopt.historical`. All methods are async and
    return coroutines that resolve to dict objects.
    """

    async def candles(
        self,
        symbol: str,
        *,
        from_date: Optional[str] = None,
        to_date: Optional[str] = None,
        timeframe: Optional[str] = None,
        after_hours: bool = False,
    ) -> dict[str, Any]:
        """Get historical candles for a FutOpt contract.

        Args:
            symbol: Contract symbol (e.g., "TXFC4" for TAIEX futures)
            from_date: Start date (YYYY-MM-DD)
            to_date: End date (YYYY-MM-DD)
            timeframe: Timeframe ("D", "W", "M", "1", "5", "10", "15", "30", "60")
            after_hours: Whether to include after-hours session data (default: False)

        Returns:
            Historical candles data

        Raises:
            MarketDataError: If the request fails

        Example:
            ```python
            candles = await client.futopt.historical.candles(
                "TXFC4",
                from_date="2024-01-01",
                to_date="2024-01-31",
                timeframe="D"
            )
            ```
        """
        ...

    async def daily(
        self,
        symbol: str,
        *,
        from_date: Optional[str] = None,
        to_date: Optional[str] = None,
        after_hours: bool = False,
    ) -> dict[str, Any]:
        """Get daily historical data for a FutOpt contract.

        Args:
            symbol: Contract symbol (e.g., "TXFC4" for TAIEX futures)
            from_date: Start date (YYYY-MM-DD)
            to_date: End date (YYYY-MM-DD)
            after_hours: Whether to include after-hours session data (default: False)

        Returns:
            Daily historical data with settlement prices

        Raises:
            MarketDataError: If the request fails

        Example:
            ```python
            daily = await client.futopt.historical.daily(
                "TXFC4",
                from_date="2024-01-01",
                to_date="2024-01-31"
            )
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
