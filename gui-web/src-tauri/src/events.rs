use serde::{Deserialize, Serialize};

use marketdata_core::websocket::channels::{parse_channel_data, ChannelData};
use marketdata_core::{
    AggregatesData, BooksData, CandleData, CandleHistoryItem, CandlesSnapshot, ConnectionEvent,
    HistoricalCandle, IndicesData, IntradayCandle, TradesData, WebSocketMessage,
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
#[serde(tag = "type")]
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
            ("data" | "snapshot", ChannelData::Trades(t)) => Some(Self::TradeTick(t)),
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
                message: format!("reconnect failed after {attempts} attempts"),
            },
            ConnectionEvent::Unauthenticated { message } => Self::Failed { message },
            ConnectionEvent::Error { message, .. } => Self::Failed { message },
        }
    }
}
