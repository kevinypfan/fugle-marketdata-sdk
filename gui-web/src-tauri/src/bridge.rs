use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use marketdata_core::models::futopt::{FutOptChannel, FutOptQuote, FutOptTicker, Product};
use marketdata_core::models::SubscribeRequest;
use marketdata_core::rest::Auth;
use marketdata_core::websocket::channels::FutOptSubscription;
use marketdata_core::{
    AuthRequest, Channel, ConnectionConfig, ReconnectionConfig, RestClient, WebSocketClient,
    Quote, Ticker, Trade,
};

use crate::error::{AppError, AppResult};
use crate::events::{
    CandleDto, ConnectionStateDto, Market, MarketConnectionStateDto, MarketEventDto,
    TaggedMarketEvent, Timeframe,
};

const COALESCE_INTERVAL_MS: u64 = 16;
pub const MARKET_BATCH_EVENT: &str = "market-batch";
pub const CONN_STATE_EVENT: &str = "connection-state";

pub struct AppBridge {
    inner: Mutex<BridgeInner>,
}

/// Per-market client + its subscription set + its pump handles. Both markets
/// share the coalesce loop so frontend gets a single ordered stream tagged by
/// `market`.
struct MarketClient {
    client: Arc<WebSocketClient>,
    subscriptions: HashSet<String>,
    pump_handles: Vec<JoinHandle<()>>,
}

struct BridgeInner {
    api_key: Option<String>,
    stock: Option<MarketClient>,
    futopt: Option<MarketClient>,
    /// Shared sender into the coalesce loop. Created on first connect, reused
    /// by both markets. `None` when nothing is connected (coalesce loop absent).
    event_tx: Option<UnboundedSender<TaggedMarketEvent>>,
    coalesce_handle: Option<JoinHandle<()>>,
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
                stock: None,
                futopt: None,
                event_tx: None,
                coalesce_handle: None,
            }),
        }
    }

    pub async fn require_api_key(&self) -> AppResult<String> {
        let inner = self.inner.lock().await;
        inner.api_key.clone().ok_or(AppError::MissingApiKey)
    }

    pub async fn connect_stock(
        &self,
        app: AppHandle,
        api_key: String,
        ws_url: Option<String>,
    ) -> AppResult<()> {
        self.connect_market(app, Market::Stock, api_key, ws_url).await
    }

    pub async fn connect_futopt(
        &self,
        app: AppHandle,
        api_key: String,
        ws_url: Option<String>,
    ) -> AppResult<()> {
        self.connect_market(app, Market::Futopt, api_key, ws_url).await
    }

    async fn connect_market(
        &self,
        app: AppHandle,
        market: Market,
        api_key: String,
        ws_url: Option<String>,
    ) -> AppResult<()> {
        let mut inner = self.inner.lock().await;

        if inner.existing_for(market).is_some() {
            log::info!("connect_{market:?} called while already connected — skipping");
            return Ok(());
        }

        let _ = app.emit(
            CONN_STATE_EVENT,
            MarketConnectionStateDto { market, state: ConnectionStateDto::Connecting },
        );

        let auth = AuthRequest::with_api_key(&api_key);
        let config = match (market, ws_url) {
            (_, Some(u)) => ConnectionConfig::new(u, auth),
            (Market::Stock, None) => ConnectionConfig::fugle_stock(auth),
            (Market::Futopt, None) => ConnectionConfig::fugle_futopt(auth),
        };
        let reconnect_cfg = ReconnectionConfig::new(
            5,
            Duration::from_secs(1),
            Duration::from_secs(60),
        )
        .expect("static reconnection config is valid");
        let client = Arc::new(WebSocketClient::with_reconnection_config(config, reconnect_cfg));

        if let Err(e) = client.connect().await {
            let _ = app.emit(
                CONN_STATE_EVENT,
                MarketConnectionStateDto {
                    market,
                    state: ConnectionStateDto::Failed { message: e.to_string() },
                },
            );
            return Err(e.into());
        }

        let _ = app.emit(
            CONN_STATE_EVENT,
            MarketConnectionStateDto { market, state: ConnectionStateDto::Connected },
        );

        // Shared coalesce infra: create on first connect, reuse thereafter.
        let tx = if let Some(tx) = &inner.event_tx {
            tx.clone()
        } else {
            let (tx, rx) = unbounded_channel::<TaggedMarketEvent>();
            let handle = spawn_coalesce_loop(app.clone(), rx);
            inner.event_tx = Some(tx.clone());
            inner.coalesce_handle = Some(handle);
            tx
        };

        let msg_pump = spawn_message_pump(Arc::clone(&client), market, tx);
        let state_pump = spawn_state_pump(Arc::clone(&client), market, app.clone());

        inner.api_key = Some(api_key);
        let held = MarketClient {
            client: Arc::clone(&client),
            subscriptions: HashSet::new(),
            pump_handles: vec![msg_pump, state_pump],
        };
        // Preserve subscriptions from a previous session of this market.
        let prior_subs: Vec<String> = inner
            .existing_for(market)
            .map(|mc| mc.subscriptions.iter().cloned().collect())
            .unwrap_or_default();
        match market {
            Market::Stock => inner.stock = Some(held),
            Market::Futopt => inner.futopt = Some(held),
        }

        drop(inner);
        // prior_subs is only non-empty when connect_market is invoked a
        // second time against a still-existing MarketClient — our current
        // `connect` paths short-circuit when that slot is `Some(_)`, so this
        // resubscribe path is effectively latent. Default `after_hours=false`
        // for safety; once we add explicit bridge-level reconnect, the
        // futoptSession needs to be threaded through here.
        for symbol in prior_subs {
            if let Err(e) = subscribe_on(&client, market, &symbol, false).await {
                log::warn!("resubscribe {market:?} {symbol} failed: {e}");
            }
        }

        Ok(())
    }

    pub async fn disconnect(&self) -> AppResult<()> {
        let mut inner = self.inner.lock().await;
        if let Some(mc) = inner.stock.take() {
            let _ = mc.client.disconnect().await;
            for h in mc.pump_handles {
                h.abort();
            }
        }
        if let Some(mc) = inner.futopt.take() {
            let _ = mc.client.disconnect().await;
            for h in mc.pump_handles {
                h.abort();
            }
        }
        if let Some(h) = inner.coalesce_handle.take() {
            h.abort();
        }
        inner.event_tx = None;
        Ok(())
    }

    pub async fn subscribe(
        &self,
        market: Market,
        symbol: String,
        after_hours: bool,
    ) -> AppResult<()> {
        let mut inner = self.inner.lock().await;
        let mc = inner.existing_for_mut(market).ok_or(AppError::NotConnected)?;
        mc.subscriptions.insert(symbol.clone());
        let client = Arc::clone(&mc.client);
        drop(inner);
        subscribe_on(&client, market, &symbol, after_hours).await
    }

    pub async fn unsubscribe(
        &self,
        market: Market,
        symbol: String,
        after_hours: bool,
    ) -> AppResult<()> {
        let mut inner = self.inner.lock().await;
        let mc = inner.existing_for_mut(market).ok_or(AppError::NotConnected)?;
        mc.subscriptions.remove(&symbol);
        let client = Arc::clone(&mc.client);
        drop(inner);
        unsubscribe_on(&client, market, &symbol, after_hours).await
    }
}

impl BridgeInner {
    fn existing_for(&self, market: Market) -> Option<&MarketClient> {
        match market {
            Market::Stock => self.stock.as_ref(),
            Market::Futopt => self.futopt.as_ref(),
        }
    }

    fn existing_for_mut(&mut self, market: Market) -> Option<&mut MarketClient> {
        match market {
            Market::Stock => self.stock.as_mut(),
            Market::Futopt => self.futopt.as_mut(),
        }
    }
}

/// Subscribe `symbol` to the appropriate channels for the given market.
/// Stock uses the generic `SubscribeRequest` path; FutOpt uses the typed
/// `FutOptSubscription` so the `afterHours` flag lands in the wire payload
/// correctly (server distinguishes 日盤 vs 夜盤 purely by that field).
async fn subscribe_on(
    client: &Arc<WebSocketClient>,
    market: Market,
    symbol: &str,
    after_hours: bool,
) -> AppResult<()> {
    match market {
        Market::Stock => {
            for ch in stock_channels_for(symbol) {
                let req = SubscribeRequest::new(*ch, symbol);
                client.subscribe(req).await?;
            }
        }
        Market::Futopt => {
            for ch in futopt_channels() {
                let sub = FutOptSubscription {
                    channel: *ch,
                    symbol: symbol.to_string(),
                    after_hours,
                };
                client.subscribe_futopt_channel(sub).await?;
            }
        }
    }
    Ok(())
}

async fn unsubscribe_on(
    client: &Arc<WebSocketClient>,
    market: Market,
    symbol: &str,
    after_hours: bool,
) -> AppResult<()> {
    match market {
        Market::Stock => {
            for ch in stock_channels_for(symbol) {
                let key = format!("{}:{}", ch.as_str(), symbol);
                client.unsubscribe(&key).await?;
            }
        }
        Market::Futopt => {
            for ch in futopt_channels() {
                let sub = FutOptSubscription {
                    channel: *ch,
                    symbol: symbol.to_string(),
                    after_hours,
                };
                client.unsubscribe_futopt_channel(&sub).await?;
            }
        }
    }
    Ok(())
}

fn stock_channels_for(symbol: &str) -> &'static [Channel] {
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

fn futopt_channels() -> &'static [FutOptChannel] {
    &[
        FutOptChannel::Aggregates,
        FutOptChannel::Trades,
        FutOptChannel::Candles,
        FutOptChannel::Books,
    ]
}

fn spawn_message_pump(
    client: Arc<WebSocketClient>,
    market: Market,
    tx: UnboundedSender<TaggedMarketEvent>,
) -> JoinHandle<()> {
    let receiver = client.messages();
    tokio::task::spawn_blocking(move || loop {
        match receiver.receive() {
            Ok(ws_msg) => {
                // Control frames (subscribed ack, auth, errors) at info;
                // per-tick `data`/`snapshot` at debug so an active multi-
                // channel subscription doesn't flood logs at info level.
                match ws_msg.event.as_str() {
                    "data" | "snapshot" => {
                        log::debug!(
                            "ws {market:?} event={} channel={:?}",
                            ws_msg.event,
                            ws_msg.channel
                        );
                    }
                    _ => {
                        log::info!(
                            "ws {market:?} event={} channel={:?} data={:?}",
                            ws_msg.event,
                            ws_msg.channel,
                            ws_msg.data
                        );
                    }
                }
                if let Some(ev) = MarketEventDto::from_ws(&ws_msg) {
                    if tx.send(TaggedMarketEvent { market, event: ev }).is_err() {
                        break;
                    }
                } else if matches!(ws_msg.event.as_str(), "data" | "snapshot") {
                    log::warn!(
                        "ws {market:?} message dropped by from_ws: event={} channel={:?} data={:?}",
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

fn spawn_state_pump(
    client: Arc<WebSocketClient>,
    market: Market,
    app: AppHandle,
) -> JoinHandle<()> {
    let events = Arc::clone(client.state_events());
    tokio::task::spawn_blocking(move || {
        let rx = events.blocking_lock();
        while let Ok(ev) = rx.recv() {
            let state: ConnectionStateDto = ev.into();
            let _ = app.emit(CONN_STATE_EVENT, MarketConnectionStateDto { market, state });
        }
    })
}

fn spawn_coalesce_loop(
    app: AppHandle,
    mut event_rx: tokio::sync::mpsc::UnboundedReceiver<TaggedMarketEvent>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut buf: Vec<TaggedMarketEvent> = Vec::with_capacity(64);
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

// ─── FutOpt REST seed helpers ────────────────────────────────────────────────

pub fn fetch_futopt_ticker_blocking(
    symbol: &str,
    api_key: &str,
    rest_base_url: Option<&str>,
) -> AppResult<FutOptTicker> {
    let rest = build_rest(api_key, rest_base_url);
    Ok(rest.futopt().intraday().ticker().symbol(symbol).send()?)
}

pub fn fetch_futopt_quote_blocking(
    symbol: &str,
    api_key: &str,
    rest_base_url: Option<&str>,
    after_hours: bool,
) -> AppResult<FutOptQuote> {
    let rest = build_rest(api_key, rest_base_url);
    let futopt = rest.futopt();
    let intraday = futopt.intraday();
    let mut builder = intraday.quote().symbol(symbol);
    if after_hours {
        builder = builder.after_hours();
    }
    Ok(builder.send()?)
}

pub fn fetch_futopt_trades_blocking(
    symbol: &str,
    api_key: &str,
    rest_base_url: Option<&str>,
    offset: Option<i32>,
    limit: Option<i32>,
    after_hours: bool,
) -> AppResult<Vec<Trade>> {
    let rest = build_rest(api_key, rest_base_url);
    let futopt = rest.futopt();
    let intraday = futopt.intraday();
    let mut builder = intraday.trades().symbol(symbol);
    if let Some(o) = offset {
        builder = builder.offset(o);
    }
    if let Some(l) = limit {
        builder = builder.limit(l);
    }
    if after_hours {
        builder = builder.after_hours();
    }
    let resp = builder.send()?;
    Ok(resp.data)
}

pub fn fetch_futopt_candles_blocking(
    symbol: &str,
    timeframe: Timeframe,
    api_key: &str,
    rest_base_url: Option<&str>,
    after_hours: bool,
) -> AppResult<Vec<CandleDto>> {
    let rest = build_rest(api_key, rest_base_url);
    if timeframe.is_intraday() {
        let futopt = rest.futopt();
        let intraday = futopt.intraday();
        let mut builder = intraday
            .candles()
            .symbol(symbol)
            .timeframe(timeframe.as_api_str());
        if after_hours {
            builder = builder.after_hours();
        }
        let resp = builder.send()?;
        Ok(resp.data.into_iter().map(Into::into).collect())
    } else {
        let from = date_n_days_ago(360);
        let resp = rest
            .futopt()
            .historical()
            .candles()
            .symbol(symbol)
            .timeframe(timeframe.as_api_str())
            .from(&from)
            .send()?;
        // FutOpt historical uses `.candles` (not `.data`) for the series.
        Ok(resp.candles.into_iter().map(Into::into).collect())
    }
}

pub fn fetch_futopt_products_blocking(
    api_key: &str,
    rest_base_url: Option<&str>,
) -> AppResult<Vec<Product>> {
    let rest = build_rest(api_key, rest_base_url);
    let resp = rest.futopt().intraday().products().send()?;
    Ok(resp.data)
}

fn build_rest(api_key: &str, base_url: Option<&str>) -> RestClient {
    let rest = RestClient::new(Auth::ApiKey(api_key.to_string()));
    match base_url {
        Some(u) => rest.base_url(u),
        None => rest,
    }
}

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
