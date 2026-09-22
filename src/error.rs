use thiserror::Error;

#[derive(Debug, Error)]
pub enum BotError {
    #[error("telegram hatasi: {0}")]
    Telegram(#[from] teloxide::RequestError),

    #[error("istek hatasi: {0}")]
    Request(#[from] reqwest::Error),

    #[error("io hatasi: {0}")]
    Io(#[from] std::io::Error),

    #[error("json hatasi: {0}")]
    Json(#[from] serde_json::Error),

    #[error("veritabani hatasi: {0}")]
    Db(#[from] libsql::Error),

    #[error("dosya bulunamadi: {0}")]
    NotFound(String),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, BotError>;
