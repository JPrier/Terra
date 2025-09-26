use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::Json,
    routing::{get, post},
    Router,
};
use application::{
    dto::*,
    services::RfqService,
};
use domain::error::DomainError;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

use crate::error::{AppError, Result};

/// RFQ handlers
pub struct RfqHandlers {
    rfq_service: Arc<RfqService>,
}

impl RfqHandlers {
    pub fn new(rfq_service: Arc<RfqService>) -> Self {
        Self { rfq_service }
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
        let idempotency_key = headers
            .get("idempotency-key")
            .and_then(|h| h.to_str().ok());

        let response = service.create_rfq(request, idempotency_key).await
            .map_err(AppError::from)?;

        Ok(Json(response))
    }

    /// GET /v1/rfqs/{id} - Get RFQ metadata
    async fn get_rfq(
        State(service): State<Arc<RfqService>>,
        Path(rfq_id): Path<String>,
    ) -> Result<Json<RfqMeta>> {
        tracing::info!("Getting RFQ {}", rfq_id);

        let rfq = service.get_rfq(&rfq_id).await
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
        let limit = params.get("limit")
            .and_then(|s| s.parse().ok())
            .map(|l: u32| l.min(200)); // Cap at 200 as per design

        let response = service.list_events(&rfq_id, since, limit).await
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
        let idempotency_key = headers
            .get("idempotency-key")
            .and_then(|h| h.to_str().ok());

        let response = service.post_message(&rfq_id, request, idempotency_key).await
            .map_err(AppError::from)?;

        Ok((StatusCode::CREATED, Json(response)))
    }
}

/// Upload handlers for presigned URLs
pub struct UploadHandlers;

impl UploadHandlers {
    pub fn router() -> Router {
        Router::new()
            .route("/uploads/presign", post(Self::presign_upload))
    }

    /// POST /v1/uploads/presign - Generate presigned upload URL
    async fn presign_upload(
        Json(request): Json<PresignUploadRequest>,
    ) -> Result<Json<PresignUploadResponse>> {
        tracing::info!("Generating presigned URL for tenant {}", request.tenant_id);

        // TODO: Implement upload service integration
        // For now, return a placeholder
        Err(AppError::new(
            StatusCode::NOT_IMPLEMENTED,
            "not_implemented",
            "Upload service not yet implemented",
        ))
    }
}

/// Manufacturer handlers (admin endpoints)
pub struct ManufacturerHandlers;

impl ManufacturerHandlers {
    pub fn router() -> Router {
        Router::new()
            .route("/manufacturers", post(Self::create_manufacturer))
    }

    /// POST /v1/manufacturers - Create/update manufacturer (admin)
    async fn create_manufacturer(
        headers: HeaderMap,
        Json(request): Json<CreateManufacturerRequest>,
    ) -> Result<(StatusCode, Json<CreateManufacturerResponse>)> {
        tracing::info!("Creating/updating manufacturer");

        // TODO: Implement authentication check for Bearer token
        let _auth_header = headers
            .get("authorization")
            .and_then(|h| h.to_str().ok())
            .ok_or_else(|| AppError::new(
                StatusCode::UNAUTHORIZED,
                "unauthorized",
                "Missing authorization header",
            ))?;

        // TODO: Implement manufacturer service integration
        Err(AppError::new(
            StatusCode::NOT_IMPLEMENTED,
            "not_implemented",
            "Manufacturer service not yet implemented",
        ))
    }
}

/// Health check handler
pub async fn health_check() -> &'static str {
    "OK"
}

/// Create the main application router
pub fn create_app_router(rfq_service: Arc<RfqService>) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .nest("/v1", 
            Router::new()
                .merge(RfqHandlers::router(rfq_service))
                .merge(UploadHandlers::router())
                .merge(ManufacturerHandlers::router())
        )
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}