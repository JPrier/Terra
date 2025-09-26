use application::{
    dto::*,
    ports::{ImageService, ManufacturerRepository},
    services::RfqService,
};
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::Json,
    routing::{get, post},
    Router,
};
use domain::entities::*;
use domain::value_objects::*;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

use crate::error::{AppError, Result};

/// RFQ handlers
pub struct RfqHandlers;

impl Default for RfqHandlers {
    fn default() -> Self {
        Self::new()
    }
}

impl RfqHandlers {
    pub fn new() -> Self {
        Self
    }

    pub fn router(rfq_service: Arc<RfqService>) -> Router {
        Router::new()
            .route("/rfqs", post(Self::create_rfq))
            .route("/rfqs/:id", get(Self::get_rfq))
            .route("/rfqs/:id/events", get(Self::list_events))
            .route("/rfqs/:id/messages", post(Self::post_message))
            .with_state(rfq_service)
    }

    /// POST /v1/rfqs - Create a new RFQ
    async fn create_rfq(
        State(service): State<Arc<RfqService>>,
        headers: HeaderMap,
        Json(request): Json<CreateRfqRequest>,
    ) -> Result<Json<CreateRfqResponse>> {
        tracing::info!("Creating RFQ for manufacturer {}", request.manufacturer_id);

        // Extract idempotency key from headers
        let idempotency_key = headers.get("idempotency-key").and_then(|h| h.to_str().ok());

        let response = service
            .create_rfq(request, idempotency_key)
            .await
            .map_err(AppError::from)?;

        Ok(Json(response))
    }

    /// GET /v1/rfqs/{id} - Get RFQ metadata
    async fn get_rfq(
        State(service): State<Arc<RfqService>>,
        Path(rfq_id): Path<String>,
    ) -> Result<Json<RfqMeta>> {
        tracing::info!("Getting RFQ {}", rfq_id);

        let rfq = service
            .get_rfq(&rfq_id)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::new(StatusCode::NOT_FOUND, "not_found", "RFQ not found"))?;

        Ok(Json(rfq))
    }

    /// GET /v1/rfqs/{id}/events - List RFQ events
    async fn list_events(
        State(service): State<Arc<RfqService>>,
        Path(rfq_id): Path<String>,
        Query(params): Query<HashMap<String, String>>,
    ) -> Result<Json<ListEventsResponse>> {
        tracing::info!("Listing events for RFQ {}", rfq_id);

        let since = params.get("since").cloned();
        let limit = params
            .get("limit")
            .and_then(|s| s.parse().ok())
            .map(|l: u32| l.min(200)); // Cap at 200 as per design

        let response = service
            .list_events(&rfq_id, since, limit)
            .await
            .map_err(AppError::from)?;

        Ok(Json(response))
    }

    /// POST /v1/rfqs/{id}/messages - Post a new message
    async fn post_message(
        State(service): State<Arc<RfqService>>,
        Path(rfq_id): Path<String>,
        headers: HeaderMap,
        Json(request): Json<PostMessageRequest>,
    ) -> Result<(StatusCode, Json<PostMessageResponse>)> {
        tracing::info!("Posting message to RFQ {}", rfq_id);

        // Extract idempotency key from headers
        let idempotency_key = headers.get("idempotency-key").and_then(|h| h.to_str().ok());

        let response = service
            .post_message(&rfq_id, request, idempotency_key)
            .await
            .map_err(AppError::from)?;

        Ok((StatusCode::CREATED, Json(response)))
    }
}

/// Upload handlers for presigned URLs
pub struct UploadHandlers;

impl UploadHandlers {
    pub fn router(image_service: Arc<dyn ImageService + Send + Sync>) -> Router {
        Router::new()
            .route("/uploads/presign", post(Self::presign_upload))
            .with_state(image_service)
    }

    /// POST /v1/uploads/presign - Generate presigned upload URL
    async fn presign_upload(
        State(image_service): State<Arc<dyn ImageService + Send + Sync>>,
        Json(request): Json<PresignUploadRequest>,
    ) -> Result<Json<PresignUploadResponse>> {
        tracing::info!("Generating presigned URL for tenant {}", request.tenant_id);

        // Validate and convert request - using simple validation for MVP
        if request.tenant_id.is_empty() {
            return Err(AppError::bad_request("Tenant ID cannot be empty"));
        }

        if request.content_type.is_empty() {
            return Err(AppError::bad_request("Content type cannot be empty"));
        }

        if request.size_bytes == 0 || request.size_bytes > 15 * 1024 * 1024 {
            return Err(AppError::bad_request(
                "File size must be between 1 byte and 15MB",
            ));
        }

        // For MVP, create simple wrappers
        let tenant_id = TenantId::new(request.tenant_id)
            .map_err(|e| AppError::bad_request(&format!("Invalid tenant ID: {}", e)))?;
        let content_type = ContentType::new(request.content_type)
            .map_err(|e| AppError::bad_request(&format!("Invalid content type: {}", e)))?;
        let file_size = FileSize::new(request.size_bytes)
            .map_err(|e| AppError::bad_request(&format!("Invalid file size: {}", e)))?;

        // Generate presigned URL
        let response = image_service
            .generate_presigned_upload_url(&tenant_id, &content_type, &file_size)
            .await
            .map_err(|e| {
                AppError::internal_server_error(&format!("Failed to generate upload URL: {}", e))
            })?;

        Ok(Json(response))
    }
}

/// Manufacturer handlers (admin endpoints)
pub struct ManufacturerHandlers;

impl ManufacturerHandlers {
    pub fn router(manufacturer_repo: Arc<dyn ManufacturerRepository + Send + Sync>) -> Router {
        Router::new()
            .route("/manufacturers", post(Self::create_manufacturer))
            .with_state(manufacturer_repo)
    }

    /// POST /v1/manufacturers - Create/update manufacturer (admin)
    async fn create_manufacturer(
        State(manufacturer_repo): State<Arc<dyn ManufacturerRepository + Send + Sync>>,
        headers: HeaderMap,
        Json(request): Json<CreateManufacturerRequest>,
    ) -> Result<(StatusCode, Json<CreateManufacturerResponse>)> {
        tracing::info!(
            "Creating/updating manufacturer for tenant {}",
            request.tenant_id
        );

        // Check for authentication header (simplified for MVP)
        let _auth_header = headers
            .get("authorization")
            .and_then(|h| h.to_str().ok())
            .ok_or_else(|| {
                AppError::new(
                    StatusCode::UNAUTHORIZED,
                    "unauthorized",
                    "Missing authorization header",
                )
            })?;

        // Convert DTO to domain entity
        let manufacturer = Manufacturer {
            id: ManufacturerId::generate().as_str().to_string(),
            tenant_id: request.tenant_id.clone(),
            name: request.name,
            description: request.description,
            location: request.location.map(|loc| Location {
                city: loc.city,
                state: loc.state,
                country: loc.country,
                lat: loc.lat,
                lng: loc.lng,
            }),
            categories: request.capabilities.clone(), // Using capabilities as categories for now
            capabilities: Some(request.capabilities),
            contact_email: Some(request.contact_email),
            media: None, // Will be populated later via image uploads
            updated_at: chrono::Utc::now(),
        };

        // Save manufacturer - convert to ManufacturerProfile first
        let profile = ManufacturerProfile {
            id: manufacturer.id.clone(),
            tenant_id: manufacturer.tenant_id.clone(),
            name: manufacturer.name.clone(),
            description: manufacturer.description.clone(),
            location: manufacturer.location.clone(),
            categories: manufacturer.categories.clone(),
            capabilities: manufacturer.capabilities.clone(),
            contact_email: manufacturer.contact_email.clone(),
            media: manufacturer.media.clone(),
            offerings: None, // Will be populated later
            updated_at: manufacturer.updated_at,
        };

        manufacturer_repo
            .save_manufacturer(&profile)
            .await
            .map_err(|e| {
                AppError::internal_server_error(&format!("Failed to create manufacturer: {}", e))
            })?;

        let response = CreateManufacturerResponse {
            id: manufacturer.id.clone(),
            tenant_id: manufacturer.tenant_id.clone(),
        };

        Ok((StatusCode::CREATED, Json(response)))
    }
}

/// Health check handler
pub async fn health_check() -> &'static str {
    "OK"
}

/// Create the main application router
pub fn create_app_router(
    rfq_service: Arc<RfqService>,
    image_service: Arc<dyn ImageService + Send + Sync>,
    manufacturer_repo: Arc<dyn ManufacturerRepository + Send + Sync>,
) -> Router {
    Router::new().route("/health", get(health_check)).nest(
        "/v1",
        Router::new()
            .merge(RfqHandlers::router(rfq_service))
            .merge(UploadHandlers::router(image_service))
            .merge(ManufacturerHandlers::router(manufacturer_repo)),
    )
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}
