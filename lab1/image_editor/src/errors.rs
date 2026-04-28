use thiserror::Error;

/// Application error type
#[derive(Debug, Error)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Image error: {0}")]
    Image(#[from] image::ImageError),

    #[error("HTTP error: {0}")]
    Http(String),

    #[error("Invalid arguments: {0}")]
    Args(String),

    #[error("Environment variable error: {0}")]
    Env(String),
}
