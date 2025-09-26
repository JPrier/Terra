use axum::{
    http::StatusCode,
    response::{IntoResponse, Json},
};
use domain::error::DomainError;
use serde_json::json;

/// HTTP error response following the design's error envelope
pub struct AppError {
    pub status: StatusCode,
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

impl AppError {
    pub fn new(status: StatusCode, code: &str, message: &str) -> Self {
        Self {
            status,
            code: code.to_string(),
            message: message.to_string(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let mut body = json!({
            "code": self.code,
            "message": self.message,
        });

        if let Some(details) = self.details {
            body["details"] = details;
        }

        (self.status, Json(body)).into_response()
    }
}

impl From<DomainError> for AppError {
    fn from(err: DomainError) -> Self {
        match err {
            DomainError::ValidationFailed(msg) => AppError::new(
                StatusCode::BAD_REQUEST,
                "validation_error",
                &msg,
            ),
            DomainError::NotFound(msg) => AppError::new(
                StatusCode::NOT_FOUND,
                "not_found",
                &msg,
            ),
            DomainError::Conflict(msg) => AppError::new(
                StatusCode::CONFLICT,
                "conflict",
                &msg,
            ),
            DomainError::InvalidInput(msg) => AppError::new(
                StatusCode::BAD_REQUEST,
                "invalid_input",
                &msg,
            ),
            DomainError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                AppError::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "An internal error occurred",
                )
            },
        }
    }
}

pub type Result<T> = std::result::Result<T, AppError>;