use serde::Serialize;
use thiserror::Error;

use marketdata_core::MarketDataError;

#[derive(Debug, Error, Serialize)]
#[serde(tag = "kind", content = "message", rename_all = "snake_case")]
pub enum AppError {
    #[error("SDK error: {0}")]
    Sdk(String),

    #[error("not connected — call connect first")]
    NotConnected,

    #[error("API key not configured")]
    MissingApiKey,

    #[error("internal: {0}")]
    Other(String),
}

impl From<MarketDataError> for AppError {
    fn from(e: MarketDataError) -> Self {
        AppError::Sdk(e.to_string())
    }
}

impl From<tokio::task::JoinError> for AppError {
    fn from(e: tokio::task::JoinError) -> Self {
        AppError::Other(format!("blocking task join failed: {e}"))
    }
}

pub type AppResult<T> = Result<T, AppError>;
