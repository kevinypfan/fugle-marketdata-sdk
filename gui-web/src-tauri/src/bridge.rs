use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use marketdata_core::rest::Auth;
use marketdata_core::websocket::channels::StockSubscription;
use marketdata_core::{
    AuthRequest, Channel, ConnectionConfig, Quote, ReconnectionConfig, RestClient, Ticker, Trade,
    WebSocketClient,
};

use crate::error::{AppError, AppResult};
use crate::events::{CandleDto, ConnectionStateDto, MarketEventDto, Timeframe};

const COALESCE_INTERVAL_MS: u64 = 16;
pub const MARKET_BATCH_EVENT: &str = "market-batch";
pub const CONN_STATE_EVENT: &str = "connection-state";

pub struct AppBridge {
    inner: Mutex<BridgeInner>,
}

struct BridgeInner {
    api_key: Option<String>,
    client: Option<Arc<WebSocketClient>>,
    subscriptions: HashSet<String>,
    pump_handles: Vec<JoinHandle<()>>,
}

impl Default for AppBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl AppBridge {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(BridgeInner {
                api_key: None,
                client: None,
                subscriptions: HashSet::new(),
                pump_handles: Vec::new(),
            }),
        }
    }

    pub async fn require_api_key(&self) -> AppResult<String> {
        let inner = self.inner.lock().await;
        inner.api_key.clone().ok_or(AppError::MissingApiKey)
    }

    pub async fn connect(
        &self,
        app: AppHandle,
        api_key: String,
        ws_url: Option<String>,
    ) -> AppResult<()> {
        let mut inner = self.inner.lock().await;

        if inner.client.is_some() {
            log::info!("connect() called while already connected — skipping");
            return Ok(());
        }

        let _ = app.emit(CONN_STATE_EVENT, ConnectionStateDto::Connecting);

        let auth = AuthRequest::with_api_key(&api_key);
        let config = match ws_url {
            Some(u) => ConnectionConfig::new(u, auth),
            None => ConnectionConfig::fugle_stock(auth),
        };
        let reconnect_cfg = ReconnectionConfig::new(
            5,
            Duration::from_secs(1),
            Duration::from_secs(60),
        )
        .expect("static reconnection config is valid");
        let client = Arc::new(WebSocketClient::with_reconnection_config(config, reconnect_cfg));

        if let Err(e) = client.connect().await {
            let dto = ConnectionStateDto::Failed {
                message: e.to_string(),
            };
            let _ = app.emit(CONN_STATE_EVENT, dto);
            return Err(e.into());
        }

        let _ = app.emit(CONN_STATE_EVENT, ConnectionStateDto::Connected);

        let (event_tx, event_rx) = unbounded_channel::<MarketEventDto>();

        let msg_pump = spawn_message_pump(Arc::clone(&client), event_tx.clone());
        let state_pump = spawn_state_pump(Arc::clone(&client), app.clone());
        let coalesce = spawn_coalesce_loop(app.clone(), event_rx);

        inner.api_key = Some(api_key);
        inner.client = Some(client);
        inner.pump_handles = vec![msg_pump, state_pump, coalesce];

        // Re-apply any subscriptions held from a previous session.
        let to_resub: Vec<String> = inner.subscriptions.iter().cloned().collect();
        let client = inner.client.as_ref().unwrap().clone();
        drop(inner);
        for symbol in to_resub {
            for ch in channels_for(&symbol) {
                let sub = StockSubscription::new(*ch, &symbol);
                if let Err(e) = client.subscribe_channel(sub).await {
                    log::warn!("resubscribe {symbol} {ch:?} failed: {e}");
                }
            }
        }

        Ok(())
    }

    pub async fn disconnect(&self) -> AppResult<()> {
        let mut inner = self.inner.lock().await;
        if let Some(client) = inner.client.take() {
            let _ = client.disconnect().await;
        }
        for h in inner.pump_handles.drain(..) {
            h.abort();
        }
        Ok(())
    }

    pub async fn subscribe(&self, symbol: String) -> AppResult<()> {
        let mut inner = self.inner.lock().await;
        let client = inner.client.clone().ok_or(AppError::NotConnected)?;
        inner.subscriptions.insert(symbol.clone());
        drop(inner);
        for ch in channels_for(&symbol) {
            let sub = StockSubscription::new(*ch, &symbol);
            client.subscribe_channel(sub).await?;
        }
        Ok(())
    }

    pub async fn unsubscribe(&self, symbol: String) -> AppResult<()> {
        let mut inner = self.inner.lock().await;
        let client = inner.client.clone().ok_or(AppError::NotConnected)?;
        inner.subscriptions.remove(&symbol);
        drop(inner);
        for ch in channels_for(&symbol) {
            let sub = StockSubscription::new(*ch, &symbol);
            client.unsubscribe_channel(&sub).await?;
        }
        Ok(())
    }
}

fn channels_for(symbol: &str) -> &'static [Channel] {
    if symbol.starts_with("IX") {
        &[Channel::Indices]
    } else {
        &[
            Channel::Aggregates,
            Channel::Trades,
            Channel::Candles,
            Channel::Books,
        ]
    }
}

fn spawn_message_pump(
    client: Arc<WebSocketClient>,
    tx: UnboundedSender<MarketEventDto>,
) -> JoinHandle<()> {
    let receiver = client.messages();
    tokio::task::spawn_blocking(move || loop {
        match receiver.receive() {
            Ok(ws_msg) => {
                log::debug!(
                    "ws message event={} channel={:?}",
                    ws_msg.event,
                    ws_msg.channel
                );
                // TEMP — dump raw data for trades channel to diagnose empty
                // trades array. Remove after schema confirmed.
                if ws_msg.channel.as_deref() == Some("trades") {
                    log::info!("trades raw data: {:?}", ws_msg.data);
                }
                if let Some(ev) = MarketEventDto::from_ws(&ws_msg) {
                    if tx.send(ev).is_err() {
                        break;
                    }
                } else if matches!(ws_msg.event.as_str(), "data" | "snapshot") {
                    // data/snapshot that from_ws couldn't map — parser miss or
                    // unhandled channel. subscribed/pong/error are expected drops.
                    log::warn!(
                        "ws message dropped by from_ws: event={} channel={:?} data={:?}",
                        ws_msg.event,
                        ws_msg.channel,
                        ws_msg.data
                    );
                }
            }
            Err(_) => break,
        }
    })
}

fn spawn_state_pump(client: Arc<WebSocketClient>, app: AppHandle) -> JoinHandle<()> {
    let events = Arc::clone(client.state_events());
    tokio::task::spawn_blocking(move || {
        let rx = events.blocking_lock();
        while let Ok(ev) = rx.recv() {
            let dto: ConnectionStateDto = ev.into();
            let _ = app.emit(CONN_STATE_EVENT, dto);
        }
    })
}

fn spawn_coalesce_loop(
    app: AppHandle,
    mut event_rx: tokio::sync::mpsc::UnboundedReceiver<MarketEventDto>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut buf: Vec<MarketEventDto> = Vec::with_capacity(64);
        let mut interval = tokio::time::interval(Duration::from_millis(COALESCE_INTERVAL_MS));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                maybe = event_rx.recv() => {
                    match maybe {
                        Some(ev) => buf.push(ev),
                        None => break,
                    }
                }
                _ = interval.tick() => {
                    if !buf.is_empty() {
                        let batch = std::mem::take(&mut buf);
                        if app.emit(MARKET_BATCH_EVENT, &batch).is_err() {
                            log::warn!("emit market-batch failed; window closed?");
                            break;
                        }
                    }
                }
            }
        }
    })
}

// ─── REST seed helpers (run on blocking thread pool) ─────────────────────────

pub fn fetch_candles_blocking(
    symbol: &str,
    timeframe: Timeframe,
    api_key: &str,
    rest_base_url: Option<&str>,
) -> AppResult<Vec<CandleDto>> {
    let rest = build_rest(api_key, rest_base_url);
    if timeframe.is_intraday() {
        let resp = rest
            .stock()
            .intraday()
            .candles()
            .symbol(symbol)
            .timeframe(timeframe.as_api_str())
            .send()?;
        Ok(resp.data.into_iter().map(Into::into).collect())
    } else {
        // Range must be strictly < 1 year per API; pad below 365 to absorb
        // UTC↔TPE day-boundary skew on the server side.
        let from = date_n_days_ago(360);
        let resp = rest
            .stock()
            .historical()
            .candles()
            .symbol(symbol)
            .timeframe(timeframe.as_api_str())
            .from(&from)
            .sort("asc")
            .send()?;
        Ok(resp.data.into_iter().map(Into::into).collect())
    }
}

pub fn fetch_ticker_blocking(
    symbol: &str,
    api_key: &str,
    rest_base_url: Option<&str>,
) -> AppResult<Ticker> {
    let rest = build_rest(api_key, rest_base_url);
    Ok(rest.stock().intraday().ticker().symbol(symbol).send()?)
}

pub fn fetch_trades_blocking(
    symbol: &str,
    api_key: &str,
    rest_base_url: Option<&str>,
    offset: Option<u32>,
    limit: Option<u32>,
    is_trial: Option<bool>,
) -> AppResult<Vec<Trade>> {
    let rest = build_rest(api_key, rest_base_url);
    let stock = rest.stock();
    let intraday = stock.intraday();
    let mut builder = intraday.trades().symbol(symbol);
    if let Some(o) = offset {
        builder = builder.offset(o);
    }
    if let Some(l) = limit {
        builder = builder.limit(l);
    }
    if let Some(t) = is_trial {
        builder = builder.is_trial(t);
    }
    let resp = builder.send()?;
    Ok(resp.data)
}

pub fn fetch_quote_blocking(
    symbol: &str,
    api_key: &str,
    rest_base_url: Option<&str>,
) -> AppResult<Quote> {
    let rest = build_rest(api_key, rest_base_url);
    Ok(rest.stock().intraday().quote().symbol(symbol).send()?)
}

fn build_rest(api_key: &str, base_url: Option<&str>) -> RestClient {
    let rest = RestClient::new(Auth::ApiKey(api_key.to_string()));
    match base_url {
        Some(u) => rest.base_url(u),
        None => rest,
    }
}

/// `YYYY-MM-DD` for the date `days` ago in UTC (Hinnant civil_from_days,
/// inlined to avoid pulling chrono just for one date string).
fn date_n_days_ago(days: i64) -> String {
    let now_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let day_number = now_secs.div_euclid(86_400) - days;
    let z = day_number + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}")
}
