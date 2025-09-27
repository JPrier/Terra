use application::services::RfqService;
use aws_sdk_s3::Client as S3Client;
use aws_sdk_sesv2::Client as SesClient;
use infrastructure::{
    config::Config,
    s3::{S3IdempotencyService, S3ManufacturerRepository, S3RfqRepository},
    ses::SesEmailService,
};
use lambda_runtime::{service_fn, Error, LambdaEvent};
use presentation::{handlers::create_app_router, middleware};
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

    // Create services
    let (_rfq_service, _image_service, _manufacturer_repo) = create_services().await?;

    // For now, return a success response
    // Full Axum integration would require more complex lambda-http integration
    Ok(serde_json::json!({
        "statusCode": 200,
        "headers": {
            "Content-Type": "application/json",
            "Access-Control-Allow-Origin": "*"
        },
        "body": serde_json::json!({
            "message": "RFQ API service initialized successfully"
        }).to_string()
    }))
}

async fn local_server() -> Result<(), Error> {
    tracing::info!("Starting RFQ API server on http://0.0.0.0:3001");

    let (rfq_service, image_service, manufacturer_repo) = create_services().await?;

    let app = create_app_router(rfq_service, image_service, manufacturer_repo).layer(
        ServiceBuilder::new()
            .layer(TraceLayer::new_for_http())
            .layer(middleware::cors_layer())
            .layer(axum::middleware::from_fn(middleware::request_id_middleware)),
    );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn create_services() -> Result<
    (
        Arc<RfqService>,
        Arc<dyn application::ports::ImageService + Send + Sync>,
        Arc<dyn application::ports::ManufacturerRepository + Send + Sync>,
    ),
    Error,
> {
    use application::ports::{ImageService, ManufacturerRepository};
    use infrastructure::s3::S3ImageService;
    // Create configuration and AWS clients
    let config = Arc::new(Config::from_env());
    let aws_config = config.create_aws_config().await;
    let s3_client = S3Client::new(&aws_config);
    let ses_client = SesClient::new(&aws_config);

    // Create repositories and services
    let rfq_repository = Arc::new(S3RfqRepository::new(s3_client.clone(), config.clone()));
    let manufacturer_repository = Arc::new(S3ManufacturerRepository::new(
        s3_client.clone(),
        config.clone(),
    ));
    let image_service = Arc::new(S3ImageService::new(s3_client.clone(), config.clone()));
    let idempotency_service = Arc::new(S3IdempotencyService::new(s3_client, config.clone()));

    let from_email =
        std::env::var("FROM_EMAIL").unwrap_or_else(|_| "noreply@terra-platform.com".to_string());
    let email_service = Arc::new(SesEmailService::new(ses_client, config, from_email));

    let rfq_service = RfqService::new(
        rfq_repository.clone(),
        manufacturer_repository.clone(),
        email_service,
        idempotency_service,
    );

    Ok((
        Arc::new(rfq_service),
        image_service as Arc<dyn ImageService + Send + Sync>,
        manufacturer_repository as Arc<dyn ManufacturerRepository + Send + Sync>,
    ))
}
