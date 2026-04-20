use serde::{Deserialize, Serialize};

use marketdata_core::websocket::channels::{parse_channel_data, ChannelData};
use marketdata_core::{
    AggregatesData, BooksData, CandleData, CandleHistoryItem, CandlesSnapshot, ConnectionEvent,
    HistoricalCandle, IndicesData, IntradayCandle, StreamTrade, TradesData, WebSocketMessage,
};

/// Candle timeframe accepted by both intraday and historical endpoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Timeframe {
    #[serde(rename = "1")]
    Min1,
    #[serde(rename = "D")]
    Day,
    #[serde(rename = "W")]
    Week,
    #[serde(rename = "M")]
    Month,
}

impl Timeframe {
    pub fn as_api_str(self) -> &'static str {
        match self {
            Self::Min1 => "1",
            Self::Day => "D",
            Self::Week => "W",
            Self::Month => "M",
        }
    }

    pub fn is_intraday(self) -> bool {
        matches!(self, Self::Min1)
    }
}

/// Unified candle DTO so the frontend handles intraday/historical/streaming the same way.
#[derive(Debug, Clone, Serialize)]
pub struct CandleDto {
    pub date: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: i64,
}

impl From<IntradayCandle> for CandleDto {
    fn from(c: IntradayCandle) -> Self {
        Self {
            date: c.date,
            open: c.open,
            high: c.high,
            low: c.low,
            close: c.close,
            volume: c.volume,
        }
    }
}

impl From<HistoricalCandle> for CandleDto {
    fn from(c: HistoricalCandle) -> Self {
        Self {
            date: c.date,
            open: c.open,
            high: c.high,
            low: c.low,
            close: c.close,
            volume: c.volume,
        }
    }
}

impl From<CandleHistoryItem> for CandleDto {
    fn from(c: CandleHistoryItem) -> Self {
        Self {
            date: c.date,
            open: c.open,
            high: c.high,
            low: c.low,
            close: c.close,
            volume: c.volume,
        }
    }
}

/// Single market event pushed from backend to frontend (one element of `market-batch`).
///
/// REST-seeded data (ticker info, trade history, candle history) does not flow
/// through this enum — those are returned directly from `fetch_ticker`,
/// `fetch_trades`, and `fetch_candles` commands so the frontend can await them.
#[derive(Debug, Clone, Serialize)]
// Use "kind" — "type" collides with TradesData/AggregatesData/IndicesData
// which have their own `#[serde(rename = "type")]` field (data_type="EQUITY"
// etc.), and flatten would overwrite the enum discriminant.
#[serde(tag = "kind")]
pub enum MarketEventDto {
    Aggregate(AggregatesData),
    TradeTick(TradesData),
    BookSnap(BooksData),
    CandleTick(CandleData),
    CandleHistory(CandlesSnapshot),
    Indices(IndicesData),
}

impl MarketEventDto {
    pub fn from_ws(msg: &WebSocketMessage) -> Option<Self> {
        let channel = msg.channel.as_deref()?;
        let data = msg.data.as_ref()?;
        let is_snapshot = msg.event == "snapshot";
        match (
            msg.event.as_str(),
            parse_channel_data(channel, data, is_snapshot).ok()?,
        ) {
            ("data" | "snapshot", ChannelData::Trades(mut t)) => {
                // "data" events send a single trade as flat top-level fields
                // (price/size/bid/ask), not wrapped in `trades: [...]`. SDK's
                // TradesData has `#[serde(default)]` on `trades`, so those
                // events deserialize into an empty Vec. Reconstruct here.
                if t.trades.is_empty() {
                    if let Some(obj) = data.as_object() {
                        if let (Some(price), Some(size)) = (
                            obj.get("price").and_then(serde_json::Value::as_f64),
                            obj.get("size").and_then(serde_json::Value::as_i64),
                        ) {
                            t.trades.push(StreamTrade {
                                price,
                                size,
                                bid: obj.get("bid").and_then(serde_json::Value::as_f64),
                                ask: obj.get("ask").and_then(serde_json::Value::as_f64),
                            });
                        }
                    }
                }
                Some(Self::TradeTick(t))
            }
            ("data" | "snapshot", ChannelData::Books(b)) => Some(Self::BookSnap(b)),
            ("data", ChannelData::CandleData(c)) => Some(Self::CandleTick(c)),
            ("snapshot", ChannelData::CandlesSnapshot(s)) => Some(Self::CandleHistory(s)),
            ("data" | "snapshot", ChannelData::Aggregates(a)) => Some(Self::Aggregate(a)),
            ("data" | "snapshot", ChannelData::Indices(i)) => Some(Self::Indices(i)),
            _ => None,
        }
    }
}

/// Connection state pushed on a separate `connection-state` channel.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum ConnectionStateDto {
    Connecting,
    Connected,
    Reconnecting { attempt: u32 },
    Disconnected { reason: String },
    Failed { message: String },
}

impl From<ConnectionEvent> for ConnectionStateDto {
    fn from(e: ConnectionEvent) -> Self {
        match e {
            ConnectionEvent::Connecting => Self::Connecting,
            ConnectionEvent::Connected => Self::Connected,
            ConnectionEvent::Authenticated => Self::Connected,
            ConnectionEvent::Disconnected { reason, .. } => Self::Disconnected { reason },
            ConnectionEvent::Reconnecting { attempt } => Self::Reconnecting { attempt },
            ConnectionEvent::ReconnectFailed { attempts } => Self::Failed {
                // SDK emits attempts=0 when the close code is non-retriable
                // (1000 normal, 4xxx auth/app), i.e. gave up before trying;
                // attempts>0 means it actually tried and ran out.
                message: if attempts == 0 {
                    "連線中斷（無法自動重連）".to_string()
                } else {
                    format!("重連失敗（{attempts} 次嘗試）")
                },
            },
            ConnectionEvent::Unauthenticated { message } => Self::Failed { message },
            ConnectionEvent::Error { message, .. } => Self::Failed { message },
        }
    }
}
