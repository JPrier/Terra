//! Database abstraction layer for DynamoDB

use aws_sdk_dynamodb::{Client, Config};
use crate::{Result, ApiError};

/// Database client wrapper
pub struct Database {
    client: Client,
    table_prefix: String,
}

impl Database {
    /// Create a new database client
    pub async fn new(region: &str, table_prefix: String) -> Result<Self> {
        let config = aws_config::from_env()
            .region(region)
            .load()
            .await;
        
        let client = Client::new(&config);
        
        Ok(Self {
            client,
            table_prefix,
        })
    }

    /// Get table name with prefix
    pub fn table_name(&self, table: &str) -> String {
        format!("{}_{}", self.table_prefix, table)
    }

    /// Get DynamoDB client
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Check if table exists
    pub async fn table_exists(&self, table_name: &str) -> Result<bool> {
        match self.client.describe_table()
            .table_name(table_name)
            .send()
            .await 
        {
            Ok(_) => Ok(true),
            Err(e) => {
                if e.to_string().contains("ResourceNotFoundException") {
                    Ok(false)
                } else {
                    Err(ApiError::DatabaseError(format!("Failed to check table existence: {}", e)))
                }
            }
        }
    }
}

/// DynamoDB item conversion trait
pub trait DynamoItem {
    /// Convert to DynamoDB item
    fn to_item(&self) -> Result<std::collections::HashMap<String, aws_sdk_dynamodb::types::AttributeValue>>;
    
    /// Convert from DynamoDB item
    fn from_item(item: std::collections::HashMap<String, aws_sdk_dynamodb::types::AttributeValue>) -> Result<Self>
    where
        Self: Sized;
}