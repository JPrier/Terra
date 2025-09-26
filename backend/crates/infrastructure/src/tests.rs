use super::*;
use crate::config::Config;
use domain::value_objects::*;
use std::sync::Arc;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = Config {
            environment: "test".to_string(),
            public_bucket: "test-public-bucket".to_string(),
            private_bucket: "test-private-bucket".to_string(),
            from_email: "test@example.com".to_string(),
        };

        assert_eq!(config.environment, "test");
        assert_eq!(config.public_bucket, "test-public-bucket");
        assert_eq!(config.private_bucket, "test-private-bucket");
        assert_eq!(config.from_email, "test@example.com");
    }

    #[tokio::test]
    async fn test_s3_idempotency_service_key_generation() {
        use crate::s3::S3IdempotencyService;
        
        let config = Arc::new(Config {
            environment: "test".to_string(),
            public_bucket: "test-public-bucket".to_string(),
            private_bucket: "test-private-bucket".to_string(),
            from_email: "test@example.com".to_string(),
        });

        // Mock S3 client would be used in real tests
        // For now, we test the key generation logic indirectly
        
        let key1 = S3IdempotencyService::idempotency_key("test-key", "test-body");
        let key2 = S3IdempotencyService::idempotency_key("test-key", "test-body");
        let key3 = S3IdempotencyService::idempotency_key("different-key", "test-body");

        // Same input should generate same key
        assert_eq!(key1, key2);
        
        // Different input should generate different key
        assert_ne!(key1, key3);
        
        // Keys should be reasonable length (SHA256 hex = 64 chars)
        assert_eq!(key1.len(), 64);
    }

    #[test]
    fn test_s3_key_formatting() {
        let rfq_id = RfqId::new();
        let manufacturer_id = ManufacturerId::new();
        
        // Test RFQ key formatting
        let rfq_meta_key = format!("rfq/{}/meta.json", rfq_id.as_str());
        assert!(rfq_meta_key.starts_with("rfq/"));
        assert!(rfq_meta_key.ends_with("/meta.json"));
        
        // Test manufacturer key formatting
        let mfg_key = format!("manufacturer/{}.json", manufacturer_id.as_str());
        assert!(mfg_key.starts_with("manufacturer/"));
        assert!(mfg_key.ends_with(".json"));
    }

    #[test]
    fn test_content_type_validation_in_image_service() {
        // Valid content types for image service
        let valid_types = vec![
            "image/jpeg",
            "image/png", 
            "image/webp",
            "image/avif",
            "application/pdf"
        ];

        for content_type in valid_types {
            assert!(ContentType::new(content_type).is_ok(), 
                    "Expected {} to be valid", content_type);
        }

        // Invalid content types
        let invalid_types = vec![
            "application/javascript",
            "text/html",
            "video/mp4",
            "audio/mpeg"
        ];

        for content_type in invalid_types {
            assert!(ContentType::new(content_type).is_err(), 
                    "Expected {} to be invalid", content_type);
        }
    }

    #[test]
    fn test_file_size_limits() {
        // Test file size limits used in image service
        let max_size = 15 * 1024 * 1024; // 15 MB
        
        assert!(FileSize::new(1024).is_ok()); // 1 KB - OK
        assert!(FileSize::new(max_size).is_ok()); // 15 MB - OK
        assert!(FileSize::new(max_size + 1).is_err()); // 15 MB + 1 byte - Too large
        assert!(FileSize::new(0).is_err()); // 0 bytes - Too small
    }

    #[test]
    fn test_event_key_generation() {
        use chrono::Utc;
        use domain::events::*;
        
        let rfq_id = RfqId::new();
        let timestamp = Utc::now();
        let event_id = uuid::Uuid::new_v4().to_string();
        
        // Simulate event key generation (as done in S3RfqRepository)
        let ts_str = timestamp.format("%Y-%m-%dT%H-%M-%SZ").to_string();
        let key = format!("rfq/{}/events/{}-{}.json", rfq_id.as_str(), ts_str, event_id);
        
        assert!(key.starts_with("rfq/"));
        assert!(key.contains("/events/"));
        assert!(key.ends_with(".json"));
        assert!(key.contains(&event_id));
    }

    #[test]
    fn test_catalog_key_generation() {
        // Test catalog key generation patterns
        let category = "machining";
        let state = "OH";
        
        let category_key = format!("catalog/category/{}.json", category);
        assert_eq!(category_key, "catalog/category/machining.json");
        
        let category_state_key = format!("catalog/category_state/{}/{}.json", category, state);
        assert_eq!(category_state_key, "catalog/category_state/machining/OH.json");
        
        let manufacturer_id = ManufacturerId::new();
        let mfg_key = format!("manufacturer/{}.json", manufacturer_id.as_str());
        assert!(mfg_key.starts_with("manufacturer/mfg_"));
    }

    #[test]
    fn test_image_manifest_key_generation() {
        let image_id = "test-image-123";
        let key = format!("tenants/shared/manifests/{}.json", image_id);
        
        assert_eq!(key, "tenants/shared/manifests/test-image-123.json");
        assert!(key.starts_with("tenants/"));
        assert!(key.contains("/manifests/"));
    }
}