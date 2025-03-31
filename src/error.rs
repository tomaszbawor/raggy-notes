use ollama_rs::error::OllamaError;
use qdrant_client::QdrantError;
// src/error.rs
use thiserror::Error;

use crate::tui::App;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("I/O error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Ollama API error: {0}")]
    OllamaError(String),

    #[error("Qdrant client error: {0}")]
    QdrantError(#[from] QdrantError),

    #[error("Vector database error: {0}")]
    VectorDBError(String),

    #[error("TUI error: {0}")]
    TUIError(String),

    #[error("Unexpected error: {0}")]
    UnexpectedError(String),
}

// Implement conversion from string and &str to AppError
impl From<String> for AppError {
    fn from(s: String) -> Self {
        AppError::UnexpectedError(s)
    }
}

impl From<&str> for AppError {
    fn from(s: &str) -> Self {
        AppError::UnexpectedError(s.to_string())
    }
}

// Implement conversion from Ollama errors
impl From<ollama_rs::error::InternalOllamaError> for AppError {
    fn from(err: ollama_rs::error::InternalOllamaError) -> Self {
        AppError::OllamaError(err.message)
    }
}

impl From<ollama_rs::error::OllamaError> for AppError {
    fn from(err: ollama_rs::error::OllamaError) -> Self {
        AppError::OllamaError(err.to_string())
    }
}
