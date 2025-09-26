use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Location information for manufacturers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Location {
    pub city: Option<String>,
    pub state: Option<String>,
    pub country: Option<String>,
    pub lat: Option<f64>,
    pub lng: Option<f64>,
}

/// Reference to an attachment file
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AttachmentRef {
    pub id: String,
    pub file_name: String,
    pub content_type: String,
    pub size_bytes: u64,
    pub key: String, // S3 private key
}

/// Contact information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Contact {
    pub email: String,
    pub name: Option<String>,
}

/// Image manifest reference
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MediaRef {
    pub image_manifest_id: String,
    pub alt: Option<String>,
}

/// Lead time range in days
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LeadTime {
    pub min: Option<u32>,
    pub max: Option<u32>,
}

/// Tenant entity
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Tenant {
    pub id: String,
    pub name: String,
    pub plan: String,
    pub created_at: DateTime<Utc>,
}

/// Manufacturer entity
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Manufacturer {
    pub id: String,
    pub tenant_id: String,
    pub name: String,
    pub description: Option<String>,
    pub location: Option<Location>,
    pub categories: Vec<String>,
    pub capabilities: Option<Vec<String>>,
    pub contact_email: Option<String>,
    pub media: Option<Vec<MediaRef>>,
    pub updated_at: DateTime<Utc>,
}

/// Manufacturing offering
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Offering {
    pub id: String,
    pub title: String,
    pub materials: Option<Vec<String>>,
    pub lead_time_days: Option<LeadTime>,
    pub media: Option<Vec<MediaRef>>,
}

/// Complete manufacturer profile (with offerings)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ManufacturerProfile {
    pub id: String,
    pub tenant_id: String,
    pub name: String,
    pub description: Option<String>,
    pub location: Option<Location>,
    pub categories: Vec<String>,
    pub capabilities: Option<Vec<String>>,
    pub contact_email: Option<String>,
    pub media: Option<Vec<MediaRef>>,
    pub offerings: Option<Vec<Offering>>,
    pub updated_at: DateTime<Utc>,
}

/// RFQ status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RfqStatus {
    Open,
    Archived,
    Closed,
}

/// RFQ participant role
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ParticipantRole {
    Buyer,
    Manufacturer,
}

/// RFQ participant
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Participant {
    pub role: ParticipantRole,
    pub email: String,
    pub name: Option<String>,
}

/// RFQ metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RfqMeta {
    pub id: String,
    pub tenant_id: String,
    pub manufacturer_id: String,
    pub buyer: Contact,
    pub subject: String,
    pub status: RfqStatus,
    pub created_at: DateTime<Utc>,
    pub last_event_ts: DateTime<Utc>,
    pub participants: Vec<Participant>,
    pub attachments: Option<Vec<AttachmentRef>>,
}

/// RFQ index for quick lookups
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RfqIndex {
    pub last_event_ts: DateTime<Utc>,
    pub count: u32,
}

/// Catalog manufacturer summary
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CatalogManufacturerSummary {
    pub id: String,
    pub name: String,
    pub city: Option<String>,
    pub state: Option<String>,
    pub categories: Vec<String>,
    pub capabilities: Option<Vec<String>>,
    pub logo: Option<String>,
}

/// Catalog category slice
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CategorySlice {
    pub category: String,
    pub generated_at: DateTime<Utc>,
    pub items: Vec<CatalogManufacturerSummary>,
}

/// Image variant
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImageVariant {
    pub w: u32,
    #[serde(rename = "t")]
    pub content_type: String, // "image/avif" | "image/webp" | "image/jpeg"
    #[serde(rename = "k")]
    pub key: String, // S3 key in public bucket
}

/// Image manifest
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImageManifest {
    pub id: String,
    pub w: u32, // original width
    pub h: u32, // original height
    pub variants: Vec<ImageVariant>,
    pub lqip: Option<String>, // data URI for low quality placeholder
    pub created_at: DateTime<Utc>,
}