//! Application error type. Serializes to a plain string so `invoke` rejects
//! with a readable message on the frontend.

use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Unsupported file: {0}")]
    Unsupported(String),
    #[error("Could not decode image: {0}")]
    Decode(#[from] image::ImageError),
    #[error("Encoding failed: {0}")]
    Encode(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("A conversion is already running")]
    Busy,
    #[error("Invalid parameters: {0}")]
    Invalid(String),
    #[error("The image processor panicked")]
    Panic,
}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<zip::result::ZipError> for AppError {
    fn from(e: zip::result::ZipError) -> Self {
        match e {
            zip::result::ZipError::Io(io) => AppError::Io(io),
            other => AppError::Encode(other.to_string()),
        }
    }
}

impl From<jpeg_encoder::EncodingError> for AppError {
    fn from(e: jpeg_encoder::EncodingError) -> Self {
        AppError::Encode(e.to_string())
    }
}

impl From<oxipng::PngError> for AppError {
    fn from(e: oxipng::PngError) -> Self {
        AppError::Encode(e.to_string())
    }
}

impl From<ravif::Error> for AppError {
    fn from(e: ravif::Error) -> Self {
        AppError::Encode(e.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
