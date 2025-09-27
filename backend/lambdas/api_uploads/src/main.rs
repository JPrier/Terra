use aws_sdk_s3::Client as S3Client;
use infrastructure::{config::Config, s3::S3ImageService};
use lambda_runtime::{service_fn, Error, LambdaEvent};
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

    if std::env::var("AWS_LAMBDA_FUNCTION_NAME").is_ok() {
        // Running on AWS Lambda
        lambda_runtime::run(service_fn(function_handler)).await
    } else {
        // Running locally for development
        local_server().await
    }
}

async fn function_handler(event: LambdaEvent<Value>) -> Result<Value, Error> {
    let (_event, _context) = event.into_parts();

    // Create AWS clients
    let config = Arc::new(Config::from_env());
    let aws_config = config.create_aws_config().await;
    let s3_client = S3Client::new(&aws_config);

    // Create image service
    let _image_service = Arc::new(S3ImageService::new(s3_client, config));

    // For now, return a success response
    // Full Axum integration would require more complex lambda-http integration
    Ok(serde_json::json!({
        "statusCode": 200,
        "headers": {
            "Content-Type": "application/json",
            "Access-Control-Allow-Origin": "*"
        },
        "body": serde_json::json!({
            "message": "Upload API service initialized successfully"
        }).to_string()
    }))
}

async fn local_server() -> Result<(), Error> {
    tracing::info!("Starting uploads API server on http://0.0.0.0:3000");

    // Create AWS clients for local development
    let config = Arc::new(Config::from_env());
    let aws_config = config.create_aws_config().await;
    let s3_client = S3Client::new(&aws_config);
    let image_service = Arc::new(S3ImageService::new(s3_client, config));

    let app = axum::Router::new()
        .merge(UploadHandlers::router(image_service))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(middleware::cors_layer())
                .layer(axum::middleware::from_fn(middleware::request_id_middleware)),
        );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
