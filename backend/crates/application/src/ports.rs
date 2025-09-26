use async_trait::async_trait;
use domain::entities::*;
use domain::events::*;
use domain::value_objects::*;
use domain::error::Result;

/// Repository for managing RFQ data
#[async_trait]
pub trait RfqRepository {
    async fn save_rfq_meta(&self, rfq: &RfqMeta) -> Result<()>;
    async fn get_rfq_meta(&self, id: &RfqId) -> Result<Option<RfqMeta>>;
    async fn save_rfq_index(&self, rfq_id: &RfqId, index: &RfqIndex) -> Result<()>;
    async fn get_rfq_index(&self, id: &RfqId) -> Result<Option<RfqIndex>>;
    async fn save_rfq_event(&self, event: &RfqEvent) -> Result<()>;
    async fn list_rfq_events(&self, rfq_id: &RfqId, since: Option<chrono::DateTime<chrono::Utc>>, limit: Option<u32>) -> Result<Vec<RfqEvent>>;
}

/// Repository for managing manufacturer data
#[async_trait]
pub trait ManufacturerRepository {
    async fn save_manufacturer(&self, manufacturer: &ManufacturerProfile) -> Result<()>;
    async fn get_manufacturer(&self, id: &ManufacturerId) -> Result<Option<ManufacturerProfile>>;
    async fn delete_manufacturer(&self, id: &ManufacturerId) -> Result<()>;
}

/// Repository for managing catalog data
#[async_trait]
pub trait CatalogRepository {
    async fn save_category_slice(&self, slice: &CategorySlice) -> Result<()>;
    async fn get_category_slice(&self, category: &str) -> Result<Option<CategorySlice>>;
    async fn save_category_state_slice(&self, category: &str, state: &str, slice: &CategorySlice) -> Result<()>;
    async fn get_category_state_slice(&self, category: &str, state: &str) -> Result<Option<CategorySlice>>;
}

/// Service for managing image uploads and processing
#[async_trait]
pub trait ImageService {
    async fn generate_presigned_upload_url(&self, tenant_id: &TenantId, content_type: &ContentType, size: &FileSize) -> Result<crate::dto::PresignUploadResponse>;
    async fn save_image_manifest(&self, manifest: &ImageManifest) -> Result<()>;
    async fn get_image_manifest(&self, id: &str) -> Result<Option<ImageManifest>>;
}

/// Email notification service
#[async_trait]
pub trait EmailService {
    async fn send_rfq_created_notification(&self, rfq: &RfqMeta) -> Result<()>;
    async fn send_rfq_message_notification(&self, rfq: &RfqMeta, event: &RfqEvent) -> Result<()>;
}

/// Idempotency service
#[async_trait]
pub trait IdempotencyService {
    async fn check_idempotency(&self, key: &str, body_hash: &str) -> Result<Option<String>>;
    async fn store_idempotency(&self, key: &str, body_hash: &str, response: &str) -> Result<()>;
}

