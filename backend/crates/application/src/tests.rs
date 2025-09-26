use super::*;
use domain::entities::*;
use domain::events::*;
use domain::value_objects::*;
use std::sync::Arc;

// Mock repository for testing
struct MockRfqRepository {
    rfqs: std::collections::HashMap<String, RfqMeta>,
    events: std::collections::HashMap<String, Vec<RfqEvent>>,
}

impl MockRfqRepository {
    fn new() -> Self {
        Self {
            rfqs: std::collections::HashMap::new(),
            events: std::collections::HashMap::new(),
        }
    }
}

#[async_trait::async_trait]
impl crate::ports::RfqRepository for MockRfqRepository {
    async fn save_rfq_meta(&self, rfq: &RfqMeta) -> domain::error::Result<()> {
        // In a real mock, we'd use interior mutability
        Ok(())
    }

    async fn get_rfq_meta(&self, id: &RfqId) -> domain::error::Result<Option<RfqMeta>> {
        Ok(None)
    }

    async fn save_rfq_index(&self, rfq_id: &RfqId, index: &RfqIndex) -> domain::error::Result<()> {
        Ok(())
    }

    async fn get_rfq_index(&self, id: &RfqId) -> domain::error::Result<Option<RfqIndex>> {
        Ok(None)
    }

    async fn save_rfq_event(&self, event: &RfqEvent) -> domain::error::Result<()> {
        Ok(())
    }

    async fn list_rfq_events(&self, rfq_id: &RfqId, since: Option<chrono::DateTime<chrono::Utc>>, limit: Option<u32>) -> domain::error::Result<Vec<RfqEvent>> {
        Ok(vec![])
    }
}

// Mock email service for testing
struct MockEmailService;

#[async_trait::async_trait]
impl crate::ports::EmailService for MockEmailService {
    async fn send_rfq_created_notification(&self, rfq: &RfqMeta) -> domain::error::Result<()> {
        Ok(())
    }

    async fn send_rfq_message_notification(&self, rfq: &RfqMeta, event: &RfqEvent) -> domain::error::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    #[tokio::test]
    async fn test_create_rfq_request_validation() {
        let request = CreateRfqRequest {
            tenant_id: TenantId::new(),
            manufacturer_id: ManufacturerId::new(),
            buyer: BuyerInfo {
                email: Email::new("buyer@example.com").unwrap(),
                name: Some("Test Buyer".to_string()),
            },
            subject: "Test RFQ".to_string(),
            body: "This is a test RFQ request".to_string(),
            attachments: vec![],
        };

        assert_eq!(request.subject, "Test RFQ");
        assert_eq!(request.body, "This is a test RFQ request");
        assert!(request.attachments.is_empty());
    }

    #[tokio::test]
    async fn test_rfq_service_initialization() {
        let rfq_repo = Arc::new(MockRfqRepository::new());
        let email_service = Arc::new(MockEmailService);
        
        let service = RfqService::new(rfq_repo.clone(), email_service.clone());
        
        // Service should be created successfully
        assert!(Arc::strong_count(&rfq_repo) >= 2); // Service holds a reference
    }

    #[test]
    fn test_attachment_info_creation() {
        let attachment = AttachmentInfo {
            upload_key: "test-key".to_string(),
            file_name: "document.pdf".to_string(),
            content_type: ContentType::new("application/pdf").unwrap(),
            size_bytes: FileSize::new(1024).unwrap(),
        };

        assert_eq!(attachment.file_name, "document.pdf");
        assert_eq!(attachment.upload_key, "test-key");
        assert_eq!(attachment.content_type.as_str(), "application/pdf");
        assert_eq!(attachment.size_bytes.bytes(), 1024);
    }

    #[test]
    fn test_create_rfq_response() {
        let rfq_id = RfqId::new();
        let timestamp = chrono::Utc::now();
        
        let response = CreateRfqResponse {
            id: rfq_id.clone(),
            last_event_ts: timestamp,
        };

        assert_eq!(response.id, rfq_id);
        assert_eq!(response.last_event_ts, timestamp);
    }

    #[test]
    fn test_list_events_request() {
        let rfq_id = RfqId::new();
        let since = Some(chrono::Utc::now());
        
        let request = ListEventsRequest {
            rfq_id: rfq_id.clone(),
            since,
            limit: Some(50),
        };

        assert_eq!(request.rfq_id, rfq_id);
        assert!(request.since.is_some());
        assert_eq!(request.limit, Some(50));
    }

    #[test]
    fn test_send_message_request() {
        let request = SendMessageRequest {
            rfq_id: RfqId::new(),
            sender: MessageSender::Buyer,
            body: "Hello manufacturer!".to_string(),
            attachments: vec![],
        };

        assert!(matches!(request.sender, MessageSender::Buyer));
        assert_eq!(request.body, "Hello manufacturer!");
        assert!(request.attachments.is_empty());
    }
}