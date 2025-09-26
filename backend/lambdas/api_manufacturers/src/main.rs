use lambda_runtime::{service_fn, Error, LambdaEvent};
use lambda_web::{is_running_on_lambda, LambdaError, RequestExt};
use serde_json::Value;
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
        lambda_runtime::run(service_fn(function_handler)).await
    } else {
        local_server().await
    }
}

async fn function_handler(event: LambdaEvent<Value>) -> Result<Value, LambdaError> {
    let (event, _context) = event.into_parts();
    
    tracing::info!("Processing manufacturer API request");
    
    // TODO: Implement manufacturer management functionality
    // For now, return a placeholder response
    Ok(serde_json::json!({
        "message": "Manufacturer API not yet implemented"
    }))
}

async fn local_server() -> Result<(), Error> {
    tracing::info!("Starting manufacturer API server on http://0.0.0.0:3002");
    
    // TODO: Implement local development server
    // For now, just run indefinitely
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
}