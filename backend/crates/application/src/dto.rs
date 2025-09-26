use domain::entities::*;
use serde::{Deserialize, Serialize};

/// DTO for creating a new RFQ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRfqRequest {
    pub tenant_id: String,
    pub manufacturer_id: String,
    pub buyer: ContactDto,
    pub subject: String,
    pub body: String,
    pub attachments: Option<Vec<AttachmentDto>>,
}

/// DTO for RFQ response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRfqResponse {
    pub id: String,
    pub last_event_ts: String,
}

/// DTO for listing events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListEventsResponse {
    pub items: Vec<RfqEventDto>,
    pub next_since: Option<String>,
}

/// DTO for posting a message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostMessageRequest {
    pub by: String, // "buyer" | "manufacturer"
    pub body: String,
    pub attachments: Option<Vec<AttachmentDto>>,
}

/// DTO for message response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostMessageResponse {
    pub ts: String,
}

/// DTO for presigned upload request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresignUploadRequest {
    pub tenant_id: String,
    #[serde(rename = "pathType")]
    pub path_type: String, // "imageRaw"
    pub content_type: String,
    pub size_bytes: u64,
}

/// DTO for presigned upload response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresignUploadResponse {
    pub url: String,
    pub key: String,
    pub expires_in: u32,
}

/// DTO for manufacturer creation/update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateManufacturerRequest {
    pub id: Option<String>,
    pub tenant_id: String,
    pub name: String,
    pub description: Option<String>,
    pub website: Option<String>,
    pub location: Option<LocationDto>,
    pub capabilities: Vec<String>,
    pub contact_email: String,
    pub contact_phone: Option<String>,
    pub offerings: Vec<OfferingDto>,
}

/// DTO for manufacturer creation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateManufacturerResponse {
    pub id: String,
    pub tenant_id: String,
}

// Supporting DTOs matching the entities

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactDto {
    pub email: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentDto {
    pub upload_key: String,
    pub file_name: String,
    pub content_type: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationDto {
    pub city: Option<String>,
    pub state: Option<String>,
    pub country: Option<String>,
    pub lat: Option<f64>,
    pub lng: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaRefDto {
    pub image_manifest_id: String,
    pub alt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfferingDto {
    pub id: Option<String>,
    pub title: String,
    pub materials: Option<Vec<String>>,
    pub lead_time_days: Option<LeadTimeDto>,
    pub media: Option<Vec<MediaRefDto>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeadTimeDto {
    pub min: Option<u32>,
    pub max: Option<u32>,
}

/// DTO for RFQ events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RfqEventDto {
    #[serde(rename = "message")]
    Message {
        id: String,
        rfq_id: String,
        ts: String,
        by: String,
        body: String,
    },
    #[serde(rename = "status")]
    Status {
        id: String,
        rfq_id: String,
        ts: String,
        by: String,
        status: String,
        note: Option<String>,
    },
    #[serde(rename = "attachment")]
    Attachment {
        id: String,
        rfq_id: String,
        ts: String,
        by: String,
        attachments: Vec<AttachmentRefDto>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentRefDto {
    pub id: String,
    pub file_name: String,
    pub content_type: String,
    pub size_bytes: u64,
    pub key: String,
}

/// Convert domain entities to DTOs
impl From<Contact> for ContactDto {
    fn from(contact: Contact) -> Self {
        ContactDto {
            email: contact.email,
            name: contact.name,
        }
    }
}

impl From<Location> for LocationDto {
    fn from(location: Location) -> Self {
        LocationDto {
            city: location.city,
            state: location.state,
            country: location.country,
            lat: location.lat,
            lng: location.lng,
        }
    }
}

impl From<AttachmentRef> for AttachmentRefDto {
    fn from(attachment: AttachmentRef) -> Self {
        AttachmentRefDto {
            id: attachment.id,
            file_name: attachment.file_name,
            content_type: attachment.content_type,
            size_bytes: attachment.size_bytes,
            key: attachment.key,
        }
    }
}
