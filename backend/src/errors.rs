//! Error handling for the Terra marketplace backend

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, ApiError>;

/// API Error types
#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Authorization failed: {0}")]
    AuthorizationFailed(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Resource already exists: {0}")]
    AlreadyExists(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("External service error: {0}")]
    ExternalServiceError(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Payment processing error: {0}")]
    PaymentError(String),

    #[error("Insufficient stock for product")]
    InsufficientStock,

    #[error("Invalid file format: {0}")]
    InvalidFileFormat(String),

    #[error("File too large: maximum size is {max_size} bytes")]
    FileTooLarge { max_size: usize },

    #[error("Internal server error: {0}")]
    InternalServer(String),

    #[error("Bad request: {0}")]
    BadRequest(String),
}

impl ApiError {
    /// Get the HTTP status code for this error
    pub fn status_code(&self) -> StatusCode {
        match self {
            ApiError::AuthenticationFailed(_) => StatusCode::UNAUTHORIZED,
            ApiError::AuthorizationFailed(_) => StatusCode::FORBIDDEN,
            ApiError::ValidationError(_) => StatusCode::BAD_REQUEST,
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::AlreadyExists(_) => StatusCode::CONFLICT,
            ApiError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::ExternalServiceError(_) => StatusCode::BAD_GATEWAY,
            ApiError::ConfigurationError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::RateLimitExceeded => StatusCode::TOO_MANY_REQUESTS,
            ApiError::PaymentError(_) => StatusCode::PAYMENT_REQUIRED,
            ApiError::InsufficientStock => StatusCode::BAD_REQUEST,
            ApiError::InvalidFileFormat(_) => StatusCode::BAD_REQUEST,
            ApiError::FileTooLarge { .. } => StatusCode::PAYLOAD_TOO_LARGE,
            ApiError::InternalServer(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
        }
    }

    /// Get error code for client identification
    pub fn error_code(&self) -> &'static str {
        match self {
            ApiError::AuthenticationFailed(_) => "AUTHENTICATION_FAILED",
            ApiError::AuthorizationFailed(_) => "AUTHORIZATION_FAILED",
            ApiError::ValidationError(_) => "VALIDATION_ERROR",
            ApiError::NotFound(_) => "NOT_FOUND",
            ApiError::AlreadyExists(_) => "ALREADY_EXISTS",
            ApiError::DatabaseError(_) => "DATABASE_ERROR",
            ApiError::ExternalServiceError(_) => "EXTERNAL_SERVICE_ERROR",
            ApiError::ConfigurationError(_) => "CONFIGURATION_ERROR",
            ApiError::RateLimitExceeded => "RATE_LIMIT_EXCEEDED",
            ApiError::PaymentError(_) => "PAYMENT_ERROR",
            ApiError::InsufficientStock => "INSUFFICIENT_STOCK",
            ApiError::InvalidFileFormat(_) => "INVALID_FILE_FORMAT",
            ApiError::FileTooLarge { .. } => "FILE_TOO_LARGE",
            ApiError::InternalServer(_) => "INTERNAL_SERVER_ERROR",
            ApiError::BadRequest(_) => "BAD_REQUEST",
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let error_code = self.error_code();
        let message = self.to_string();

        tracing::error!("API Error: {} - {}", error_code, message);

        let body = Json(json!({
            "success": false,
            "error": {
                "code": error_code,
                "message": message,
                "timestamp": chrono::Utc::now()
            }
        }));

        (status, body).into_response()
    }
}

// Convert common error types to ApiError
impl From<aws_sdk_dynamodb::Error> for ApiError {
    fn from(err: aws_sdk_dynamodb::Error) -> Self {
        ApiError::DatabaseError(err.to_string())
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        ApiError::ValidationError(format!("JSON parsing error: {}", err))
    }
}

impl From<validator::ValidationErrors> for ApiError {
    fn from(err: validator::ValidationErrors) -> Self {
        let messages: Vec<String> = err
            .field_errors()
            .iter()
            .flat_map(|(field, errors)| {
                errors.iter().map(move |error| {
                    format!("{}: {}", field, error.message.as_ref().map_or("Invalid value", |m| m))
                })
            })
            .collect();
        
        ApiError::ValidationError(messages.join(", "))
    }
}

impl From<jsonwebtoken::errors::Error> for ApiError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        ApiError::AuthenticationFailed(format!("JWT error: {}", err))
    }
}

impl From<bcrypt::BcryptError> for ApiError {
    fn from(err: bcrypt::BcryptError) -> Self {
        ApiError::InternalServer(format!("Password hashing error: {}", err))
    }
}

impl From<reqwest::Error> for ApiError {
    fn from(err: reqwest::Error) -> Self {
        ApiError::ExternalServiceError(format!("HTTP client error: {}", err))
    }
}

impl From<std::env::VarError> for ApiError {
    fn from(err: std::env::VarError) -> Self {
        ApiError::ConfigurationError(format!("Environment variable error: {}", err))
    }
}

// Helper functions for common error scenarios
impl ApiError {
    pub fn product_not_found(id: uuid::Uuid) -> Self {
        ApiError::NotFound(format!("Product with ID {} not found", id))
    }

    pub fn user_not_found(id: uuid::Uuid) -> Self {
        ApiError::NotFound(format!("User with ID {} not found", id))
    }

    pub fn order_not_found(id: uuid::Uuid) -> Self {
        ApiError::NotFound(format!("Order with ID {} not found", id))
    }

    pub fn invalid_credentials() -> Self {
        ApiError::AuthenticationFailed("Invalid email or password".to_string())
    }

    pub fn token_expired() -> Self {
        ApiError::AuthenticationFailed("Authentication token has expired".to_string())
    }

    pub fn insufficient_permissions() -> Self {
        ApiError::AuthorizationFailed("Insufficient permissions for this operation".to_string())
    }

    pub fn email_already_exists() -> Self {
        ApiError::AlreadyExists("User with this email already exists".to_string())
    }

    pub fn username_already_exists() -> Self {
        ApiError::AlreadyExists("User with this username already exists".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_status_codes() {
        assert_eq!(ApiError::ValidationError("test".to_string()).status_code(), StatusCode::BAD_REQUEST);
        assert_eq!(ApiError::NotFound("test".to_string()).status_code(), StatusCode::NOT_FOUND);
        assert_eq!(ApiError::AuthenticationFailed("test".to_string()).status_code(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_error_codes() {
        assert_eq!(ApiError::ValidationError("test".to_string()).error_code(), "VALIDATION_ERROR");
        assert_eq!(ApiError::NotFound("test".to_string()).error_code(), "NOT_FOUND");
        assert_eq!(ApiError::AuthenticationFailed("test".to_string()).error_code(), "AUTHENTICATION_FAILED");
    }

    #[test]
    fn test_helper_functions() {
        let id = uuid::Uuid::new_v4();
        let error = ApiError::product_not_found(id);
        match error {
            ApiError::NotFound(msg) => assert!(msg.contains(&id.to_string())),
            _ => panic!("Expected NotFound error"),
        }
    }
}