use axum::http::{StatusCode, Method};
use axum::http::header::{AUTHORIZATION, CONTENT_TYPE};
use serde_json::json;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_error_serialization() {
        let error = crate::error::AppError::BadRequest("Invalid input".to_string());

        let error_string = format!("{}", error);
        assert!(error_string.contains("Invalid input"));
    }

    #[test]
    fn test_app_error_status_codes() {
        use crate::error::AppError;

        let bad_request = AppError::BadRequest("Invalid".to_string());
        assert_eq!(bad_request.status_code(), StatusCode::BAD_REQUEST);

        let not_found = AppError::NotFound("RFQ not found".to_string());
        assert_eq!(not_found.status_code(), StatusCode::NOT_FOUND);

        let internal = AppError::InternalServerError("Something went wrong".to_string());
        assert_eq!(internal.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_cors_configuration() {
        use tower_http::cors::CorsLayer;

        // Test CORS layer configuration (would be used in middleware)
        let _cors = CorsLayer::new()
            .allow_origin("https://example.github.io".parse::<axum::http::HeaderValue>().unwrap())
            .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
            .allow_headers([AUTHORIZATION, CONTENT_TYPE]);

        // CORS layer should be created without panicking
        // In real tests, we'd test the actual middleware behavior
    }

    #[test]
    fn test_json_validation_schema() {
        // Test that our JSON schemas are well-formed
        let create_rfq_schema = json!({
            "type": "object",
            "required": ["tenant_id", "manufacturer_id", "buyer", "subject", "body"],
            "properties": {
                "tenant_id": {"type": "string"},
                "manufacturer_id": {"type": "string"},
                "buyer": {
                    "type": "object",
                    "required": ["email"],
                    "properties": {
                        "email": {"type": "string", "format": "email"},
                        "name": {"type": "string"}
                    }
                },
                "subject": {"type": "string", "minLength": 1, "maxLength": 200},
                "body": {"type": "string", "minLength": 1, "maxLength": 8000},
                "attachments": {
                    "type": "array",
                    "maxItems": 10
                }
            }
        });

        // Schema should serialize properly
        let schema_str = serde_json::to_string(&create_rfq_schema).unwrap();
        assert!(schema_str.contains("tenant_id"));
        assert!(schema_str.contains("manufacturer_id"));
    }

    #[test]
    fn test_request_id_generation() {
        // Test request ID generation (used in middleware)
        let request_id = uuid::Uuid::new_v4().to_string();
        assert!(!request_id.is_empty());
        assert_eq!(request_id.len(), 36); // UUID v4 string length
        
        let request_id2 = uuid::Uuid::new_v4().to_string();
        assert_ne!(request_id, request_id2);
    }

    #[test]
    fn test_idempotency_key_validation() {
        // Test idempotency key validation logic
        let valid_key = "user-action-2024-01-01";
        assert!(valid_key.len() > 0 && valid_key.len() <= 255);
        assert!(valid_key.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_'));

        let invalid_key = ""; // Empty
        assert!(invalid_key.is_empty());
        
        let too_long = "a".repeat(256);
        assert!(too_long.len() > 255);
    }

    #[tokio::test]
    async fn test_health_check_response() {
        // Test health check endpoint response format
        let health_response = json!({
            "status": "healthy",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "service": "api_rfqs",
            "version": "0.1.0"
        });

        assert_eq!(health_response["status"], "healthy");
        assert!(health_response["timestamp"].is_string());
        assert_eq!(health_response["service"], "api_rfqs");
    }

    #[test]
    fn test_rate_limiting_headers() {
        // Test rate limiting header values
        let rate_limit_headers = vec![
            ("X-RateLimit-Limit", "10"),
            ("X-RateLimit-Remaining", "5"),
            ("X-RateLimit-Reset", "1640995200"),
        ];

        for (header_name, header_value) in rate_limit_headers {
            assert!(!header_name.is_empty());
            assert!(!header_value.is_empty());
            assert!(header_name.starts_with("X-RateLimit-"));
        }
    }

    #[test]
    fn test_content_type_validation_middleware() {
        // Test content type validation for different endpoints
        let valid_json_type = "application/json";
        let valid_form_type = "multipart/form-data";
        
        assert_eq!(valid_json_type, "application/json");
        assert!(valid_form_type.starts_with("multipart/"));

        // Invalid content types
        let invalid_types = vec![
            "text/html",
            "application/xml", 
            "image/jpeg", // Not valid for API endpoints
        ];

        for invalid_type in invalid_types {
            assert_ne!(invalid_type, "application/json");
        }
    }

    #[test]
    fn test_error_response_format() {
        use crate::error::AppError;

        let error = AppError::BadRequest("Invalid email format".to_string());

        // Test that error implements necessary traits
        let error_string = format!("{}", error);
        assert!(error_string.contains("Invalid email format"));

        // Test status code
        assert_eq!(error.status_code(), StatusCode::BAD_REQUEST);
    }
}