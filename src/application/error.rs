use axum::http::StatusCode;
use thiserror::Error;

use super::ApiErrorResponse;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Sonyflake error: {0}")]
    Sonyflake(#[from] sonyflake::Error),
    #[error("Resource not found")]
    NotFound,
    #[error("Unauthorized access")]
    Unauthorized,
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Internal error: {0}")]
    Internal(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("S3 error: {0}")]
    S3(String),
}

impl From<AppError> for ApiErrorResponse {
    fn from(error: AppError) -> Self {
        match error {
            AppError::Database(_) | AppError::Sonyflake(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong").into()
            }
            AppError::NotFound => (StatusCode::NOT_FOUND, "Not found").into(),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized").into(),
            AppError::BadRequest(message) => (StatusCode::BAD_REQUEST, message).into(),
            AppError::Internal(message) => (StatusCode::INTERNAL_SERVER_ERROR, message).into(),
            AppError::Validation(message) => (StatusCode::BAD_REQUEST, message).into(),
            AppError::Serialization(message) => (StatusCode::INTERNAL_SERVER_ERROR, message).into(),
            AppError::S3(message) => (StatusCode::INTERNAL_SERVER_ERROR, message).into(),
        }
    }
}

impl From<serde_json::Error> for AppError {
    fn from(error: serde_json::Error) -> Self {
        AppError::Serialization(error.to_string())
    }
}
