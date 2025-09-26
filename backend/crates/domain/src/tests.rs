use super::*;
use crate::value_objects::*;
use crate::entities::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_validation() {
        // Valid emails
        assert!(Email::new("test@example.com").is_ok());
        assert!(Email::new("user+tag@domain.co.uk").is_ok());
        
        // Invalid emails
        assert!(Email::new("invalid-email").is_err());
        assert!(Email::new("@example.com").is_err());
        assert!(Email::new("test@").is_err());
        assert!(Email::new("").is_err());
    }

    #[test]
    fn test_tenant_id_generation() {
        let tenant_id = TenantId::new();
        assert!(!tenant_id.as_str().is_empty());
        
        let tenant_id2 = TenantId::new();
        assert_ne!(tenant_id.as_str(), tenant_id2.as_str());
    }

    #[test]
    fn test_rfq_id_generation() {
        let rfq_id = RfqId::new();
        assert!(rfq_id.as_str().starts_with("rfq_"));
        
        let rfq_id2 = RfqId::new();
        assert_ne!(rfq_id.as_str(), rfq_id2.as_str());
    }

    #[test]
    fn test_manufacturer_id_generation() {
        let mfg_id = ManufacturerId::new();
        assert!(mfg_id.as_str().starts_with("mfg_"));
        
        let mfg_id2 = ManufacturerId::new();
        assert_ne!(mfg_id.as_str(), mfg_id2.as_str());
    }

    #[test]
    fn test_content_type_validation() {
        // Valid content types
        assert!(ContentType::new("image/jpeg").is_ok());
        assert!(ContentType::new("image/png").is_ok());
        assert!(ContentType::new("application/pdf").is_ok());
        
        // Invalid content types
        assert!(ContentType::new("application/javascript").is_err());
        assert!(ContentType::new("text/html").is_err());
        assert!(ContentType::new("").is_err());
    }

    #[test]
    fn test_file_size_validation() {
        // Valid sizes
        assert!(FileSize::new(1024).is_ok());
        assert!(FileSize::new(15 * 1024 * 1024).is_ok()); // 15 MB
        
        // Invalid sizes
        assert!(FileSize::new(0).is_err());
        assert!(FileSize::new(16 * 1024 * 1024).is_err()); // 16 MB (too large)
    }

    #[test]
    fn test_money_formatting() {
        let price = Money::from_cents(12345);
        assert_eq!(price.cents(), 12345);
        assert_eq!(price.dollars(), 123.45);
        
        let zero = Money::from_cents(0);
        assert_eq!(zero.dollars(), 0.0);
    }

    #[test]
    fn test_location_creation() {
        let location = Location::new("Columbus".to_string(), "OH".to_string());
        assert_eq!(location.city(), "Columbus");
        assert_eq!(location.state(), "OH");
    }

    #[test]
    fn test_phone_number_validation() {
        // Valid phone numbers
        assert!(PhoneNumber::new("(555) 123-4567").is_ok());
        assert!(PhoneNumber::new("555-123-4567").is_ok());
        assert!(PhoneNumber::new("5551234567").is_ok());
        
        // Invalid phone numbers
        assert!(PhoneNumber::new("123").is_err());
        assert!(PhoneNumber::new("").is_err());
        assert!(PhoneNumber::new("abc-def-ghij").is_err());
    }
}