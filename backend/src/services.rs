//! Business logic services for the Terra marketplace

pub mod product_service;
pub mod user_service;
pub mod order_service;
pub mod email_service;
pub mod file_service;

// Service trait for common operations
use crate::{Result, Entity};
use uuid::Uuid;

#[async_trait::async_trait]
pub trait CrudService<T: Entity> {
    async fn create(&self, entity: T) -> Result<T>;
    async fn get_by_id(&self, id: &T::Id) -> Result<Option<T>>;
    async fn update(&self, entity: T) -> Result<T>;
    async fn delete(&self, id: &T::Id) -> Result<()>;
    async fn list(&self, limit: u32, offset: u32) -> Result<Vec<T>>;
}

/// Health check service
pub struct HealthService;

impl HealthService {
    pub fn new() -> Self {
        Self
    }

    pub async fn check_health(&self) -> Result<crate::HealthCheck> {
        // In a real implementation, this would check:
        // - Database connectivity
        // - External service availability
        // - System resources
        Ok(crate::HealthCheck::new())
    }
}