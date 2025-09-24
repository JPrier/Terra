//! Utility functions for the Terra marketplace backend

use uuid::Uuid;
use crate::{Result, ApiError};

/// Generate a new UUID v4
pub fn generate_id() -> Uuid {
    Uuid::new_v4()
}

/// Validate email format
pub fn is_valid_email(email: &str) -> bool {
    email.contains('@') && email.contains('.') && email.len() > 5
}

/// Sanitize string input
pub fn sanitize_string(input: &str) -> String {
    input.trim().to_string()
}

/// Calculate price with tax
pub fn calculate_price_with_tax(price: f64, tax_rate: f64) -> f64 {
    price * (1.0 + tax_rate)
}

/// Generate slug from title
pub fn generate_slug(title: &str) -> String {
    title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<&str>>()
        .join("-")
}

/// Validate file extension
pub fn is_valid_file_extension(filename: &str, allowed_extensions: &[String]) -> bool {
    if let Some(extension) = filename.split('.').last() {
        allowed_extensions.contains(&extension.to_lowercase())
    } else {
        false
    }
}

/// Format currency amount
pub fn format_currency(amount: f64, currency: &str) -> String {
    match currency.to_uppercase().as_str() {
        "USD" => format!("${:.2}", amount),
        "EUR" => format!("€{:.2}", amount),
        "GBP" => format!("£{:.2}", amount),
        _ => format!("{:.2} {}", amount, currency),
    }
}

/// Parse pagination parameters with defaults
pub fn parse_pagination_params(page: Option<u32>, limit: Option<u32>) -> (u32, u32) {
    let page = page.unwrap_or(1).max(1);
    let limit = limit.unwrap_or(20).min(100).max(1);
    (page, limit)
}

/// Calculate offset from page and limit
pub fn calculate_offset(page: u32, limit: u32) -> u32 {
    (page - 1) * limit
}

/// Validate password strength
pub fn validate_password_strength(password: &str) -> Result<()> {
    if password.len() < 8 {
        return Err(ApiError::ValidationError("Password must be at least 8 characters long".to_string()));
    }

    if !password.chars().any(|c| c.is_uppercase()) {
        return Err(ApiError::ValidationError("Password must contain at least one uppercase letter".to_string()));
    }

    if !password.chars().any(|c| c.is_lowercase()) {
        return Err(ApiError::ValidationError("Password must contain at least one lowercase letter".to_string()));
    }

    if !password.chars().any(|c| c.is_numeric()) {
        return Err(ApiError::ValidationError("Password must contain at least one number".to_string()));
    }

    Ok(())
}

/// Generate a random string
pub fn generate_random_string(length: usize) -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";
    
    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_email() {
        assert!(is_valid_email("test@example.com"));
        assert!(is_valid_email("user.name@domain.co.uk"));
        assert!(!is_valid_email("invalid"));
        assert!(!is_valid_email("@domain.com"));
        assert!(!is_valid_email("user@"));
    }

    #[test]
    fn test_sanitize_string() {
        assert_eq!(sanitize_string("  hello world  "), "hello world");
        assert_eq!(sanitize_string("\t\ntest\r\n"), "test");
    }

    #[test]
    fn test_calculate_price_with_tax() {
        assert_eq!(calculate_price_with_tax(100.0, 0.1), 110.0);
        assert_eq!(calculate_price_with_tax(50.0, 0.08), 54.0);
    }

    #[test]
    fn test_generate_slug() {
        assert_eq!(generate_slug("Hello World!"), "hello-world");
        assert_eq!(generate_slug("Product Name 123"), "product-name-123");
        assert_eq!(generate_slug("Special@#$Characters"), "special-characters");
    }

    #[test]
    fn test_is_valid_file_extension() {
        let allowed = vec!["jpg".to_string(), "png".to_string(), "gif".to_string()];
        assert!(is_valid_file_extension("image.jpg", &allowed));
        assert!(is_valid_file_extension("photo.PNG", &allowed));
        assert!(!is_valid_file_extension("document.pdf", &allowed));
        assert!(!is_valid_file_extension("noextension", &allowed));
    }

    #[test]
    fn test_format_currency() {
        assert_eq!(format_currency(123.45, "USD"), "$123.45");
        assert_eq!(format_currency(67.89, "EUR"), "€67.89");
        assert_eq!(format_currency(45.67, "GBP"), "£45.67");
        assert_eq!(format_currency(100.0, "JPY"), "100.00 JPY");
    }

    #[test]
    fn test_parse_pagination_params() {
        assert_eq!(parse_pagination_params(Some(1), Some(10)), (1, 10));
        assert_eq!(parse_pagination_params(None, None), (1, 20));
        assert_eq!(parse_pagination_params(Some(0), Some(200)), (1, 100));
    }

    #[test]
    fn test_calculate_offset() {
        assert_eq!(calculate_offset(1, 10), 0);
        assert_eq!(calculate_offset(2, 10), 10);
        assert_eq!(calculate_offset(3, 25), 50);
    }

    #[test]
    fn test_validate_password_strength() {
        assert!(validate_password_strength("Password123").is_ok());
        assert!(validate_password_strength("short").is_err());
        assert!(validate_password_strength("alllowercase123").is_err());
        assert!(validate_password_strength("ALLUPPERCASE123").is_err());
        assert!(validate_password_strength("NoNumbers").is_err());
    }

    #[test]
    fn test_generate_random_string() {
        let s1 = generate_random_string(10);
        let s2 = generate_random_string(10);
        assert_eq!(s1.len(), 10);
        assert_eq!(s2.len(), 10);
        assert_ne!(s1, s2); // Very unlikely to be the same
    }
}