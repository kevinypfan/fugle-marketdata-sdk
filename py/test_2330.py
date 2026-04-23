#!/usr/bin/env python3
"""簡單的 REST API 範例"""

import os
from fugle_marketdata import RestClient

# 從環境變數取得 API key
api_key = os.environ.get("FUGLE_API_KEY")
if not api_key:
    print("請設定 FUGLE_API_KEY 環境變數")
    print("  export FUGLE_API_KEY='your-api-key'")
    exit(1)

# 建立 REST client
client = RestClient(api_key)

# 查詢台積電股票報價
quote = client.stock.intraday.quote("2330")
print(f"股票: 2330 台積電")
print(f"收盤價: {quote.get('closePrice')}")
print(f"漲跌: {quote.get('change')}")
print(f"漲跌幅: {quote.get('changePercent')}%")
