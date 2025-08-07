//! Error types for the OpenAI CLI application

use thiserror::Error;

/// Main error type for the application
#[derive(Error, Debug)]
pub enum AppError {
    #[error("API key not found. Please set OPENAI_API_KEY environment variable")]
    ApiKeyNotFound,

    #[error("Failed to read configuration: {0}")]
    ConfigError(String),

    #[error("HTTP request failed: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("Failed to parse response: {0}")]
    ParseError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("OpenAI API error: {message}")]
    ApiError { message: String },

    #[error("Invalid model: {0}")]
    InvalidModel(String),

    #[error("Rate limit exceeded. Please try again later")]
    RateLimitExceeded,

    #[error("Response truncated: exceeded maximum token limit")]
    TokenLimitExceeded,
}

/// Result type alias for the application
pub type Result<T> = std::result::Result<T, AppError>;
