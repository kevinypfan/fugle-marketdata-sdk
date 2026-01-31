#!/usr/bin/env python3
"""WebSocket 自動 Callback 模式測試

展示如何用純 callback 風格處理 WebSocket 訊息，
不需要手動 iterator 迴圈！
"""

import os
import time
from marketdata_py import WebSocketClient

# 從環境變數取得 API key
api_key = os.environ.get("FUGLE_API_KEY")
if not api_key:
    print("請設定 FUGLE_API_KEY 環境變數")
    print("  export FUGLE_API_KEY='your-api-key'")
    exit(1)

# 建立 WebSocket client
ws = WebSocketClient(api_key)
stock = ws.stock

# 用來追蹤收到的訊息
received_messages = []


# === Callback functions ===
def on_message(msg):
    """收到訊息時的 callback（自動觸發！）"""
    event = msg.get('event')
    data = msg.get('data', {})
    print(f"✓ 收到訊息 message={msg}")

    if event == 'subscribed':
        print(f"  [訂閱成功] channel={data.get('channel')}, symbol={data.get('symbol')}")
    elif event == 'snapshot':
        print(f"  [快照] {msg.get('channel')}:{data.get('symbol')} 價格={data.get('price')} 成交量={data.get('volume')}")
    elif event == 'data':
        print(f"  [即時] {msg.get('channel')}:{data.get('symbol')} 價格={data.get('price')}")
    elif event == 'heartbeat':
        print(f"  [心跳] time={data.get('time')}")
    else:
        print(f"  [{event}] {data}")

    received_messages.append(msg)


def on_connect():
    """連線成功的 callback"""
    print("✓ 已連線!")


def on_disconnect(code, reason):
    """斷線的 callback"""
    print(f"✓ 已斷線! code={code}, reason={reason}")


# ==========================================
# 1️⃣ 在 connect 之前註冊 callbacks
# ==========================================
print("1. 註冊 callbacks...")
stock.on("message", on_message)  # 關鍵！message callback 會自動觸發
stock.on("connect", on_connect)
stock.on("disconnect", on_disconnect)

# ==========================================
# 2️⃣ 連線（會自動啟動背景線程）
# ==========================================
print("\n2. 連線中...")
stock.connect()

# ==========================================
# 3️⃣ 訂閱
# ==========================================
print("\n3. 訂閱 2330 trades...")
stock.subscribe("trades", "2330")

# ==========================================
# 4️⃣ 等待訊息（callback 會自動被調用）
# ==========================================
print("\n4. 等待訊息 (30秒)...\n")
time.sleep(30)

# ==========================================
# 5️⃣ 結果
# ==========================================
print(f"\n=== 總共收到 {len(received_messages)} 則訊息 ===")

# ==========================================
# 6️⃣ 斷線
# ==========================================
print("\n5. 斷線中...")
stock.disconnect()
print("\n完成!")
