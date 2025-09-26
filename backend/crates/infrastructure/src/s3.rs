use async_trait::async_trait;
use aws_sdk_s3::{Client as S3Client, Error as S3Error};
use aws_sdk_s3::types::{BucketCannedAcl, ObjectCannedAcl};
use aws_sdk_s3::presigning::PresigningConfig;
use chrono::{DateTime, Utc};
use domain::entities::*;
use domain::events::*;
use domain::value_objects::*;
use domain::error::{DomainError, Result};
use application::ports::*;
use std::time::Duration;
use std::sync::Arc;

use crate::config::Config;

/// S3-based repository implementations
pub struct S3RfqRepository {
    client: S3Client,
    config: Arc<Config>,
}

impl S3RfqRepository {
    pub fn new(client: S3Client, config: Arc<Config>) -> Self {
        Self { client, config }
    }

    fn rfq_meta_key(&self, rfq_id: &RfqId) -> String {
        format!("rfq/{}/meta.json", rfq_id.as_str())
    }

    fn rfq_index_key(&self, rfq_id: &RfqId) -> String {
        format!("rfq/{}/index.json", rfq_id.as_str())
    }

    fn rfq_event_key(&self, rfq_id: &str, event: &RfqEvent) -> String {
        // Format timestamp for S3 key (replace : with -)
        let ts_str = event.timestamp().format("%Y-%m-%dT%H-%M-%SZ").to_string();
        format!("rfq/{}/events/{}-{}.json", rfq_id, ts_str, event.id())
    }
}

#[async_trait]
impl RfqRepository for S3RfqRepository {
    async fn save_rfq_meta(&self, rfq: &RfqMeta) -> Result<()> {
        let key = self.rfq_meta_key(&RfqId::new(rfq.id.clone())?);
        let body = serde_json::to_string(rfq)
            .map_err(|e| DomainError::Internal(format!("Failed to serialize RFQ meta: {}", e)))?;

        self.client
            .put_object()
            .bucket(&self.config.private_bucket)
            .key(&key)
            .body(body.into_bytes().into())
            .content_type("application/json")
            .cache_control("private, max-age=0, no-store")
            .send()
            .await
            .map_err(|e| DomainError::Internal(format!("Failed to save RFQ meta: {}", e)))?;

        Ok(())
    }

    async fn get_rfq_meta(&self, id: &RfqId) -> Result<Option<RfqMeta>> {
        let key = self.rfq_meta_key(id);
        
        match self.client
            .get_object()
            .bucket(&self.config.private_bucket)
            .key(&key)
            .send()
            .await
        {
            Ok(output) => {
                let bytes = output.body.collect().await
                    .map_err(|e| DomainError::Internal(format!("Failed to read RFQ meta body: {}", e)))?
                    .into_bytes();
                
                let rfq_meta: RfqMeta = serde_json::from_slice(&bytes)
                    .map_err(|e| DomainError::Internal(format!("Failed to deserialize RFQ meta: {}", e)))?;
                
                Ok(Some(rfq_meta))
            }
            Err(S3Error::NoSuchKey(_)) => Ok(None),
            Err(e) => Err(DomainError::Internal(format!("Failed to get RFQ meta: {}", e))),
        }
    }

    async fn save_rfq_index(&self, rfq_id: &RfqId, index: &RfqIndex) -> Result<()> {
        let key = self.rfq_index_key(rfq_id);
        let body = serde_json::to_string(index)
            .map_err(|e| DomainError::Internal(format!("Failed to serialize RFQ index: {}", e)))?;

        self.client
            .put_object()
            .bucket(&self.config.private_bucket)
            .key(&key)
            .body(body.into_bytes().into())
            .content_type("application/json")
            .cache_control("private, max-age=0, no-store")
            .send()
            .await
            .map_err(|e| DomainError::Internal(format!("Failed to save RFQ index: {}", e)))?;

        Ok(())
    }

    async fn get_rfq_index(&self, id: &RfqId) -> Result<Option<RfqIndex>> {
        let key = self.rfq_index_key(id);
        
        match self.client
            .get_object()
            .bucket(&self.config.private_bucket)
            .key(&key)
            .send()
            .await
        {
            Ok(output) => {
                let bytes = output.body.collect().await
                    .map_err(|e| DomainError::Internal(format!("Failed to read RFQ index body: {}", e)))?
                    .into_bytes();
                
                let rfq_index: RfqIndex = serde_json::from_slice(&bytes)
                    .map_err(|e| DomainError::Internal(format!("Failed to deserialize RFQ index: {}", e)))?;
                
                Ok(Some(rfq_index))
            }
            Err(S3Error::NoSuchKey(_)) => Ok(None),
            Err(e) => Err(DomainError::Internal(format!("Failed to get RFQ index: {}", e))),
        }
    }

    async fn save_rfq_event(&self, event: &RfqEvent) -> Result<()> {
        let key = self.rfq_event_key(event.rfq_id(), event);
        let body = serde_json::to_string(event)
            .map_err(|e| DomainError::Internal(format!("Failed to serialize RFQ event: {}", e)))?;

        self.client
            .put_object()
            .bucket(&self.config.private_bucket)
            .key(&key)
            .body(body.into_bytes().into())
            .content_type("application/json")
            .cache_control("private, max-age=0, no-store")
            .send()
            .await
            .map_err(|e| DomainError::Internal(format!("Failed to save RFQ event: {}", e)))?;

        Ok(())
    }

    async fn list_rfq_events(&self, rfq_id: &RfqId, since: Option<DateTime<Utc>>, limit: Option<u32>) -> Result<Vec<RfqEvent>> {
        let prefix = format!("rfq/{}/events/", rfq_id.as_str());
        let limit = limit.unwrap_or(50).min(200); // Cap at 200 as per design
        
        let mut request = self.client
            .list_objects_v2()
            .bucket(&self.config.private_bucket)
            .prefix(&prefix)
            .max_keys(limit as i32);

        // If since is provided, use it as start_after (approximate)
        if let Some(since_ts) = since {
            let since_key = format!("{}{}-", prefix, since_ts.format("%Y-%m-%dT%H-%M-%SZ"));
            request = request.start_after(since_key);
        }

        let list_output = request
            .send()
            .await
            .map_err(|e| DomainError::Internal(format!("Failed to list RFQ events: {}", e)))?;

        let mut events = Vec::new();
        
        if let Some(objects) = list_output.contents {
            for obj in objects {
                if let Some(key) = obj.key {
                    let get_output = self.client
                        .get_object()
                        .bucket(&self.config.private_bucket)
                        .key(&key)
                        .send()
                        .await
                        .map_err(|e| DomainError::Internal(format!("Failed to get event {}: {}", key, e)))?;

                    let bytes = get_output.body.collect().await
                        .map_err(|e| DomainError::Internal(format!("Failed to read event body: {}", e)))?
                        .into_bytes();
                    
                    let event: RfqEvent = serde_json::from_slice(&bytes)
                        .map_err(|e| DomainError::Internal(format!("Failed to deserialize event: {}", e)))?;
                    
                    events.push(event);
                }
            }
        }

        // Sort events by timestamp then by ID for stability
        events.sort_by(|a, b| {
            a.timestamp().cmp(&b.timestamp())
                .then_with(|| a.id().cmp(b.id()))
        });

        Ok(events)
    }
}

/// S3-based manufacturer repository
pub struct S3ManufacturerRepository {
    client: S3Client,
    config: Arc<Config>,
}

impl S3ManufacturerRepository {
    pub fn new(client: S3Client, config: Arc<Config>) -> Self {
        Self { client, config }
    }

    fn manufacturer_key(&self, id: &ManufacturerId) -> String {
        format!("manufacturer/{}.json", id.as_str())
    }
}

#[async_trait]
impl ManufacturerRepository for S3ManufacturerRepository {
    async fn save_manufacturer(&self, manufacturer: &ManufacturerProfile) -> Result<()> {
        let key = self.manufacturer_key(&ManufacturerId::new(manufacturer.id.clone())?);
        let body = serde_json::to_string(manufacturer)
            .map_err(|e| DomainError::Internal(format!("Failed to serialize manufacturer: {}", e)))?;

        self.client
            .put_object()
            .bucket(&self.config.public_bucket)
            .key(&key)
            .body(body.into_bytes().into())
            .content_type("application/json")
            .cache_control("public, max-age=31536000, immutable")
            .send()
            .await
            .map_err(|e| DomainError::Internal(format!("Failed to save manufacturer: {}", e)))?;

        Ok(())
    }

    async fn get_manufacturer(&self, id: &ManufacturerId) -> Result<Option<ManufacturerProfile>> {
        let key = self.manufacturer_key(id);
        
        match self.client
            .get_object()
            .bucket(&self.config.public_bucket)
            .key(&key)
            .send()
            .await
        {
            Ok(output) => {
                let bytes = output.body.collect().await
                    .map_err(|e| DomainError::Internal(format!("Failed to read manufacturer body: {}", e)))?
                    .into_bytes();
                
                let manufacturer: ManufacturerProfile = serde_json::from_slice(&bytes)
                    .map_err(|e| DomainError::Internal(format!("Failed to deserialize manufacturer: {}", e)))?;
                
                Ok(Some(manufacturer))
            }
            Err(S3Error::NoSuchKey(_)) => Ok(None),
            Err(e) => Err(DomainError::Internal(format!("Failed to get manufacturer: {}", e))),
        }
    }

    async fn delete_manufacturer(&self, id: &ManufacturerId) -> Result<()> {
        let key = self.manufacturer_key(id);

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

/// S3-based catalog repository
pub struct S3CatalogRepository {
    client: S3Client,
    config: Arc<Config>,
}

impl S3CatalogRepository {
    pub fn new(client: S3Client, config: Arc<Config>) -> Self {
        Self { client, config }
    }

    fn category_key(&self, category: &str) -> String {
        format!("catalog/category/{}.json", category)
    }

    fn category_state_key(&self, category: &str, state: &str) -> String {
        format!("catalog/category_state/{}/{}.json", category, state)
    }
}

#[async_trait]
impl CatalogRepository for S3CatalogRepository {
    async fn save_category_slice(&self, slice: &CategorySlice) -> Result<()> {
        let key = self.category_key(&slice.category);
        let body = serde_json::to_string(slice)
            .map_err(|e| DomainError::Internal(format!("Failed to serialize category slice: {}", e)))?;

        self.client
            .put_object()
            .bucket(&self.config.public_bucket)
            .key(&key)
            .body(body.into_bytes().into())
            .content_type("application/json")
            .cache_control("public, max-age=31536000, immutable")
            .send()
            .await
            .map_err(|e| DomainError::Internal(format!("Failed to save category slice: {}", e)))?;

        Ok(())
    }

    async fn get_category_slice(&self, category: &str) -> Result<Option<CategorySlice>> {
        let key = self.category_key(category);
        
        match self.client
            .get_object()
            .bucket(&self.config.public_bucket)
            .key(&key)
            .send()
            .await
        {
            Ok(output) => {
                let bytes = output.body.collect().await
                    .map_err(|e| DomainError::Internal(format!("Failed to read category slice body: {}", e)))?
                    .into_bytes();
                
                let slice: CategorySlice = serde_json::from_slice(&bytes)
                    .map_err(|e| DomainError::Internal(format!("Failed to deserialize category slice: {}", e)))?;
                
                Ok(Some(slice))
            }
            Err(S3Error::NoSuchKey(_)) => Ok(None),
            Err(e) => Err(DomainError::Internal(format!("Failed to get category slice: {}", e))),
        }
    }

    async fn save_category_state_slice(&self, category: &str, state: &str, slice: &CategorySlice) -> Result<()> {
        let key = self.category_state_key(category, state);
        let body = serde_json::to_string(slice)
            .map_err(|e| DomainError::Internal(format!("Failed to serialize category state slice: {}", e)))?;

        self.client
            .put_object()
            .bucket(&self.config.public_bucket)
            .key(&key)
            .body(body.into_bytes().into())
            .content_type("application/json")
            .cache_control("public, max-age=31536000, immutable")
            .send()
            .await
            .map_err(|e| DomainError::Internal(format!("Failed to save category state slice: {}", e)))?;

        Ok(())
    }

    async fn get_category_state_slice(&self, category: &str, state: &str) -> Result<Option<CategorySlice>> {
        let key = self.category_state_key(category, state);
        
        match self.client
            .get_object()
            .bucket(&self.config.public_bucket)
            .key(&key)
            .send()
            .await
        {
            Ok(output) => {
                let bytes = output.body.collect().await
                    .map_err(|e| DomainError::Internal(format!("Failed to read category state slice body: {}", e)))?
                    .into_bytes();
                
                let slice: CategorySlice = serde_json::from_slice(&bytes)
                    .map_err(|e| DomainError::Internal(format!("Failed to deserialize category state slice: {}", e)))?;
                
                Ok(Some(slice))
            }
            Err(S3Error::NoSuchKey(_)) => Ok(None),
            Err(e) => Err(DomainError::Internal(format!("Failed to get category state slice: {}", e))),
        }
    }
}

/// S3-based image service
pub struct S3ImageService {
    client: S3Client,
    config: Arc<Config>,
}

impl S3ImageService {
    pub fn new(client: S3Client, config: Arc<Config>) -> Self {
        Self { client, config }
    }

    fn generate_raw_image_key(&self, tenant_id: &TenantId) -> String {
        let uuid = uuid::Uuid::new_v4();
        format!("tenants/{}/images/raw/{}", tenant_id.as_str(), uuid)
    }

    fn manifest_key(&self, tenant_id: &TenantId, image_id: &str) -> String {
        format!("tenants/{}/manifests/{}.json", tenant_id.as_str(), image_id)
    }
}

#[async_trait]
impl ImageService for S3ImageService {
    async fn generate_presigned_upload_url(&self, tenant_id: &TenantId, content_type: &ContentType, size: &FileSize) -> Result<PresignedUploadResponse> {
        let key = self.generate_raw_image_key(tenant_id);
        
        let presigning_config = PresigningConfig::expires_in(Duration::from_secs(600)) // 10 minutes
            .map_err(|e| DomainError::Internal(format!("Failed to create presigning config: {}", e)))?;

        let presigned_request = self.client
            .put_object()
            .bucket(&self.config.private_bucket)
            .key(&key)
            .content_type(content_type.as_str())
            .content_length(size.as_u64() as i64)
            .metadata("tenant", tenant_id.as_str())
            .presigned(presigning_config)
            .await
            .map_err(|e| DomainError::Internal(format!("Failed to generate presigned URL: {}", e)))?;

        Ok(PresignedUploadResponse {
            url: presigned_request.uri().to_string(),
            key: key,
            expires_in: 600,
        })
    }

    async fn save_image_manifest(&self, manifest: &ImageManifest) -> Result<()> {
        // Extract tenant_id from the first variant key
        let tenant_id = if let Some(variant) = manifest.variants.first() {
            let parts: Vec<&str> = variant.key.split('/').collect();
            if parts.len() >= 2 && parts[0] == "tenants" {
                TenantId::new(parts[1].to_string())?
            } else {
                return Err(DomainError::ValidationFailed("Invalid variant key format".to_string()));
            }
        } else {
            return Err(DomainError::ValidationFailed("No variants in manifest".to_string()));
        };

        let key = self.manifest_key(&tenant_id, &manifest.id);
        let body = serde_json::to_string(manifest)
            .map_err(|e| DomainError::Internal(format!("Failed to serialize image manifest: {}", e)))?;

        self.client
            .put_object()
            .bucket(&self.config.public_bucket)
            .key(&key)
            .body(body.into_bytes().into())
            .content_type("application/json")
            .cache_control("public, max-age=31536000, immutable")
            .send()
            .await
            .map_err(|e| DomainError::Internal(format!("Failed to save image manifest: {}", e)))?;

        Ok(())
    }

    async fn get_image_manifest(&self, id: &str) -> Result<Option<ImageManifest>> {
        // This is a simplified implementation - in practice, we'd need the tenant_id
        // For now, we'll search across all tenants (not efficient, but works for MVP)
        return Err(DomainError::Internal("get_image_manifest not fully implemented".to_string()));
    }
}

/// S3-based idempotency service
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
    async fn check_idempotency(&self, key: &str, body_hash: &str) -> Result<Option<String>> {
        let s3_key = self.idempotency_key(key);
        
        match self.client
            .get_object()
            .bucket(&self.config.private_bucket)
            .key(&s3_key)
            .send()
            .await
        {
            Ok(output) => {
                let bytes = output.body.collect().await
                    .map_err(|e| DomainError::Internal(format!("Failed to read idempotency record: {}", e)))?
                    .into_bytes();
                
                let record: serde_json::Value = serde_json::from_slice(&bytes)
                    .map_err(|e| DomainError::Internal(format!("Failed to deserialize idempotency record: {}", e)))?;
                
                if let (Some(stored_hash), Some(response)) = (record.get("body_hash"), record.get("response")) {
                    let stored_hash_str = stored_hash.as_str().ok_or_else(|| 
                        DomainError::Internal("Invalid body_hash in idempotency record".to_string()))?;
                    
                    if stored_hash_str == body_hash {
                        let response_str = response.as_str().ok_or_else(|| 
                            DomainError::Internal("Invalid response in idempotency record".to_string()))?;
                        return Ok(Some(response_str.to_string()));
                    } else {
                        return Err(DomainError::Conflict("Idempotency key reused with different body".to_string()));
                    }
                }
                
                Ok(None)
            }
            Err(S3Error::NoSuchKey(_)) => Ok(None),
            Err(e) => Err(DomainError::Internal(format!("Failed to check idempotency: {}", e))),
        }
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

        self.client
            .put_object()
            .bucket(&self.config.private_bucket)
            .key(&s3_key)
            .body(body.into_bytes().into())
            .content_type("application/json")
            .cache_control("private, max-age=86400") // 24 hours as per design
            .send()
            .await
            .map_err(|e| DomainError::Internal(format!("Failed to store idempotency record: {}", e)))?;

        Ok(())
    }
}