use tauri::{AppHandle, State};

use marketdata_core::models::futopt::{FutOptQuote, FutOptTicker, Product};
use marketdata_core::{Quote, Ticker, Trade};

use crate::bridge::{
    fetch_candles_blocking, fetch_futopt_candles_blocking, fetch_futopt_products_blocking,
    fetch_futopt_quote_blocking, fetch_futopt_ticker_blocking, fetch_futopt_trades_blocking,
    fetch_quote_blocking, fetch_ticker_blocking, fetch_trades_blocking, AppBridge,
};
use crate::error::{AppError, AppResult};
use crate::events::{CandleDto, Market, Timeframe};

#[tauri::command]
pub async fn connect(
    app: AppHandle,
    state: State<'_, AppBridge>,
    api_key: String,
    ws_url: Option<String>,
) -> AppResult<()> {
    state.connect_stock(app, api_key, ws_url).await
}

#[tauri::command]
pub async fn connect_futopt(
    app: AppHandle,
    state: State<'_, AppBridge>,
    api_key: String,
    ws_url: Option<String>,
) -> AppResult<()> {
    state.connect_futopt(app, api_key, ws_url).await
}

#[tauri::command]
pub async fn disconnect(state: State<'_, AppBridge>) -> AppResult<()> {
    state.disconnect().await
}

#[tauri::command]
pub async fn subscribe(state: State<'_, AppBridge>, symbol: String) -> AppResult<()> {
    state.subscribe(Market::Stock, symbol, false).await
}

#[tauri::command]
pub async fn unsubscribe(state: State<'_, AppBridge>, symbol: String) -> AppResult<()> {
    state.unsubscribe(Market::Stock, symbol, false).await
}

#[tauri::command]
pub async fn subscribe_futopt(
    state: State<'_, AppBridge>,
    symbol: String,
    after_hours: Option<bool>,
) -> AppResult<()> {
    state
        .subscribe(Market::Futopt, symbol, after_hours.unwrap_or(false))
        .await
}

#[tauri::command]
pub async fn unsubscribe_futopt(
    state: State<'_, AppBridge>,
    symbol: String,
    after_hours: Option<bool>,
) -> AppResult<()> {
    state
        .unsubscribe(Market::Futopt, symbol, after_hours.unwrap_or(false))
        .await
}

#[tauri::command]
pub async fn fetch_candles(
    state: State<'_, AppBridge>,
    symbol: String,
    timeframe: Timeframe,
    rest_base_url: Option<String>,
) -> AppResult<Vec<CandleDto>> {
    let api_key = state.require_api_key().await?;
    tokio::task::spawn_blocking(move || {
        fetch_candles_blocking(&symbol, timeframe, &api_key, rest_base_url.as_deref())
    })
    .await
    .map_err(AppError::from)?
}

#[tauri::command]
pub async fn fetch_ticker(
    state: State<'_, AppBridge>,
    symbol: String,
    rest_base_url: Option<String>,
) -> AppResult<Ticker> {
    let api_key = state.require_api_key().await?;
    tokio::task::spawn_blocking(move || {
        fetch_ticker_blocking(&symbol, &api_key, rest_base_url.as_deref())
    })
    .await
    .map_err(AppError::from)?
}

#[tauri::command]
pub async fn fetch_trades(
    state: State<'_, AppBridge>,
    symbol: String,
    rest_base_url: Option<String>,
    offset: Option<u32>,
    limit: Option<u32>,
    is_trial: Option<bool>,
) -> AppResult<Vec<Trade>> {
    let api_key = state.require_api_key().await?;
    tokio::task::spawn_blocking(move || {
        fetch_trades_blocking(
            &symbol,
            &api_key,
            rest_base_url.as_deref(),
            offset,
            limit,
            is_trial,
        )
    })
    .await
    .map_err(AppError::from)?
}

#[tauri::command]
pub async fn fetch_quote(
    state: State<'_, AppBridge>,
    symbol: String,
    rest_base_url: Option<String>,
) -> AppResult<Quote> {
    let api_key = state.require_api_key().await?;
    tokio::task::spawn_blocking(move || {
        fetch_quote_blocking(&symbol, &api_key, rest_base_url.as_deref())
    })
    .await
    .map_err(AppError::from)?
}

// ─── FutOpt commands ─────────────────────────────────────────────────────────

#[tauri::command]
pub async fn fetch_futopt_ticker(
    state: State<'_, AppBridge>,
    symbol: String,
    rest_base_url: Option<String>,
) -> AppResult<FutOptTicker> {
    let api_key = state.require_api_key().await?;
    tokio::task::spawn_blocking(move || {
        fetch_futopt_ticker_blocking(&symbol, &api_key, rest_base_url.as_deref())
    })
    .await
    .map_err(AppError::from)?
}

#[tauri::command]
pub async fn fetch_futopt_quote(
    state: State<'_, AppBridge>,
    symbol: String,
    rest_base_url: Option<String>,
    after_hours: Option<bool>,
) -> AppResult<FutOptQuote> {
    let api_key = state.require_api_key().await?;
    tokio::task::spawn_blocking(move || {
        fetch_futopt_quote_blocking(
            &symbol,
            &api_key,
            rest_base_url.as_deref(),
            after_hours.unwrap_or(false),
        )
    })
    .await
    .map_err(AppError::from)?
}

#[tauri::command]
pub async fn fetch_futopt_trades(
    state: State<'_, AppBridge>,
    symbol: String,
    rest_base_url: Option<String>,
    offset: Option<i32>,
    limit: Option<i32>,
    after_hours: Option<bool>,
    is_trial: Option<bool>,
) -> AppResult<Vec<Trade>> {
    let api_key = state.require_api_key().await?;
    tokio::task::spawn_blocking(move || {
        fetch_futopt_trades_blocking(
            &symbol,
            &api_key,
            rest_base_url.as_deref(),
            offset,
            limit,
            after_hours.unwrap_or(false),
            is_trial,
        )
    })
    .await
    .map_err(AppError::from)?
}

#[tauri::command]
pub async fn fetch_futopt_candles(
    state: State<'_, AppBridge>,
    symbol: String,
    timeframe: Timeframe,
    rest_base_url: Option<String>,
    after_hours: Option<bool>,
) -> AppResult<Vec<CandleDto>> {
    let api_key = state.require_api_key().await?;
    tokio::task::spawn_blocking(move || {
        fetch_futopt_candles_blocking(
            &symbol,
            timeframe,
            &api_key,
            rest_base_url.as_deref(),
            after_hours.unwrap_or(false),
        )
    })
    .await
    .map_err(AppError::from)?
}

#[tauri::command]
pub async fn fetch_futopt_products(
    state: State<'_, AppBridge>,
    rest_base_url: Option<String>,
) -> AppResult<Vec<Product>> {
    let api_key = state.require_api_key().await?;
    tokio::task::spawn_blocking(move || {
        fetch_futopt_products_blocking(&api_key, rest_base_url.as_deref())
    })
    .await
    .map_err(AppError::from)?
}
