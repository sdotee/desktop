use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("API error: {0}")]
    Api(String),

    #[error("SDK error: {0}")]
    Sdk(#[from] see_sdk::error::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("QR code generation error: {0}")]
    QrCode(String),

    #[error("Image error: {0}")]
    Image(#[from] image::ImageError),

    #[error("No API key configured")]
    NoApiKey,

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
}

pub type Result<T> = std::result::Result<T, AppError>;
