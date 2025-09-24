//! Terra Digital Marketplace Backend
//! 
//! A serverless backend API for the Terra digital marketplace built with Rust
//! and designed to run on AWS Lambda.

pub mod models;
pub mod handlers;
pub mod services;
pub mod utils;
pub mod auth;
pub mod db;
pub mod config;
pub mod errors;

// Re-export commonly used types
pub use models::*;
pub use errors::{ApiError, Result};
pub use config::Config;
pub use auth::{Claims, AuthService};

use serde::{Deserialize, Serialize};

/// Standard API response wrapper
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Pagination parameters
#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: Some(1),
            limit: Some(20),
        }
    }
}

/// Common filter parameters
#[derive(Debug, Deserialize)]
pub struct FilterParams {
    pub category: Option<String>,
    pub min_price: Option<f64>,
    pub max_price: Option<f64>,
    pub search: Option<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

/// Health check response
#[derive(Debug, Serialize)]
pub struct HealthCheck {
    pub status: String,
    pub version: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl HealthCheck {
    pub fn new() -> Self {
        Self {
            status: "healthy".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Initialize tracing for Lambda functions
pub fn init_tracing() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_response_success() {
        let response = ApiResponse::success("test data".to_string());
        assert!(response.success);
        assert_eq!(response.data, Some("test data".to_string()));
        assert!(response.error.is_none());
    }

    #[test]
    fn test_api_response_error() {
        let response: ApiResponse<String> = ApiResponse::error("test error".to_string());
        assert!(!response.success);
        assert!(response.data.is_none());
        assert_eq!(response.error, Some("test error".to_string()));
    }

    #[test]
    fn test_health_check() {
        let health = HealthCheck::new();
        assert_eq!(health.status, "healthy");
        assert_eq!(health.version, env!("CARGO_PKG_VERSION"));
    }
}