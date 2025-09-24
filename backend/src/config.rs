//! Configuration management for Terra marketplace backend

use serde::Deserialize;
use std::env;

/// Application configuration
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
    pub aws: AwsConfig,
    pub email: EmailConfig,
    pub storage: StorageConfig,
    pub payment: PaymentConfig,
    pub app: AppConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub dynamodb_table_prefix: String,
    pub region: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_expiration_hours: u64,
    pub password_salt_rounds: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AwsConfig {
    pub region: String,
    pub access_key_id: Option<String>,
    pub secret_access_key: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EmailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub from_email: String,
    pub from_name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StorageConfig {
    pub s3_bucket: String,
    pub s3_region: String,
    pub max_file_size_mb: usize,
    pub allowed_file_types: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PaymentConfig {
    pub stripe_secret_key: String,
    pub stripe_webhook_secret: String,
    pub currency: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub name: String,
    pub version: String,
    pub environment: Environment,
    pub cors_origins: Vec<String>,
    pub rate_limit_requests_per_minute: u32,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Development,
    Staging,
    Production,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, crate::errors::ApiError> {
        Ok(Config {
            database: DatabaseConfig {
                dynamodb_table_prefix: env::var("DYNAMODB_TABLE_PREFIX")
                    .unwrap_or_else(|_| "terra".to_string()),
                region: env::var("AWS_REGION")
                    .unwrap_or_else(|_| "us-east-1".to_string()),
            },
            auth: AuthConfig {
                jwt_secret: env::var("JWT_SECRET")
                    .map_err(|_| crate::errors::ApiError::ConfigurationError(
                        "JWT_SECRET environment variable is required".to_string()
                    ))?,
                jwt_expiration_hours: env::var("JWT_EXPIRATION_HOURS")
                    .unwrap_or_else(|_| "24".to_string())
                    .parse()
                    .map_err(|_| crate::errors::ApiError::ConfigurationError(
                        "Invalid JWT_EXPIRATION_HOURS value".to_string()
                    ))?,
                password_salt_rounds: env::var("PASSWORD_SALT_ROUNDS")
                    .unwrap_or_else(|_| "12".to_string())
                    .parse()
                    .map_err(|_| crate::errors::ApiError::ConfigurationError(
                        "Invalid PASSWORD_SALT_ROUNDS value".to_string()
                    ))?,
            },
            aws: AwsConfig {
                region: env::var("AWS_REGION")
                    .unwrap_or_else(|_| "us-east-1".to_string()),
                access_key_id: env::var("AWS_ACCESS_KEY_ID").ok(),
                secret_access_key: env::var("AWS_SECRET_ACCESS_KEY").ok(),
            },
            email: EmailConfig {
                smtp_host: env::var("SMTP_HOST")
                    .unwrap_or_else(|_| "smtp.gmail.com".to_string()),
                smtp_port: env::var("SMTP_PORT")
                    .unwrap_or_else(|_| "587".to_string())
                    .parse()
                    .map_err(|_| crate::errors::ApiError::ConfigurationError(
                        "Invalid SMTP_PORT value".to_string()
                    ))?,
                smtp_username: env::var("SMTP_USERNAME")
                    .unwrap_or_default(),
                smtp_password: env::var("SMTP_PASSWORD")
                    .unwrap_or_default(),
                from_email: env::var("FROM_EMAIL")
                    .unwrap_or_else(|_| "noreply@terra-marketplace.com".to_string()),
                from_name: env::var("FROM_NAME")
                    .unwrap_or_else(|_| "Terra Marketplace".to_string()),
            },
            storage: StorageConfig {
                s3_bucket: env::var("S3_BUCKET")
                    .unwrap_or_else(|_| "terra-marketplace-assets".to_string()),
                s3_region: env::var("S3_REGION")
                    .unwrap_or_else(|_| "us-east-1".to_string()),
                max_file_size_mb: env::var("MAX_FILE_SIZE_MB")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()
                    .map_err(|_| crate::errors::ApiError::ConfigurationError(
                        "Invalid MAX_FILE_SIZE_MB value".to_string()
                    ))?,
                allowed_file_types: env::var("ALLOWED_FILE_TYPES")
                    .unwrap_or_else(|_| "jpg,jpeg,png,gif,webp".to_string())
                    .split(',')
                    .map(|s| s.trim().to_lowercase())
                    .collect(),
            },
            payment: PaymentConfig {
                stripe_secret_key: env::var("STRIPE_SECRET_KEY")
                    .unwrap_or_default(),
                stripe_webhook_secret: env::var("STRIPE_WEBHOOK_SECRET")
                    .unwrap_or_default(),
                currency: env::var("CURRENCY")
                    .unwrap_or_else(|_| "usd".to_string()),
            },
            app: AppConfig {
                name: env::var("APP_NAME")
                    .unwrap_or_else(|_| "Terra Marketplace".to_string()),
                version: env!("CARGO_PKG_VERSION").to_string(),
                environment: env::var("ENVIRONMENT")
                    .unwrap_or_else(|_| "development".to_string())
                    .parse()
                    .unwrap_or(Environment::Development),
                cors_origins: env::var("CORS_ORIGINS")
                    .unwrap_or_else(|_| "http://localhost:3000,https://jprier.github.io".to_string())
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect(),
                rate_limit_requests_per_minute: env::var("RATE_LIMIT_RPM")
                    .unwrap_or_else(|_| "100".to_string())
                    .parse()
                    .map_err(|_| crate::errors::ApiError::ConfigurationError(
                        "Invalid RATE_LIMIT_RPM value".to_string()
                    ))?,
            },
        })
    }

    /// Get database table name with prefix
    pub fn table_name(&self, table: &str) -> String {
        format!("{}_{}", self.database.dynamodb_table_prefix, table)
    }

    /// Check if we're in production environment
    pub fn is_production(&self) -> bool {
        self.app.environment == Environment::Production
    }

    /// Check if we're in development environment
    pub fn is_development(&self) -> bool {
        self.app.environment == Environment::Development
    }
}

impl std::str::FromStr for Environment {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "development" | "dev" => Ok(Environment::Development),
            "staging" | "stage" => Ok(Environment::Staging),
            "production" | "prod" => Ok(Environment::Production),
            _ => Err(format!("Invalid environment: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_parsing() {
        assert_eq!("development".parse::<Environment>().unwrap(), Environment::Development);
        assert_eq!("staging".parse::<Environment>().unwrap(), Environment::Staging);
        assert_eq!("production".parse::<Environment>().unwrap(), Environment::Production);
        assert!("invalid".parse::<Environment>().is_err());
    }

    #[test]
    fn test_table_name() {
        let config = Config {
            database: DatabaseConfig {
                dynamodb_table_prefix: "test".to_string(),
                region: "us-east-1".to_string(),
            },
            auth: AuthConfig {
                jwt_secret: "secret".to_string(),
                jwt_expiration_hours: 24,
                password_salt_rounds: 12,
            },
            aws: AwsConfig {
                region: "us-east-1".to_string(),
                access_key_id: None,
                secret_access_key: None,
            },
            email: EmailConfig {
                smtp_host: "smtp.gmail.com".to_string(),
                smtp_port: 587,
                smtp_username: "".to_string(),
                smtp_password: "".to_string(),
                from_email: "test@example.com".to_string(),
                from_name: "Test".to_string(),
            },
            storage: StorageConfig {
                s3_bucket: "test-bucket".to_string(),
                s3_region: "us-east-1".to_string(),
                max_file_size_mb: 10,
                allowed_file_types: vec!["jpg".to_string(), "png".to_string()],
            },
            payment: PaymentConfig {
                stripe_secret_key: "".to_string(),
                stripe_webhook_secret: "".to_string(),
                currency: "usd".to_string(),
            },
            app: AppConfig {
                name: "Test App".to_string(),
                version: "1.0.0".to_string(),
                environment: Environment::Development,
                cors_origins: vec!["http://localhost:3000".to_string()],
                rate_limit_requests_per_minute: 100,
            },
        };

        assert_eq!(config.table_name("products"), "test_products");
        assert!(config.is_development());
        assert!(!config.is_production());
    }
}