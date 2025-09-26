use async_trait::async_trait;
use aws_sdk_s3::Client as S3Client;
use aws_sdk_s3::presigning::PresigningConfig;
use chrono::{DateTime, Utc};
use domain::entities::*;
use domain::events::*;
use domain::value_objects::*;
use domain::error::{DomainError, Result};
use application::ports::*;
use std::sync::Arc;
use sha2::Digest;

use crate::config::Config;

/// S3-based RFQ repository (simplified for MVP)
pub struct S3RfqRepository {
    client: S3Client,
    config: Arc<Config>,
}

impl S3RfqRepository {
    pub fn new(client: S3Client, config: Arc<Config>) -> Self {
        Self { client, config }
    }
}

#[async_trait]
impl RfqRepository for S3RfqRepository {
    async fn save_rfq_meta(&self, rfq: &RfqMeta) -> Result<()> {
        let key = format!("rfq/{}/meta.json", rfq.id);
        let body = serde_json::to_string(rfq)
            .map_err(|e| DomainError::Internal(format!("Failed to serialize RFQ meta: {}", e)))?;

        let _result = self.client
            .put_object()
            .bucket(&self.config.private_bucket)
            .key(&key)
            .body(body.into_bytes().into())
            .content_type("application/json")
            .send()
            .await
            .map_err(|e| DomainError::Internal(format!("Failed to save RFQ meta: {}", e)))?;

        Ok(())
    }

    async fn get_rfq_meta(&self, id: &RfqId) -> Result<Option<RfqMeta>> {
        let key = format!("rfq/{}/meta.json", id.as_str());
        
        match self.client
            .get_object()
            .bucket(&self.config.private_bucket)
            .key(&key)
            .send()
            .await
        {
            Ok(response) => {
                let body = response.body.collect().await
                    .map_err(|e| DomainError::Internal(format!("Failed to read RFQ meta body: {}", e)))?
                    .into_bytes();
                
                let rfq_meta: RfqMeta = serde_json::from_slice(&body)
                    .map_err(|e| DomainError::Internal(format!("Failed to deserialize RFQ meta: {}", e)))?;
                
                Ok(Some(rfq_meta))
            }
            Err(e) => {
                // Check if it's a NoSuchKey error, which is expected for non-existent RFQs
                if e.to_string().contains("NoSuchKey") {
                    Ok(None)
                } else {
                    Err(DomainError::Internal(format!("Failed to fetch RFQ meta: {}", e)))
                }
            }
        }
    }

    async fn save_rfq_index(&self, rfq_id: &RfqId, index: &RfqIndex) -> Result<()> {
        let key = format!("rfq/{}/index.json", rfq_id.as_str());
        let body = serde_json::to_string(index)
            .map_err(|e| DomainError::Internal(format!("Failed to serialize RFQ index: {}", e)))?;

        let _result = self.client
            .put_object()
            .bucket(&self.config.private_bucket)
            .key(&key)
            .body(body.into_bytes().into())
            .content_type("application/json")
            .send()
            .await
            .map_err(|e| DomainError::Internal(format!("Failed to save RFQ index: {}", e)))?;

        Ok(())
    }

    async fn get_rfq_index(&self, id: &RfqId) -> Result<Option<RfqIndex>> {
        let key = format!("rfq/{}/index.json", id.as_str());
        
        match self.client
            .get_object()
            .bucket(&self.config.private_bucket)
            .key(&key)
            .send()
            .await
        {
            Ok(response) => {
                let body = response.body.collect().await
                    .map_err(|e| DomainError::Internal(format!("Failed to read RFQ index body: {}", e)))?
                    .into_bytes();
                
                let rfq_index: RfqIndex = serde_json::from_slice(&body)
                    .map_err(|e| DomainError::Internal(format!("Failed to deserialize RFQ index: {}", e)))?;
                
                Ok(Some(rfq_index))
            }
            Err(e) => {
                if e.to_string().contains("NoSuchKey") {
                    Ok(None)
                } else {
                    Err(DomainError::Internal(format!("Failed to fetch RFQ index: {}", e)))
                }
            }
        }
    }

    async fn save_rfq_event(&self, event: &RfqEvent) -> Result<()> {
        let ts_str = event.timestamp().format("%Y-%m-%dT%H-%M-%SZ").to_string();
        let key = format!("rfq/{}/events/{}-{}.json", event.rfq_id(), ts_str, event.id());
        let body = serde_json::to_string(event)
            .map_err(|e| DomainError::Internal(format!("Failed to serialize RFQ event: {}", e)))?;

        let _result = self.client
            .put_object()
            .bucket(&self.config.private_bucket)
            .key(&key)
            .body(body.into_bytes().into())
            .content_type("application/json")
            .send()
            .await
            .map_err(|e| DomainError::Internal(format!("Failed to save RFQ event: {}", e)))?;

        Ok(())
    }

    async fn list_rfq_events(&self, rfq_id: &RfqId, since: Option<DateTime<Utc>>, limit: Option<u32>) -> Result<Vec<RfqEvent>> {
        let prefix = format!("rfq/{}/events/", rfq_id.as_str());
        let limit = limit.unwrap_or(100).min(1000); // Reasonable upper bound
        
        let list_response = self.client
            .list_objects_v2()
            .bucket(&self.config.private_bucket)
            .prefix(&prefix)
            .max_keys(limit as i32)
            .send()
            .await
            .map_err(|e| DomainError::Internal(format!("Failed to list RFQ events: {}", e)))?;

        let mut events = Vec::new();
        
        if let Some(objects) = list_response.contents {
            for object in objects {
                if let Some(key) = object.key {
                    // Try to get the object
                    match self.client
                        .get_object()
                        .bucket(&self.config.private_bucket)
                        .key(&key)
                        .send()
                        .await
                    {
                        Ok(response) => {
                            let body = response.body.collect().await
                                .map_err(|e| DomainError::Internal(format!("Failed to read event body: {}", e)))?
                                .into_bytes();
                            
                            match serde_json::from_slice::<RfqEvent>(&body) {
                                Ok(event) => {
                                    // Filter by since timestamp if provided
                                    if let Some(since_time) = since {
                                        if event.timestamp() < since_time {
                                            continue;
                                        }
                                    }
                                    events.push(event);
                                }
                                Err(e) => {
                                    // Log the error but continue processing other events
                                    tracing::warn!("Failed to deserialize event {}: {}", key, e);
                                }
                            }
                        }
                        Err(e) => {
                            // Log the error but continue processing other events
                            tracing::warn!("Failed to fetch event {}: {}", key, e);
                        }
                    }
                }
            }
        }

        // Sort by timestamp (events should be in order, but ensure it)
        events.sort_by_key(|e| e.timestamp());
        
        Ok(events)
    }
}

/// S3-based manufacturer repository (simplified for MVP)
pub struct S3ManufacturerRepository {
    client: S3Client,
    config: Arc<Config>,
}

impl S3ManufacturerRepository {
    pub fn new(client: S3Client, config: Arc<Config>) -> Self {
        Self { client, config }
    }
}

#[async_trait]
impl ManufacturerRepository for S3ManufacturerRepository {
    async fn save_manufacturer(&self, manufacturer: &ManufacturerProfile) -> Result<()> {
        let key = format!("manufacturer/{}.json", manufacturer.id);
        let body = serde_json::to_string(manufacturer)
            .map_err(|e| DomainError::Internal(format!("Failed to serialize manufacturer: {}", e)))?;

        let _result = self.client
            .put_object()
            .bucket(&self.config.public_bucket)
            .key(&key)
            .body(body.into_bytes().into())
            .content_type("application/json")
            .send()
            .await
            .map_err(|e| DomainError::Internal(format!("Failed to save manufacturer: {}", e)))?;

        Ok(())
    }

    async fn get_manufacturer(&self, id: &ManufacturerId) -> Result<Option<ManufacturerProfile>> {
        let key = format!("manufacturer/{}.json", id.as_str());
        
        match self.client
            .get_object()
            .bucket(&self.config.public_bucket)
            .key(&key)
            .send()
            .await
        {
            Ok(response) => {
                let body = response.body.collect().await
                    .map_err(|e| DomainError::Internal(format!("Failed to read manufacturer body: {}", e)))?
                    .into_bytes();
                
                let manufacturer: ManufacturerProfile = serde_json::from_slice(&body)
                    .map_err(|e| DomainError::Internal(format!("Failed to deserialize manufacturer: {}", e)))?;
                
                Ok(Some(manufacturer))
            }
            Err(e) => {
                if e.to_string().contains("NoSuchKey") {
                    Ok(None)
                } else {
                    Err(DomainError::Internal(format!("Failed to fetch manufacturer: {}", e)))
                }
            }
        }
    }

    async fn delete_manufacturer(&self, id: &ManufacturerId) -> Result<()> {
        let key = format!("manufacturer/{}.json", id.as_str());
        
        self.client
            .delete_object()
            .bucket(&self.config.public_bucket)
            .key(&key)
            .send()
            .await
            .map_err(|e| DomainError::Internal(format!("Failed to delete manufacturer: {}", e)))?;

        Ok(())
    }
}

/// S3-based catalog repository (simplified for MVP)  
pub struct S3CatalogRepository {
    client: S3Client,
    config: Arc<Config>,
}

impl S3CatalogRepository {
    pub fn new(client: S3Client, config: Arc<Config>) -> Self {
        Self { client, config }
    }
}

#[async_trait]
impl CatalogRepository for S3CatalogRepository {
    async fn save_category_slice(&self, slice: &CategorySlice) -> Result<()> {
        let key = format!("catalog/category/{}.json", slice.category);
        let body = serde_json::to_string(slice)
            .map_err(|e| DomainError::Internal(format!("Failed to serialize category slice: {}", e)))?;

        let _result = self.client
            .put_object()
            .bucket(&self.config.public_bucket)
            .key(&key)
            .body(body.into_bytes().into())
            .content_type("application/json")
            .send()
            .await
            .map_err(|e| DomainError::Internal(format!("Failed to save category slice: {}", e)))?;

        Ok(())
    }

    async fn get_category_slice(&self, category: &str) -> Result<Option<CategorySlice>> {
        let key = format!("catalog/category/{}.json", category);
        
        match self.client
            .get_object()
            .bucket(&self.config.public_bucket)
            .key(&key)
            .send()
            .await
        {
            Ok(response) => {
                let body = response.body.collect().await
                    .map_err(|e| DomainError::Internal(format!("Failed to read category slice body: {}", e)))?
                    .into_bytes();
                
                let slice: CategorySlice = serde_json::from_slice(&body)
                    .map_err(|e| DomainError::Internal(format!("Failed to deserialize category slice: {}", e)))?;
                
                Ok(Some(slice))
            }
            Err(e) => {
                if e.to_string().contains("NoSuchKey") {
                    Ok(None)
                } else {
                    Err(DomainError::Internal(format!("Failed to fetch category slice: {}", e)))
                }
            }
        }
    }

    async fn save_category_state_slice(&self, category: &str, state: &str, slice: &CategorySlice) -> Result<()> {
        let key = format!("catalog/category_state/{}/{}.json", category, state);
        let body = serde_json::to_string(slice)
            .map_err(|e| DomainError::Internal(format!("Failed to serialize category state slice: {}", e)))?;

        let _result = self.client
            .put_object()
            .bucket(&self.config.public_bucket)
            .key(&key)
            .body(body.into_bytes().into())
            .content_type("application/json")
            .send()
            .await
            .map_err(|e| DomainError::Internal(format!("Failed to save category state slice: {}", e)))?;

        Ok(())
    }

    async fn get_category_state_slice(&self, category: &str, state: &str) -> Result<Option<CategorySlice>> {
        let key = format!("catalog/category_state/{}/{}.json", category, state);
        
        match self.client
            .get_object()
            .bucket(&self.config.public_bucket)
            .key(&key)
            .send()
            .await
        {
            Ok(response) => {
                let body = response.body.collect().await
                    .map_err(|e| DomainError::Internal(format!("Failed to read category state slice body: {}", e)))?
                    .into_bytes();
                
                let slice: CategorySlice = serde_json::from_slice(&body)
                    .map_err(|e| DomainError::Internal(format!("Failed to deserialize category state slice: {}", e)))?;
                
                Ok(Some(slice))
            }
            Err(e) => {
                if e.to_string().contains("NoSuchKey") {
                    Ok(None)
                } else {
                    Err(DomainError::Internal(format!("Failed to fetch category state slice: {}", e)))
                }
            }
        }
    }
}

/// S3-based image service (simplified for MVP)
pub struct S3ImageService {
    client: S3Client,
    config: Arc<Config>,
}

impl S3ImageService {
    pub fn new(client: S3Client, config: Arc<Config>) -> Self {
        Self { client, config }
    }
}

#[async_trait]
impl ImageService for S3ImageService {
    async fn generate_presigned_upload_url(&self, tenant_id: &TenantId, content_type: &ContentType, size: &FileSize) -> Result<application::dto::PresignUploadResponse> {
        // Generate unique key for upload
        let file_id = uuid::Uuid::new_v4();
        let extension = match content_type.as_str() {
            "image/jpeg" => "jpg",
            "image/png" => "png", 
            "image/webp" => "webp",
            "image/avif" => "avif",
            "application/pdf" => "pdf",
            _ => "bin",
        };
        let key = format!("tenants/{}/images/raw/{}.{}", tenant_id.as_str(), file_id, extension);
        
        // Generate presigned URL with 10-minute expiration
        let presign_config = PresigningConfig::builder()
            .expires_in(std::time::Duration::from_secs(600))
            .build()
            .map_err(|e| DomainError::Internal(format!("Presign config error: {}", e)))?;
            
        let presigned_request = self.client
            .put_object()
            .bucket(&self.config.private_bucket)
            .key(&key)
            .content_type(content_type.as_str())
            .content_length(size.as_u64() as i64)
            .metadata("tenant", tenant_id.as_str())
            .presigned(presign_config)
            .await
            .map_err(|e| DomainError::Internal(format!("Failed to generate presigned URL: {}", e)))?;
        
        Ok(application::dto::PresignUploadResponse {
            url: presigned_request.uri().to_string(),
            key: key.clone(),
            expires_in: 600,
        })
    }

    async fn save_image_manifest(&self, manifest: &ImageManifest) -> Result<()> {
        // For MVP, derive tenant from a shared pool since ImageManifest doesn't have tenant_id
        let key = format!("tenants/shared/manifests/{}.json", manifest.id);
        let body = serde_json::to_vec(manifest)
            .map_err(|e| DomainError::Internal(format!("Failed to serialize manifest: {}", e)))?;
            
        self.client
            .put_object()
            .bucket(&self.config.public_bucket)
            .key(&key)
            .body(body.into())
            .content_type("application/json")
            .cache_control("public, max-age=31536000, immutable")
            .send()
            .await
            .map_err(|e| DomainError::Internal(format!("Failed to save manifest: {}", e)))?;
            
        Ok(())
    }

    async fn get_image_manifest(&self, id: &str) -> Result<Option<ImageManifest>> {
        let key = format!("tenants/shared/manifests/{}.json", id);
        
        match self.client
            .get_object()
            .bucket(&self.config.public_bucket)
            .key(&key)
            .send()
            .await
        {
            Ok(response) => {
                let body = response.body.collect().await
                    .map_err(|e| DomainError::Internal(format!("Failed to read image manifest body: {}", e)))?
                    .into_bytes();
                
                let manifest: ImageManifest = serde_json::from_slice(&body)
                    .map_err(|e| DomainError::Internal(format!("Failed to deserialize image manifest: {}", e)))?;
                
                Ok(Some(manifest))
            }
            Err(e) => {
                if e.to_string().contains("NoSuchKey") {
                    Ok(None)
                } else {
                    Err(DomainError::Internal(format!("Failed to fetch image manifest: {}", e)))
                }
            }
        }
    }
}

/// S3-based idempotency service (simplified for MVP)
pub struct S3IdempotencyService {
    client: S3Client,
    config: Arc<Config>,
}

impl S3IdempotencyService {
    pub fn new(client: S3Client, config: Arc<Config>) -> Self {
        Self { client, config }
    }

    fn idempotency_key(&self, key: &str) -> String {
        let hash = sha2::Sha256::digest(key.as_bytes());
        format!("idem/{:x}.json", hash)
    }
}

#[async_trait]
impl IdempotencyService for S3IdempotencyService {
    async fn check_idempotency(&self, _key: &str, _body_hash: &str) -> Result<Option<String>> {
        // For MVP, we'll skip idempotency checking
        Ok(None)
    }

    async fn store_idempotency(&self, key: &str, body_hash: &str, response: &str) -> Result<()> {
        let s3_key = self.idempotency_key(key);
        
        let record = serde_json::json!({
            "body_hash": body_hash,
            "response": response,
            "stored_at": Utc::now().to_rfc3339()
        });
        
        let body = serde_json::to_string(&record)
            .map_err(|e| DomainError::Internal(format!("Failed to serialize idempotency record: {}", e)))?;

        let _result = self.client
            .put_object()
            .bucket(&self.config.private_bucket)
            .key(&s3_key)
            .body(body.into_bytes().into())
            .content_type("application/json")
            .send()
            .await
            .map_err(|e| DomainError::Internal(format!("Failed to store idempotency record: {}", e)))?;

        Ok(())
    }
}