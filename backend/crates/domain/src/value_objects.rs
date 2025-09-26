use regex::Regex;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::error::{DomainError, Result};

/// Email address value object with validation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Email(String);

impl Email {
    pub fn new(email: String) -> Result<Self> {
        let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
            .map_err(|_| DomainError::Internal("Failed to compile email regex".to_string()))?;

        if !email_regex.is_match(&email) {
            return Err(DomainError::ValidationFailed(
                "Invalid email format".to_string(),
            ));
        }

        Ok(Email(email))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for Email {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self> {
        Email::new(s.to_string())
    }
}

/// Tenant ID value object
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TenantId(String);

impl TenantId {
    pub fn new(id: String) -> Result<Self> {
        if id.is_empty() || id.len() > 50 {
            return Err(DomainError::ValidationFailed(
                "Tenant ID must be 1-50 characters".to_string(),
            ));
        }

        let id_regex = Regex::new(r"^[a-zA-Z0-9_-]+$")
            .map_err(|_| DomainError::Internal("Failed to compile ID regex".to_string()))?;

        if !id_regex.is_match(&id) {
            return Err(DomainError::ValidationFailed(
                "Tenant ID can only contain alphanumeric characters, underscores, and hyphens"
                    .to_string(),
            ));
        }

        Ok(TenantId(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// RFQ ID value object
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RfqId(String);

impl RfqId {
    pub fn new(id: String) -> Result<Self> {
        if id.is_empty() || id.len() > 50 {
            return Err(DomainError::ValidationFailed(
                "RFQ ID must be 1-50 characters".to_string(),
            ));
        }

        Ok(RfqId(id))
    }

    pub fn generate() -> Self {
        let id = format!(
            "r_{}",
            &uuid::Uuid::new_v4().simple().to_string()[0..8].to_uppercase()
        );
        RfqId(id)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Manufacturer ID value object
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ManufacturerId(String);

impl ManufacturerId {
    pub fn new(id: String) -> Result<Self> {
        if id.is_empty() || id.len() > 50 {
            return Err(DomainError::ValidationFailed(
                "Manufacturer ID must be 1-50 characters".to_string(),
            ));
        }

        let id_regex = Regex::new(r"^mfg_[a-zA-Z0-9_-]+$").map_err(|_| {
            DomainError::Internal("Failed to compile manufacturer ID regex".to_string())
        })?;

        if !id_regex.is_match(&id) {
            return Err(DomainError::ValidationFailed("Manufacturer ID must start with 'mfg_' and contain only alphanumeric characters, underscores, and hyphens".to_string()));
        }

        Ok(ManufacturerId(id))
    }

    pub fn generate() -> Self {
        let id = format!("mfg_{}", &uuid::Uuid::new_v4().simple().to_string()[0..8]);
        ManufacturerId(id)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// S3 key value object with validation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct S3Key(String);

impl S3Key {
    pub fn new(key: String) -> Result<Self> {
        if key.is_empty() || key.len() > 1024 {
            return Err(DomainError::ValidationFailed(
                "S3 key must be 1-1024 characters".to_string(),
            ));
        }

        // Basic validation - no leading slash, no double slashes
        if key.starts_with('/') || key.contains("//") {
            return Err(DomainError::ValidationFailed(
                "Invalid S3 key format".to_string(),
            ));
        }

        Ok(S3Key(key))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Content type value object with validation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContentType(String);

impl ContentType {
    pub fn new(content_type: String) -> Result<Self> {
        // Validate against allowed content types from design
        match content_type.as_str() {
            "image/jpeg" | "image/png" | "image/webp" | "image/avif" | "application/pdf" => {
                Ok(ContentType(content_type))
            }
            _ => Err(DomainError::ValidationFailed(format!(
                "Content type '{}' is not allowed",
                content_type
            ))),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_image(&self) -> bool {
        self.0.starts_with("image/")
    }
}

/// File size validation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileSize(u64);

impl FileSize {
    const MAX_SIZE_BYTES: u64 = 15 * 1024 * 1024; // 15 MB

    pub fn new(size_bytes: u64) -> Result<Self> {
        if size_bytes == 0 {
            return Err(DomainError::ValidationFailed(
                "File size cannot be zero".to_string(),
            ));
        }

        if size_bytes > Self::MAX_SIZE_BYTES {
            return Err(DomainError::ValidationFailed(format!(
                "File size {} exceeds maximum of {} bytes",
                size_bytes,
                Self::MAX_SIZE_BYTES
            )));
        }

        Ok(FileSize(size_bytes))
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

/// Message body validation (max 8000 characters)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MessageBody(String);

impl MessageBody {
    const MAX_LENGTH: usize = 8000;

    pub fn new(body: String) -> Result<Self> {
        if body.is_empty() {
            return Err(DomainError::ValidationFailed(
                "Message body cannot be empty".to_string(),
            ));
        }

        if body.len() > Self::MAX_LENGTH {
            return Err(DomainError::ValidationFailed(format!(
                "Message body exceeds maximum length of {} characters",
                Self::MAX_LENGTH
            )));
        }

        // Basic validation - no HTML tags allowed
        if body.contains('<') && body.contains('>') {
            return Err(DomainError::ValidationFailed(
                "HTML tags are not allowed in message body".to_string(),
            ));
        }

        Ok(MessageBody(body))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
