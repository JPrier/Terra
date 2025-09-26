use axum::{
    body::Body,
    extract::Request,
    http::{HeaderMap, HeaderValue, Method},
    middleware::Next,
    response::Response,
};
use tower_http::cors::{CorsLayer, Any};
use uuid::Uuid;

/// Add request ID to all requests
pub async fn request_id_middleware(mut request: Request, next: Next) -> Response {
    let request_id = Uuid::new_v4().to_string();
    
    // Add request ID to headers for downstream handlers
    request.headers_mut().insert(
        "x-request-id",
        HeaderValue::from_str(&request_id).unwrap(),
    );
    
    let mut response = next.run(request).await;
    
    // Add request ID to response headers
    response.headers_mut().insert(
        "x-request-id",
        HeaderValue::from_str(&request_id).unwrap(),
    );
    
    response
}

/// Rate limiting middleware placeholder
pub async fn rate_limit_middleware(request: Request, next: Next) -> Response {
    // TODO: Implement actual rate limiting with API Gateway usage plans
    // For now, just pass through
    next.run(request).await
}

/// CORS configuration for the API
pub fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any) // TODO: Restrict to specific GitHub Pages domains
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([
            "content-type",
            "authorization", 
            "idempotency-key",
            "x-request-id",
            "if-none-match",
        ])
        .expose_headers([
            "x-request-id",
            "etag",
        ])
}