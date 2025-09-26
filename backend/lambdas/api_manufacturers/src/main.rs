use lambda_runtime::{service_fn, Error, LambdaEvent};
use lambda_web::{is_running_on_lambda, LambdaError, RequestExt};
use serde_json::Value;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use std::sync::Arc;

// Import the shared crates
use application::services::*;
use application::ports::*;
use infrastructure::s3::*;
use infrastructure::ses::*;
use infrastructure::config::*;
use presentation::handlers::ManufacturerHandlers;
use axum::{Router, serve};
use tokio::net::TcpListener;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::Client as S3Client;
use aws_sdk_sesv2::Client as SesClient;

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
        lambda_runtime::run(service_fn(function_handler)).await
    } else {
        local_server().await
    }
}

async fn function_handler(event: LambdaEvent<Value>) -> Result<Value, LambdaError> {
    let (event, _context) = event.into_parts();
    
    tracing::info!("Processing manufacturer API request: {:?}", event);
    
    // Set up AWS clients
    let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
    let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .region(region_provider)
        .load()
        .await;
        
    let s3_client = S3Client::new(&config);
    let ses_client = SesClient::new(&config);
    
    // Create services
    let app_config = Arc::new(Config::from_env());
    let manufacturer_repo = Arc::new(S3ManufacturerRepository::new(s3_client, app_config.clone()));
    
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
    let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
    let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .region(region_provider)
        .load()
        .await;
        
    let s3_client = S3Client::new(&config);
    let app_config = Arc::new(Config::from_env());
    let manufacturer_repo = Arc::new(S3ManufacturerRepository::new(s3_client, app_config.clone()));
    
    // Create router
    let app = Router::new()
        .nest("/v1", ManufacturerHandlers::router(manufacturer_repo));
    
    // Start server
    let listener = TcpListener::bind("0.0.0.0:3002").await.unwrap();
    serve(listener, app).await.unwrap();
    
    Ok(())
}