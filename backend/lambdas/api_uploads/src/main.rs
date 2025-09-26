use aws_config::BehaviorVersion;
use aws_sdk_s3::Client as S3Client;
use infrastructure::{config::Config, s3::S3ImageService};
use lambda_runtime::{service_fn, Error, LambdaEvent};
use lambda_web::{is_running_on_lambda, LambdaError, RequestExt};
use presentation::{handlers::UploadHandlers, middleware};
use serde_json::Value;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    if is_running_on_lambda() {
        // Running on AWS Lambda
        lambda_runtime::run(service_fn(function_handler)).await
    } else {
        // Running locally for development
        local_server().await
    }
}

async fn function_handler(event: LambdaEvent<Value>) -> Result<Value, LambdaError> {
    let (event, _context) = event.into_parts();
    
    // Convert API Gateway event to HTTP request
    let request = event.try_into_request()?;
    
    // Create AWS clients
    let aws_config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let s3_client = S3Client::new(&aws_config);
    let config = Arc::new(Config::from_env());
    
    // Create image service
    let image_service = Arc::new(S3ImageService::new(s3_client, config));
    
    // Create upload handler router
    let app = axum::Router::new()
        .merge(UploadHandlers::router())
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(middleware::cors_layer())
                .layer(axum::middleware::from_fn(middleware::request_id_middleware))
        );
    
    // Process request through Axum
    let response = tower::ServiceExt::oneshot(app, request)
        .await
        .map_err(LambdaError::from)?;
    
    // Convert HTTP response back to Lambda response
    response.try_into_response()
}

async fn local_server() -> Result<(), Error> {
    tracing::info!("Starting uploads API server on http://0.0.0.0:3000");
    
    let app = axum::Router::new()
        .merge(UploadHandlers::router())
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(middleware::cors_layer())
                .layer(axum::middleware::from_fn(middleware::request_id_middleware))
        );
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}