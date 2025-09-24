//! Data models for the Terra marketplace

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Product model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub price: f64,
    pub category: String,
    pub seller_id: Uuid,
    pub images: Vec<String>,
    pub stock_quantity: u32,
    pub rating: f32,
    pub review_count: u32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub is_active: bool,
    pub tags: Vec<String>,
}

/// Product creation request
#[derive(Debug, Deserialize, Validate)]
pub struct CreateProductRequest {
    #[validate(length(min = 1, max = 200))]
    pub title: String,
    #[validate(length(min = 1, max = 2000))]
    pub description: String,
    #[validate(range(min = 0.01, max = 100000.0))]
    pub price: f64,
    #[validate(length(min = 1, max = 50))]
    pub category: String,
    pub images: Vec<String>,
    #[validate(range(min = 0, max = 10000))]
    pub stock_quantity: u32,
    pub tags: Vec<String>,
}

/// User model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub first_name: String,
    pub last_name: String,
    pub avatar_url: Option<String>,
    pub is_seller: bool,
    pub is_verified: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub last_login: Option<chrono::DateTime<chrono::Utc>>,
}

/// User registration request
#[derive(Debug, Deserialize, Validate)]
pub struct RegisterUserRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 3, max = 30))]
    pub username: String,
    #[validate(length(min = 8, max = 128))]
    pub password: String,
    #[validate(length(min = 1, max = 50))]
    pub first_name: String,
    #[validate(length(min = 1, max = 50))]
    pub last_name: String,
}

/// User login request
#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8, max = 128))]
    pub password: String,
}

/// Order model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: Uuid,
    pub user_id: Uuid,
    pub items: Vec<OrderItem>,
    pub total_amount: f64,
    pub shipping_address: Address,
    pub billing_address: Address,
    pub status: OrderStatus,
    pub payment_method: PaymentMethod,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub shipped_at: Option<chrono::DateTime<chrono::Utc>>,
    pub delivered_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Order item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderItem {
    pub product_id: Uuid,
    pub product_title: String,
    pub quantity: u32,
    pub unit_price: f64,
    pub total_price: f64,
}

/// Order status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderStatus {
    Pending,
    Processing,
    Shipped,
    Delivered,
    Cancelled,
    Refunded,
}

/// Payment method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PaymentMethod {
    CreditCard,
    DebitCard,
    PayPal,
    ApplePay,
    GooglePay,
}

/// Address model
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Address {
    #[validate(length(min = 1, max = 100))]
    pub street: String,
    #[validate(length(min = 1, max = 50))]
    pub city: String,
    #[validate(length(min = 1, max = 50))]
    pub state: String,
    #[validate(length(min = 5, max = 10))]
    pub zip_code: String,
    #[validate(length(min = 2, max = 2))]
    pub country: String,
}

/// Cart model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cart {
    pub user_id: Uuid,
    pub items: Vec<CartItem>,
    pub total_amount: f64,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Cart item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CartItem {
    pub product_id: Uuid,
    pub quantity: u32,
    pub added_at: chrono::DateTime<chrono::Utc>,
}

/// Review model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Review {
    pub id: Uuid,
    pub product_id: Uuid,
    pub user_id: Uuid,
    pub rating: u8, // 1-5 stars
    pub title: String,
    pub comment: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub helpful_votes: u32,
}

/// Category model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: String,
    pub name: String,
    pub description: String,
    pub parent_id: Option<String>,
    pub icon: String,
    pub is_active: bool,
}

/// Search result
#[derive(Debug, Serialize)]
pub struct SearchResult<T> {
    pub items: Vec<T>,
    pub total_count: u64,
    pub page: u32,
    pub limit: u32,
    pub has_more: bool,
}

impl<T> SearchResult<T> {
    pub fn new(items: Vec<T>, total_count: u64, page: u32, limit: u32) -> Self {
        let has_more = (page * limit) < total_count as u32;
        Self {
            items,
            total_count,
            page,
            limit,
            has_more,
        }
    }
}

/// Database entity trait
pub trait Entity {
    type Id;
    
    fn id(&self) -> &Self::Id;
    fn created_at(&self) -> chrono::DateTime<chrono::Utc>;
    fn updated_at(&self) -> chrono::DateTime<chrono::Utc>;
}

impl Entity for Product {
    type Id = Uuid;
    
    fn id(&self) -> &Self::Id {
        &self.id
    }
    
    fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.created_at
    }
    
    fn updated_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.updated_at
    }
}

impl Entity for User {
    type Id = Uuid;
    
    fn id(&self) -> &Self::Id {
        &self.id
    }
    
    fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.created_at
    }
    
    fn updated_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.updated_at
    }
}

impl Entity for Order {
    type Id = Uuid;
    
    fn id(&self) -> &Self::Id {
        &self.id
    }
    
    fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.created_at
    }
    
    fn updated_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.updated_at
    }
}