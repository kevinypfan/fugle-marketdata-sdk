#!/usr/bin/env python3
"""
VCR Recording Script for Official Fugle SDK Responses

This script records API responses from the official fugle-marketdata-python SDK
to VCR cassettes for fixture-based compatibility testing.

Usage:
    1. Install official SDK: pip install fugle-marketdata
    2. Set your API key: export FUGLE_API_KEY="your-key-here"
    3. Run this script: python record_official_responses.py

The script will create YAML cassette files in the fixtures/ directory.
These cassettes can then be used for deterministic testing without API calls.
"""
import os
import sys
from pathlib import Path

# Check for API key first
API_KEY = os.environ.get('FUGLE_API_KEY')

if not API_KEY:
    print("=" * 80)
    print("FUGLE_API_KEY not set - VCR recording skipped")
    print("=" * 80)
    print()
    print("To record official SDK responses:")
    print("  1. Install official SDK:")
    print("     pip install fugle-marketdata")
    print()
    print("  2. Get API key from Fugle Developer Portal:")
    print("     https://developer.fugle.tw/")
    print("     → API Management → Create API Key")
    print()
    print("  3. Export API key:")
    print("     export FUGLE_API_KEY='your-key-here'")
    print()
    print("  4. Run this script:")
    print("     python tests/fixtures/record_official_responses.py")
    print()
    print("=" * 80)
    sys.exit(0)

# Import VCR and official SDK
try:
    import vcr
except ImportError:
    print("ERROR: vcrpy not installed")
    print("Install with: pip install vcrpy")
    sys.exit(1)

try:
    from fugle_marketdata import RestClient
except ImportError:
    print("ERROR: fugle-marketdata not installed")
    print("Install with: pip install fugle-marketdata")
    sys.exit(1)

# Configure VCR
FIXTURES_DIR = Path(__file__).parent
vcr_config = vcr.VCR(
    cassette_library_dir=str(FIXTURES_DIR),
    record_mode='new_episodes',  # Record new interactions
    match_on=['method', 'scheme', 'host', 'port', 'path', 'query'],
    decode_compressed_response=True,
)

def record_quote():
    """Record quote endpoint response."""
    print("Recording quote(2330)...")
    with vcr_config.use_cassette('official_sdk_quote.yaml'):
        client = RestClient(api_key=API_KEY)
        response = client.stock.intraday.quote(symbol='2330')
        print(f"  ✓ Recorded quote response: {len(str(response))} bytes")

def record_ticker():
    """Record ticker endpoint response."""
    print("Recording ticker(2330)...")
    with vcr_config.use_cassette('official_sdk_ticker.yaml'):
        client = RestClient(api_key=API_KEY)
        response = client.stock.intraday.ticker(symbol='2330')
        print(f"  ✓ Recorded ticker response: {len(str(response))} bytes")

def record_trades():
    """Record trades endpoint response."""
    print("Recording trades(2330)...")
    with vcr_config.use_cassette('official_sdk_trades.yaml'):
        client = RestClient(api_key=API_KEY)
        response = client.stock.intraday.trades(symbol='2330')
        print(f"  ✓ Recorded trades response: {len(str(response))} bytes")

def record_candles():
    """Record candles endpoint response."""
    print("Recording candles(2330, '5')...")
    with vcr_config.use_cassette('official_sdk_candles.yaml'):
        client = RestClient(api_key=API_KEY)
        # Note: candles() signature may vary - adjust as needed
        response = client.stock.intraday.candles(symbol='2330')
        print(f"  ✓ Recorded candles response: {len(str(response))} bytes")

def record_volumes():
    """Record volumes endpoint response."""
    print("Recording volumes(2330)...")
    with vcr_config.use_cassette('official_sdk_volumes.yaml'):
        client = RestClient(api_key=API_KEY)
        response = client.stock.intraday.volumes(symbol='2330')
        print(f"  ✓ Recorded volumes response: {len(str(response))} bytes")

def main():
    """Record all endpoints."""
    print("=" * 80)
    print("Recording Official SDK Responses to VCR Cassettes")
    print("=" * 80)
    print()

    try:
        record_quote()
        record_ticker()
        record_trades()
        record_candles()
        record_volumes()
    except Exception as e:
        print(f"\nERROR during recording: {e}")
        print("\nPartial cassettes may have been created.")
        print("Check the fixtures/ directory for details.")
        sys.exit(1)

    print()
    print("=" * 80)
    print("✓ Recording complete!")
    print("=" * 80)
    print()
    print("Cassettes saved to:", FIXTURES_DIR)
    print()
    print("Next steps:")
    print("  1. Review cassettes to ensure they contain expected data")
    print("  2. Run compatibility tests: pytest py/tests/test_response_compatibility.py")
    print()

if __name__ == '__main__':
    main()
