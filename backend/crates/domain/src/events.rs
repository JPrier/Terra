use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::AttachmentRef;

/// Who created the event
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum EventAuthor {
    Buyer,
    Manufacturer,
    System,
}

/// Base properties for all RFQ events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RfqEventBase {
    pub id: String,
    pub rfq_id: String,
    pub ts: DateTime<Utc>,
    pub by: EventAuthor,
    #[serde(rename = "type")]
    pub event_type: String,
}

/// Message event - buyer or manufacturer sends a text message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MessageEvent {
    #[serde(flatten)]
    pub base: RfqEventBase,
    pub body: String, // markdown-safe plain text, max 8000 chars
}

/// Status types for status events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum StatusType {
    RfqCreated,
    VendorViewed,
    VendorReplied,
    BuyerViewed,
    Closed,
    Archived,
}

/// Status event - system or participant changes RFQ status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StatusEvent {
    #[serde(flatten)]
    pub base: RfqEventBase,
    pub status: StatusType,
    pub note: Option<String>,
}

/// Attachment event - files are uploaded
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AttachmentEvent {
    #[serde(flatten)]
    pub base: RfqEventBase,
    pub attachments: Vec<AttachmentRef>,
}

/// Union type for all RFQ events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum RfqEvent {
    #[serde(rename = "message")]
    Message(MessageEvent),
    #[serde(rename = "status")]
    Status(StatusEvent),
    #[serde(rename = "attachment")]
    Attachment(AttachmentEvent),
}

impl RfqEvent {
    pub fn id(&self) -> &str {
        match self {
            RfqEvent::Message(e) => &e.base.id,
            RfqEvent::Status(e) => &e.base.id,
            RfqEvent::Attachment(e) => &e.base.id,
        }
    }

    pub fn rfq_id(&self) -> &str {
        match self {
            RfqEvent::Message(e) => &e.base.rfq_id,
            RfqEvent::Status(e) => &e.base.rfq_id,
            RfqEvent::Attachment(e) => &e.base.rfq_id,
        }
    }

    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            RfqEvent::Message(e) => e.base.ts,
            RfqEvent::Status(e) => e.base.ts,
            RfqEvent::Attachment(e) => e.base.ts,
        }
    }

    pub fn author(&self) -> &EventAuthor {
        match self {
            RfqEvent::Message(e) => &e.base.by,
            RfqEvent::Status(e) => &e.base.by,
            RfqEvent::Attachment(e) => &e.base.by,
        }
    }

    /// Create a new message event
    pub fn new_message(
        rfq_id: String,
        author: EventAuthor,
        body: String,
    ) -> Self {
        let id = Uuid::new_v4().to_string();
        let ts = Utc::now();
        
        RfqEvent::Message(MessageEvent {
            base: RfqEventBase {
                id,
                rfq_id,
                ts,
                by: author,
                event_type: "message".to_string(),
            },
            body,
        })
    }

    /// Create a new status event
    pub fn new_status(
        rfq_id: String,
        author: EventAuthor,
        status: StatusType,
        note: Option<String>,
    ) -> Self {
        let id = Uuid::new_v4().to_string();
        let ts = Utc::now();
        
        RfqEvent::Status(StatusEvent {
            base: RfqEventBase {
                id,
                rfq_id,
                ts,
                by: author,
                event_type: "status".to_string(),
            },
            status,
            note,
        })
    }

    /// Create a new attachment event
    pub fn new_attachment(
        rfq_id: String,
        author: EventAuthor,
        attachments: Vec<AttachmentRef>,
    ) -> Self {
        let id = Uuid::new_v4().to_string();
        let ts = Utc::now();
        
        RfqEvent::Attachment(AttachmentEvent {
            base: RfqEventBase {
                id,
                rfq_id,
                ts,
                by: author,
                event_type: "attachment".to_string(),
            },
            attachments,
        })
    }
}