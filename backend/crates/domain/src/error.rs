use thiserror::Error;

#[derive(Error, Debug)]
pub enum DomainError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, DomainError>;
