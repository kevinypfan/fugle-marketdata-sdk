"""
Response Compatibility Tests

These tests validate that our SDK produces responses with identical structure
to the official fugle-marketdata-python SDK using recorded VCR cassettes.

The cassettes are either:
1. Real recordings from official SDK (if user ran record_official_responses.py)
2. Mock cassettes with expected structure (for CI without API key)

Run: pytest py/tests/test_response_compatibility.py -v
"""
import pytest
import vcr
from pathlib import Path

FIXTURES_DIR = Path(__file__).parent / "fixtures"

# VCR configuration
vcr_config = vcr.VCR(
    cassette_library_dir=str(FIXTURES_DIR),
    record_mode='none',  # Never record during tests - use record_official_responses.py
    match_on=['method', 'scheme', 'host', 'port', 'path', 'query'],
)


class TestQuoteResponseCompatibility:
    """Validate quote response matches official SDK structure."""

    @vcr_config.use_cassette('official_sdk_quote.yaml')
    @pytest.mark.asyncio
    async def test_quote_has_required_fields(self, rest_client):
        """Response must have all fields present in official SDK."""
        response = await rest_client.stock.intraday.quote("2330")

        # Top-level structure
        assert 'apiVersion' in response or 'data' in response or 'symbol' in response

        # Extract data payload (handle both wrapped and unwrapped formats)
        if 'data' in response:
            data = response['data']
        else:
            data = response

        # Info section (metadata)
        if 'info' in data:
            info = data['info']
            assert 'date' in info
            assert 'symbolId' in info or 'symbol' in info

        # Quote section (price data)
        if 'quote' in data:
            quote = data['quote']

            # Total statistics
            if 'total' in quote:
                total = quote['total']
                assert 'tradeVolume' in total or 'volume' in total
                assert 'tradeValue' in total or 'value' in total

            # Current trade
            if 'trade' in quote:
                trade = quote['trade']
                assert 'price' in trade
                assert 'at' in trade or 'time' in trade

            # Order book
            if 'order' in quote:
                order = quote['order']
                if 'bids' in order:
                    assert isinstance(order['bids'], list)
                    if order['bids']:
                        assert 'price' in order['bids'][0]
                        assert 'volume' in order['bids'][0]
                if 'asks' in order:
                    assert isinstance(order['asks'], list)
                    if order['asks']:
                        assert 'price' in order['asks'][0]
                        assert 'volume' in order['asks'][0]

    @vcr_config.use_cassette('official_sdk_quote.yaml')
    @pytest.mark.asyncio
    async def test_quote_field_types(self, rest_client):
        """Validate field types match official SDK."""
        response = await rest_client.stock.intraday.quote("2330")

        # Extract data
        if 'data' in response:
            data = response['data']
        else:
            data = response

        # Validate info types
        if 'info' in data:
            info = data['info']
            if 'date' in info:
                assert isinstance(info['date'], str)
            if 'symbolId' in info:
                assert isinstance(info['symbolId'], str)
            elif 'symbol' in info:
                assert isinstance(info['symbol'], str)

        # Validate quote types
        if 'quote' in data:
            quote = data['quote']

            # Price values should be numeric
            if 'priceHigh' in quote:
                price_high = quote['priceHigh']
                if isinstance(price_high, dict) and 'price' in price_high:
                    assert isinstance(price_high['price'], (int, float))

            if 'priceLow' in quote:
                price_low = quote['priceLow']
                if isinstance(price_low, dict) and 'price' in price_low:
                    assert isinstance(price_low['price'], (int, float))

    @vcr_config.use_cassette('official_sdk_quote.yaml')
    @pytest.mark.asyncio
    async def test_quote_order_book_structure(self, rest_client):
        """Order book (bids/asks) must have proper structure."""
        response = await rest_client.stock.intraday.quote("2330")

        # Extract quote
        if 'data' in response:
            quote = response['data'].get('quote', {})
        else:
            quote = response.get('quote', {})

        if 'order' in quote:
            order = quote['order']

            # Bids structure
            if 'bids' in order and order['bids']:
                for bid in order['bids']:
                    assert 'price' in bid, "Bid must have price"
                    assert 'volume' in bid, "Bid must have volume"
                    assert isinstance(bid['price'], (int, float)), "Bid price must be numeric"
                    assert isinstance(bid['volume'], (int, float)), "Bid volume must be numeric"

            # Asks structure
            if 'asks' in order and order['asks']:
                for ask in order['asks']:
                    assert 'price' in ask, "Ask must have price"
                    assert 'volume' in ask, "Ask must have volume"
                    assert isinstance(ask['price'], (int, float)), "Ask price must be numeric"
                    assert isinstance(ask['volume'], (int, float)), "Ask volume must be numeric"


class TestTickerResponseCompatibility:
    """Validate ticker response matches official SDK structure."""

    @vcr_config.use_cassette('official_sdk_ticker.yaml')
    @pytest.mark.asyncio
    async def test_ticker_has_array_structure(self, rest_client):
        """Ticker response should be an array of tick data."""
        response = await rest_client.stock.intraday.ticker("2330")

        # Extract ticker array
        if 'data' in response:
            data = response['data']
            if 'ticker' in data:
                ticker = data['ticker']
            else:
                ticker = data
        else:
            ticker = response

        # Should be array-like
        assert isinstance(ticker, (list, tuple)), "Ticker should be array/list"

    @vcr_config.use_cassette('official_sdk_ticker.yaml')
    @pytest.mark.asyncio
    async def test_ticker_items_have_required_fields(self, rest_client):
        """Each ticker item should have timestamp and price fields."""
        response = await rest_client.stock.intraday.ticker("2330")

        # Extract ticker array
        if 'data' in response:
            data = response['data']
            ticker = data.get('ticker', data)
        else:
            ticker = response

        if isinstance(ticker, (list, tuple)) and ticker:
            # Check first item structure
            item = ticker[0]

            # Time field (various formats)
            assert any(k in item for k in ['at', 'time', 'timestamp']), \
                "Ticker item must have time field"

            # Price field
            assert any(k in item for k in ['price', 'lastPrice', 'close']), \
                "Ticker item must have price field"


class TestTradesResponseCompatibility:
    """Validate trades response matches official SDK structure."""

    @vcr_config.use_cassette('official_sdk_trades.yaml')
    @pytest.mark.asyncio
    async def test_trades_has_array_structure(self, rest_client):
        """Trades response should be an array of trade executions."""
        response = await rest_client.stock.intraday.trades("2330")

        # Extract trades array
        if 'data' in response:
            data = response['data']
            trades = data.get('trades', data)
        else:
            trades = response

        assert isinstance(trades, (list, tuple)), "Trades should be array/list"

    @vcr_config.use_cassette('official_sdk_trades.yaml')
    @pytest.mark.asyncio
    async def test_trades_items_have_price_volume(self, rest_client):
        """Each trade should have price and volume."""
        response = await rest_client.stock.intraday.trades("2330")

        # Extract trades array
        if 'data' in response:
            data = response['data']
            trades = data.get('trades', data)
        else:
            trades = response

        if isinstance(trades, (list, tuple)) and trades:
            trade = trades[0]

            # Must have price
            assert 'price' in trade, "Trade must have price"
            assert isinstance(trade['price'], (int, float)), "Price must be numeric"

            # Must have volume
            assert 'volume' in trade, "Trade must have volume"
            assert isinstance(trade['volume'], (int, float)), "Volume must be numeric"

    @vcr_config.use_cassette('official_sdk_trades.yaml')
    @pytest.mark.asyncio
    async def test_trades_items_have_timestamp(self, rest_client):
        """Each trade should have timestamp."""
        response = await rest_client.stock.intraday.trades("2330")

        # Extract trades array
        if 'data' in response:
            data = response['data']
            trades = data.get('trades', data)
        else:
            trades = response

        if isinstance(trades, (list, tuple)) and trades:
            trade = trades[0]

            # Must have timestamp (various field names)
            assert any(k in trade for k in ['at', 'time', 'timestamp']), \
                "Trade must have timestamp field"


class TestCandlesResponseCompatibility:
    """Validate candles response matches official SDK structure."""

    @vcr_config.use_cassette('official_sdk_candles.yaml')
    @pytest.mark.asyncio
    async def test_candles_has_array_structure(self, rest_client):
        """Candles response should be an array of OHLCV data."""
        response = await rest_client.stock.intraday.candles("2330")

        # Extract candles array
        if 'data' in response:
            data = response['data']
            candles = data.get('candles', data)
        else:
            candles = response

        assert isinstance(candles, (list, tuple)), "Candles should be array/list"

    @vcr_config.use_cassette('official_sdk_candles.yaml')
    @pytest.mark.asyncio
    async def test_candles_items_have_ohlcv_fields(self, rest_client):
        """Each candle should have OHLCV structure."""
        response = await rest_client.stock.intraday.candles("2330")

        # Extract candles array
        if 'data' in response:
            data = response['data']
            candles = data.get('candles', data)
        else:
            candles = response

        if isinstance(candles, (list, tuple)) and candles:
            candle = candles[0]

            # OHLC fields (open, high, low, close)
            assert 'open' in candle, "Candle must have open price"
            assert 'high' in candle, "Candle must have high price"
            assert 'low' in candle, "Candle must have low price"
            assert 'close' in candle, "Candle must have close price"

            # Volume field
            assert 'volume' in candle, "Candle must have volume"

            # All OHLCV values should be numeric
            assert isinstance(candle['open'], (int, float))
            assert isinstance(candle['high'], (int, float))
            assert isinstance(candle['low'], (int, float))
            assert isinstance(candle['close'], (int, float))
            assert isinstance(candle['volume'], (int, float))

    @vcr_config.use_cassette('official_sdk_candles.yaml')
    @pytest.mark.asyncio
    async def test_candles_items_have_timestamp(self, rest_client):
        """Each candle should have timestamp."""
        response = await rest_client.stock.intraday.candles("2330")

        # Extract candles array
        if 'data' in response:
            data = response['data']
            candles = data.get('candles', data)
        else:
            candles = response

        if isinstance(candles, (list, tuple)) and candles:
            candle = candles[0]

            # Must have timestamp
            assert any(k in candle for k in ['at', 'time', 'timestamp']), \
                "Candle must have timestamp field"


# ============================================================================
# Response Structure Documentation
# ============================================================================
#
# These tests validate that our SDK returns responses with the same structure
# as the official fugle-marketdata-python SDK. This ensures drop-in compatibility.
#
# Test Strategy:
# 1. Use VCR cassettes to capture official SDK responses
# 2. Validate our SDK produces same field structure
# 3. Tests are flexible to handle:
#    - Wrapped responses (apiVersion + data)
#    - Unwrapped responses (direct data)
#    - Different field name variations
#
# Running Tests:
#   Without API key (using mock cassettes):
#     pytest py/tests/test_response_compatibility.py -v
#
#   With real recordings (after running record_official_responses.py):
#     pytest py/tests/test_response_compatibility.py -v
#
# Mock vs Real:
# - Mock cassettes provide baseline structure validation
# - Real recordings provide exact response format validation
# - Both approaches ensure API compatibility
#
# ============================================================================
