//! HTTP handlers for the Terra marketplace API

pub mod products;
pub mod users;
pub mod orders;
pub mod auth;

// Common handler utilities
use axum::{extract::Query, http::HeaderMap};
use crate::{ApiError, Result, PaginationParams, FilterParams, auth::Claims};

/// Extract pagination parameters from query string
pub fn extract_pagination(Query(params): Query<PaginationParams>) -> PaginationParams {
    params
}

/// Extract filter parameters from query string
pub fn extract_filters(Query(params): Query<FilterParams>) -> FilterParams {
    params
}

/// Extract authorization claims from headers
pub fn extract_auth_claims(headers: &HeaderMap, auth_service: &crate::AuthService) -> Result<Claims> {
    let auth_header = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| ApiError::AuthenticationFailed("Missing authorization header".to_string()))?;

    crate::auth::require_auth(Some(auth_header), auth_service)
}

/// Extract optional authorization claims from headers
pub fn extract_optional_auth_claims(headers: &HeaderMap, auth_service: &crate::AuthService) -> Option<Claims> {
    headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|auth_header| crate::auth::require_auth(Some(auth_header), auth_service).ok())
}