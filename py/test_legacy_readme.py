#!/usr/bin/env python3
"""Smoke test mirroring fugle-marketdata-python's README examples.

Each section corresponds to a code block in the legacy README so you can
eyeball-compare API surface and runtime behavior. The only intentional
difference is the import path:

    Legacy:  from fugle_marketdata import WebSocketClient, RestClient
    Ours:    from marketdata_py    import WebSocketClient, RestClient

Run individual sections, or all of them:

    export FUGLE_API_KEY='your-key'
    python test_legacy_readme.py rest
    python test_legacy_readme.py ws
    python test_legacy_readme.py errors
    python test_legacy_readme.py             # all sections
"""

import os
import sys
import time

from marketdata_py import (
    RestClient,
    WebSocketClient,
    FugleAPIError,
)


API_KEY = os.environ.get("FUGLE_API_KEY")
if not API_KEY:
    print("Set FUGLE_API_KEY environment variable first.")
    sys.exit(1)


# ---------------------------------------------------------------------------
# Section 1 — REST API (legacy README "REST API" block)
# ---------------------------------------------------------------------------
def section_rest():
    print("=" * 60)
    print("Section 1: REST API")
    print("=" * 60)

    client = RestClient(api_key=API_KEY)
    stock = client.stock  # Stock REST API client
    print(stock.intraday.quote(symbol="2330"))
    print()


# ---------------------------------------------------------------------------
# Section 2 — WebSocket API (legacy README "WebSocket API" block)
# ---------------------------------------------------------------------------
def section_ws():
    print("=" * 60)
    print("Section 2: WebSocket API")
    print("=" * 60)

    def handle_message(message):
        print(f"message: {message}")

    def handle_connect():
        print("connected")

    def handle_disconnect(code, message):
        print(f"disconnect: {code}, {message}")

    def handle_error(error, code):
        # NOTE: legacy README shows `handle_error(error)` but the new SDK
        # passes (message, code). Both fields surface here for parity.
        print(f"error: {error} (code={code})")

    client = WebSocketClient(api_key=API_KEY)
    stock = client.stock
    stock.on("connect", handle_connect)
    stock.on("message", handle_message)
    stock.on("disconnect", handle_disconnect)
    stock.on("error", handle_error)
    stock.connect()
    stock.subscribe({
        "channel": "trades",
        "symbol": "2330",
    })

    # Stream for ~10 seconds then disconnect (the README example never
    # disconnects; we add a bounded wait so the script can finish).
    time.sleep(10)
    stock.disconnect()
    print()


# ---------------------------------------------------------------------------
# Section 3 — Error handling (legacy README "Catching API Errors" block)
# ---------------------------------------------------------------------------
def section_errors():
    print("=" * 60)
    print("Section 3: Error handling")
    print("=" * 60)

    client = RestClient(api_key=API_KEY)

    # 3a. Generic catch — happy path
    try:
        data = client.stock.intraday.quote(symbol="2330")
        print(f"OK: got quote for 2330 ({len(str(data))} bytes)")
    except FugleAPIError as e:
        print(f"Error: {e}")

    # 3b. Common error scenario — invalid symbol triggers an HTTP error
    print("\nTriggering an invalid-symbol error:")
    try:
        client.stock.intraday.quote(symbol="INVALID_SYMBOL")
    except FugleAPIError as e:
        # The legacy README accesses e.message / e.url / e.status_code /
        # e.params / e.response_text. Our MarketDataError variants don't
        # currently expose those as attributes, but `str(e)` carries the
        # diagnostic. The except-clause itself is the API contract being
        # validated here.
        print(f"  caught FugleAPIError: {e}")
        print(f"  type: {type(e).__name__}")
    print()


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------
SECTIONS = {
    "rest": section_rest,
    "ws": section_ws,
    "errors": section_errors,
}


def main():
    selected = sys.argv[1:] or list(SECTIONS.keys())
    for name in selected:
        if name not in SECTIONS:
            print(f"Unknown section: {name}")
            print(f"Available: {', '.join(SECTIONS.keys())}")
            sys.exit(1)
    for name in selected:
        SECTIONS[name]()


if __name__ == "__main__":
    main()
