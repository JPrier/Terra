use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde_json::Value;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Import the shared crates
use aws_sdk_s3::Client as S3Client;
use axum::{serve, Router};
use infrastructure::config::Config;
use infrastructure::s3::S3ManufacturerRepository;
use presentation::handlers::ManufacturerHandlers;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    if std::env::var("AWS_LAMBDA_FUNCTION_NAME").is_ok() {
        lambda_runtime::run(service_fn(function_handler)).await
    } else {
        local_server().await
    }
}

async fn function_handler(event: LambdaEvent<Value>) -> Result<Value, Error> {
    let (event, _context) = event.into_parts();

    tracing::info!("Processing manufacturer API request: {:?}", event);

    // Set up AWS clients
    let region_provider =
        aws_config::meta::region::RegionProviderChain::default_provider().or_else("us-east-1");
    let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .region(region_provider)
        .load()
        .await;

    let s3_client = S3Client::new(&config);

    // Create services
    let app_config = Arc::new(Config::from_env());
    let _manufacturer_repo = Arc::new(S3ManufacturerRepository::new(s3_client, app_config.clone()));

    // For now, return a success response indicating the service is ready
    // Full implementation would parse the Lambda event and route to appropriate handler
    Ok(serde_json::json!({
        "statusCode": 200,
        "headers": {
            "Content-Type": "application/json",
            "Access-Control-Allow-Origin": "*"
        },
        "body": serde_json::json!({
            "message": "Manufacturer API service initialized successfully"
        }).to_string()
    }))
}

async fn local_server() -> Result<(), Error> {
    tracing::info!("Starting manufacturer API server on http://0.0.0.0:3002");

    // Set up AWS clients for local development
    let region_provider =
        aws_config::meta::region::RegionProviderChain::default_provider().or_else("us-east-1");
    let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .region(region_provider)
        .load()
        .await;

    let s3_client = S3Client::new(&config);
    let app_config = Arc::new(Config::from_env());
    let manufacturer_repo = Arc::new(S3ManufacturerRepository::new(s3_client, app_config.clone()));

    // Create router
    let app = Router::new().nest("/v1", ManufacturerHandlers::router(manufacturer_repo));

    // Start server
    let listener = TcpListener::bind("0.0.0.0:3002").await.unwrap();
    serve(listener, app).await.unwrap();

    Ok(())
}
