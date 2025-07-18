use axum::extract::multipart::MultipartError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, BackendError>;

#[derive(Debug, Error)]
pub enum BackendError {
    #[error("Invalid input: {0}")]
    BadRequest(String),

    #[error("Internal server error")]
    Internal(#[from] anyhow::Error),

    #[error("Scene not found")]
    NotFound,

    #[error("Multipart error")]
    Multipart(#[from] MultipartError),

    #[error("Tokio IO error")]
    TokioIo(#[from] tokio::io::Error),
    
    #[error("Zip Extract error: {0}")]
    Zip(String),

    #[error("Pipeline error")]
    Pipeline(#[from] pipeline::PipelineError),
}

impl IntoResponse for BackendError {
    fn into_response(self) -> Response {
        let status = match self {
            BackendError::BadRequest(_) => StatusCode::BAD_REQUEST,
            BackendError::NotFound => StatusCode::NOT_FOUND,
            BackendError::Multipart(_) => StatusCode::BAD_REQUEST,
            BackendError::TokioIo(_) => StatusCode::INTERNAL_SERVER_ERROR,
            BackendError::Zip(_) => StatusCode::BAD_REQUEST,
            BackendError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            BackendError::Pipeline(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, self.to_string()).into_response()
    }
}