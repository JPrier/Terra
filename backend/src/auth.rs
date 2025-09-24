//! Authentication and authorization for Terra marketplace

use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::{ApiError, Result};

/// JWT Claims structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // User ID
    pub email: String,
    pub username: String,
    pub is_seller: bool,
    pub exp: usize, // Expiration time
    pub iat: usize, // Issued at time
}

/// Authentication service
pub struct AuthService {
    jwt_secret: String,
    jwt_expiration_hours: u64,
}

impl AuthService {
    pub fn new(jwt_secret: String, jwt_expiration_hours: u64) -> Self {
        Self {
            jwt_secret,
            jwt_expiration_hours,
        }
    }

    /// Generate JWT token for user
    pub fn generate_token(&self, user_id: Uuid, email: &str, username: &str, is_seller: bool) -> Result<String> {
        let now = chrono::Utc::now();
        let exp = now + chrono::Duration::hours(self.jwt_expiration_hours as i64);

        let claims = Claims {
            sub: user_id.to_string(),
            email: email.to_string(),
            username: username.to_string(),
            is_seller,
            exp: exp.timestamp() as usize,
            iat: now.timestamp() as usize,
        };

        let header = Header::default();
        let encoding_key = EncodingKey::from_secret(self.jwt_secret.as_ref());

        encode(&header, &claims, &encoding_key)
            .map_err(|e| ApiError::InternalServer(format!("Failed to generate token: {}", e)))
    }

    /// Validate and decode JWT token
    pub fn validate_token(&self, token: &str) -> Result<Claims> {
        let decoding_key = DecodingKey::from_secret(self.jwt_secret.as_ref());
        let validation = Validation::default();

        decode::<Claims>(token, &decoding_key, &validation)
            .map(|token_data| token_data.claims)
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                    ApiError::AuthenticationFailed("Token has expired".to_string())
                }
                _ => ApiError::AuthenticationFailed(format!("Invalid token: {}", e)),
            })
    }

    /// Extract token from Authorization header
    pub fn extract_token_from_header(auth_header: &str) -> Result<&str> {
        if auth_header.starts_with("Bearer ") {
            Ok(&auth_header[7..])
        } else {
            Err(ApiError::AuthenticationFailed(
                "Invalid authorization header format".to_string(),
            ))
        }
    }

    /// Hash password using bcrypt
    pub fn hash_password(&self, password: &str) -> Result<String> {
        bcrypt::hash(password, bcrypt::DEFAULT_COST)
            .map_err(|e| ApiError::InternalServer(format!("Failed to hash password: {}", e)))
    }

    /// Verify password against hash
    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool> {
        bcrypt::verify(password, hash)
            .map_err(|e| ApiError::InternalServer(format!("Failed to verify password: {}", e)))
    }
}

/// Middleware for JWT authentication
pub fn require_auth(auth_header: Option<&str>, auth_service: &AuthService) -> Result<Claims> {
    let auth_header = auth_header
        .ok_or_else(|| ApiError::AuthenticationFailed("Missing authorization header".to_string()))?;

    let token = AuthService::extract_token_from_header(auth_header)?;
    auth_service.validate_token(token)
}

/// Middleware for seller authorization
pub fn require_seller(claims: &Claims) -> Result<()> {
    if claims.is_seller {
        Ok(())
    } else {
        Err(ApiError::AuthorizationFailed(
            "Seller privileges required".to_string(),
        ))
    }
}

/// User ID from claims
pub fn user_id_from_claims(claims: &Claims) -> Result<Uuid> {
    Uuid::parse_str(&claims.sub)
        .map_err(|_| ApiError::InternalServer("Invalid user ID in token".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_service() {
        let auth_service = AuthService::new("test_secret".to_string(), 24);
        let user_id = Uuid::new_v4();
        
        // Test token generation
        let token = auth_service
            .generate_token(user_id, "test@example.com", "testuser", false)
            .unwrap();
        
        assert!(!token.is_empty());
        
        // Test token validation
        let claims = auth_service.validate_token(&token).unwrap();
        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.email, "test@example.com");
        assert_eq!(claims.username, "testuser");
        assert!(!claims.is_seller);
    }

    #[test]
    fn test_password_hashing() {
        let auth_service = AuthService::new("test_secret".to_string(), 24);
        let password = "test_password";
        
        let hash = auth_service.hash_password(password).unwrap();
        assert!(auth_service.verify_password(password, &hash).unwrap());
        assert!(!auth_service.verify_password("wrong_password", &hash).unwrap());
    }

    #[test]
    fn test_extract_token_from_header() {
        let header = "Bearer eyJhbGciOiJIUzI1NiJ9.test";
        let token = AuthService::extract_token_from_header(header).unwrap();
        assert_eq!(token, "eyJhbGciOiJIUzI1NiJ9.test");
        
        let invalid_header = "Invalid header";
        assert!(AuthService::extract_token_from_header(invalid_header).is_err());
    }

    #[test]
    fn test_require_seller() {
        let seller_claims = Claims {
            sub: Uuid::new_v4().to_string(),
            email: "seller@example.com".to_string(),
            username: "seller".to_string(),
            is_seller: true,
            exp: 0,
            iat: 0,
        };
        
        assert!(require_seller(&seller_claims).is_ok());
        
        let buyer_claims = Claims {
            sub: Uuid::new_v4().to_string(),
            email: "buyer@example.com".to_string(),
            username: "buyer".to_string(),
            is_seller: false,
            exp: 0,
            iat: 0,
        };
        
        assert!(require_seller(&buyer_claims).is_err());
    }
}