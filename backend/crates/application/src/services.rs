use chrono::Utc;
use domain::entities::*;
use domain::events::*;
use domain::value_objects::*;
use domain::error::{DomainError, Result};
use std::sync::Arc;
use uuid::Uuid;
use sha2::Digest;

use crate::dto::*;
use crate::ports::*;

/// Main RFQ service for handling RFQ operations
pub struct RfqService {
    rfq_repository: Arc<dyn RfqRepository + Send + Sync>,
    manufacturer_repository: Arc<dyn ManufacturerRepository + Send + Sync>,
    email_service: Arc<dyn EmailService + Send + Sync>,
    idempotency_service: Arc<dyn IdempotencyService + Send + Sync>,
}

impl RfqService {
    pub fn new(
        rfq_repository: Arc<dyn RfqRepository + Send + Sync>,
        manufacturer_repository: Arc<dyn ManufacturerRepository + Send + Sync>,
        email_service: Arc<dyn EmailService + Send + Sync>,
        idempotency_service: Arc<dyn IdempotencyService + Send + Sync>,
    ) -> Self {
        Self {
            rfq_repository,
            manufacturer_repository,
            email_service,
            idempotency_service,
        }
    }

    pub async fn create_rfq(&self, request: CreateRfqRequest, idempotency_key: Option<&str>) -> Result<CreateRfqResponse> {
        // Check idempotency if key provided
        if let Some(key) = idempotency_key {
            let body_hash = self.compute_request_hash(&request)?;
            if let Some(cached_response) = self.idempotency_service.check_idempotency(key, &body_hash).await? {
                return Ok(serde_json::from_str(&cached_response)
                    .map_err(|_| DomainError::Internal("Failed to deserialize cached response".to_string()))?);
            }
        }

        // Validate input
        let tenant_id = TenantId::new(request.tenant_id.clone())?;
        let manufacturer_id = ManufacturerId::new(request.manufacturer_id.clone())?;
        let buyer_email = Email::new(request.buyer.email.clone())?;
        let message_body = MessageBody::new(request.body.clone())?;

        // Verify manufacturer exists
        let manufacturer = self.manufacturer_repository.get_manufacturer(&manufacturer_id).await?
            .ok_or_else(|| DomainError::NotFound("Manufacturer not found".to_string()))?;

        // Generate RFQ ID
        let rfq_id = RfqId::generate();
        let now = Utc::now();

        // Create buyer contact
        let buyer = Contact {
            email: buyer_email.as_str().to_string(),
            name: request.buyer.name.clone(),
        };

        // Process attachments if any
        let attachments = if let Some(attachment_dtos) = request.attachments.clone() {
            let mut processed_attachments = Vec::new();
            for attachment_dto in attachment_dtos {
                let content_type = ContentType::new(attachment_dto.content_type)?;
                let file_size = FileSize::new(attachment_dto.size_bytes)?;
                
                processed_attachments.push(AttachmentRef {
                    id: Uuid::new_v4().to_string(),
                    file_name: attachment_dto.file_name,
                    content_type: content_type.as_str().to_string(),
                    size_bytes: file_size.as_u64(),
                    key: attachment_dto.upload_key,
                });
            }
            Some(processed_attachments)
        } else {
            None
        };

        // Create participants
        let participants = vec![
            Participant {
                role: ParticipantRole::Buyer,
                email: buyer.email.clone(),
                name: buyer.name.clone(),
            },
            Participant {
                role: ParticipantRole::Manufacturer,
                email: manufacturer.contact_email.unwrap_or_default(),
                name: Some(manufacturer.name.clone()),
            },
        ];

        // Create RFQ meta
        let rfq_meta = RfqMeta {
            id: rfq_id.as_str().to_string(),
            tenant_id: tenant_id.as_str().to_string(),
            manufacturer_id: manufacturer_id.as_str().to_string(),
            buyer: buyer.clone(),
            subject: request.subject.clone(),
            status: RfqStatus::Open,
            created_at: now,
            last_event_ts: now,
            participants,
            attachments: attachments.clone(),
        };

        // Save RFQ meta
        self.rfq_repository.save_rfq_meta(&rfq_meta).await?;

        // Create initial events
        let status_event = RfqEvent::new_status(
            rfq_id.as_str().to_string(),
            EventAuthor::System,
            StatusType::RfqCreated,
            None,
        );

        let message_event = RfqEvent::new_message(
            rfq_id.as_str().to_string(),
            EventAuthor::Buyer,
            message_body.as_str().to_string(),
        );

        // Save events
        self.rfq_repository.save_rfq_event(&status_event).await?;
        self.rfq_repository.save_rfq_event(&message_event).await?;

        // If there are attachments, create attachment event
        if let Some(attachments) = attachments {
            let attachment_event = RfqEvent::new_attachment(
                rfq_id.as_str().to_string(),
                EventAuthor::Buyer,
                attachments,
            );
            self.rfq_repository.save_rfq_event(&attachment_event).await?;
        }

        // Update RFQ index
        let index = RfqIndex {
            last_event_ts: now,
            count: if rfq_meta.attachments.is_some() { 3 } else { 2 },
        };
        self.rfq_repository.save_rfq_index(&rfq_id, &index).await?;

        // Send notifications
        self.email_service.send_rfq_created_notification(&rfq_meta).await?;

        let response = CreateRfqResponse {
            id: rfq_id.as_str().to_string(),
            last_event_ts: now.to_rfc3339(),
        };

        // Store idempotency result if key provided
        if let Some(key) = idempotency_key {
            let body_hash = self.compute_request_hash(&request)?;
            let response_json = serde_json::to_string(&response)
                .map_err(|_| DomainError::Internal("Failed to serialize response".to_string()))?;
            self.idempotency_service.store_idempotency(key, &body_hash, &response_json).await?;
        }

        Ok(response)
    }

    pub async fn get_rfq(&self, rfq_id: &str) -> Result<Option<RfqMeta>> {
        let rfq_id = RfqId::new(rfq_id.to_string())?;
        self.rfq_repository.get_rfq_meta(&rfq_id).await
    }

    pub async fn list_events(&self, rfq_id: &str, since: Option<String>, limit: Option<u32>) -> Result<ListEventsResponse> {
        let rfq_id = RfqId::new(rfq_id.to_string())?;
        
        let since_dt = if let Some(since_str) = since {
            Some(chrono::DateTime::parse_from_rfc3339(&since_str)
                .map_err(|_| DomainError::ValidationFailed("Invalid since timestamp format".to_string()))?
                .with_timezone(&Utc))
        } else {
            None
        };

        let events = self.rfq_repository.list_rfq_events(&rfq_id, since_dt, limit).await?;
        
        let event_dtos: Vec<RfqEventDto> = events.iter().map(|e| self.event_to_dto(e)).collect();
        
        let next_since = events.last().map(|e| e.timestamp().to_rfc3339());

        Ok(ListEventsResponse {
            items: event_dtos,
            next_since,
        })
    }

    pub async fn post_message(&self, rfq_id: &str, request: PostMessageRequest, idempotency_key: Option<&str>) -> Result<PostMessageResponse> {
        let rfq_id = RfqId::new(rfq_id.to_string())?;

        // Check idempotency if key provided
        if let Some(key) = idempotency_key {
            let body_hash = self.compute_message_hash(&request)?;
            if let Some(cached_response) = self.idempotency_service.check_idempotency(key, &body_hash).await? {
                return Ok(serde_json::from_str(&cached_response)
                    .map_err(|_| DomainError::Internal("Failed to deserialize cached response".to_string()))?);
            }
        }

        let message_body = MessageBody::new(request.body.clone())?;

        // Verify RFQ exists
        let rfq_meta = self.rfq_repository.get_rfq_meta(&rfq_id).await?
            .ok_or_else(|| DomainError::NotFound("RFQ not found".to_string()))?;

        // Parse author
        let author = match request.by.as_str() {
            "buyer" => EventAuthor::Buyer,
            "manufacturer" => EventAuthor::Manufacturer,
            _ => return Err(DomainError::ValidationFailed("Invalid author role".to_string())),
        };

        // Create message event
        let message_event = RfqEvent::new_message(
            rfq_id.as_str().to_string(),
            author,
            message_body.as_str().to_string(),
        );

        let timestamp = message_event.timestamp();

        // Save event
        self.rfq_repository.save_rfq_event(&message_event).await?;

        // Update RFQ index
        let mut index = self.rfq_repository.get_rfq_index(&rfq_id).await?
            .unwrap_or(RfqIndex { last_event_ts: timestamp, count: 0 });
        index.last_event_ts = timestamp;
        index.count += 1;
        self.rfq_repository.save_rfq_index(&rfq_id, &index).await?;

        // Send notification
        self.email_service.send_rfq_message_notification(&rfq_meta, &message_event).await?;

        let response = PostMessageResponse {
            ts: timestamp.to_rfc3339(),
        };

        // Store idempotency result if key provided
        if let Some(key) = idempotency_key {
            let body_hash = self.compute_message_hash(&request)?;
            let response_json = serde_json::to_string(&response)
                .map_err(|_| DomainError::Internal("Failed to serialize response".to_string()))?;
            self.idempotency_service.store_idempotency(key, &body_hash, &response_json).await?;
        }

        Ok(response)
    }

    fn compute_request_hash(&self, request: &CreateRfqRequest) -> Result<String> {
        let json = serde_json::to_string(request)
            .map_err(|_| DomainError::Internal("Failed to serialize request".to_string()))?;
        Ok(format!("{:x}", sha2::Sha256::digest(json.as_bytes())))
    }

    fn compute_message_hash(&self, request: &PostMessageRequest) -> Result<String> {
        let json = serde_json::to_string(request)
            .map_err(|_| DomainError::Internal("Failed to serialize request".to_string()))?;
        Ok(format!("{:x}", sha2::Sha256::digest(json.as_bytes())))
    }

    fn event_to_dto(&self, event: &RfqEvent) -> RfqEventDto {
        match event {
            RfqEvent::Message(e) => RfqEventDto::Message {
                id: e.base.id.clone(),
                rfq_id: e.base.rfq_id.clone(),
                ts: e.base.ts.to_rfc3339(),
                by: format!("{:?}", e.base.by).to_lowercase(),
                body: e.body.clone(),
            },
            RfqEvent::Status(e) => RfqEventDto::Status {
                id: e.base.id.clone(),
                rfq_id: e.base.rfq_id.clone(),
                ts: e.base.ts.to_rfc3339(),
                by: format!("{:?}", e.base.by).to_lowercase(),
                status: format!("{:?}", e.status).to_lowercase(),
                note: e.note.clone(),
            },
            RfqEvent::Attachment(e) => RfqEventDto::Attachment {
                id: e.base.id.clone(),
                rfq_id: e.base.rfq_id.clone(),
                ts: e.base.ts.to_rfc3339(),
                by: format!("{:?}", e.base.by).to_lowercase(),
                attachments: e.attachments.iter().map(|a| a.clone().into()).collect(),
            },
        }
    }
}