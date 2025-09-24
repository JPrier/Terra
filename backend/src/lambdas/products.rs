//! AWS Lambda function for product-related operations

use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde_json::{json, Value};
use terra_backend::{
    init_tracing, ApiResponse, Config, Product, CreateProductRequest, 
    PaginationParams, FilterParams, Result, ApiError
};
use uuid::Uuid;

/// Lambda handler for product operations
async fn function_handler(event: LambdaEvent<Value>) -> Result<Value> {
    init_tracing();
    
    let (event, _context) = event.into_parts();
    
    // Parse HTTP method and path from API Gateway event
    let http_method = event["httpMethod"].as_str().unwrap_or("GET");
    let path = event["path"].as_str().unwrap_or("/");
    let query_params = event["queryStringParameters"].clone();
    let body = event["body"].as_str().unwrap_or("{}");

    tracing::info!("Processing {} {} request", http_method, path);

    let response = match (http_method, path) {
        ("GET", "/products") => handle_list_products(query_params).await,
        ("GET", path) if path.starts_with("/products/") => {
            let id = path.trim_start_matches("/products/");
            handle_get_product(id).await
        },
        ("POST", "/products") => handle_create_product(body).await,
        ("PUT", path) if path.starts_with("/products/") => {
            let id = path.trim_start_matches("/products/");
            handle_update_product(id, body).await
        },
        ("DELETE", path) if path.starts_with("/products/") => {
            let id = path.trim_start_matches("/products/");
            handle_delete_product(id).await
        },
        _ => Err(ApiError::NotFound("Endpoint not found".to_string())),
    };

    match response {
        Ok(data) => Ok(json!({
            "statusCode": 200,
            "headers": {
                "Content-Type": "application/json",
                "Access-Control-Allow-Origin": "*",
                "Access-Control-Allow-Methods": "GET, POST, PUT, DELETE, OPTIONS",
                "Access-Control-Allow-Headers": "Content-Type, Authorization"
            },
            "body": serde_json::to_string(&data)?
        })),
        Err(err) => {
            let status_code = err.status_code().as_u16();
            Ok(json!({
                "statusCode": status_code,
                "headers": {
                    "Content-Type": "application/json",
                    "Access-Control-Allow-Origin": "*"
                },
                "body": serde_json::to_string(&ApiResponse::<()>::error(err.to_string()))?
            }))
        }
    }
}

/// Handle listing products with filters and pagination
async fn handle_list_products(query_params: Value) -> Result<ApiResponse<Vec<Product>>> {
    // Parse query parameters
    let pagination = if query_params.is_null() {
        PaginationParams::default()
    } else {
        PaginationParams {
            page: query_params["page"].as_str().and_then(|s| s.parse().ok()),
            limit: query_params["limit"].as_str().and_then(|s| s.parse().ok()),
        }
    };

    let filters = if query_params.is_null() {
        FilterParams {
            category: None,
            min_price: None,
            max_price: None,
            search: None,
            sort_by: None,
            sort_order: None,
        }
    } else {
        FilterParams {
            category: query_params["category"].as_str().map(|s| s.to_string()),
            min_price: query_params["min_price"].as_str().and_then(|s| s.parse().ok()),
            max_price: query_params["max_price"].as_str().and_then(|s| s.parse().ok()),
            search: query_params["search"].as_str().map(|s| s.to_string()),
            sort_by: query_params["sort_by"].as_str().map(|s| s.to_string()),
            sort_order: query_params["sort_order"].as_str().map(|s| s.to_string()),
        }
    };

    // For demo purposes, return sample data
    // In a real implementation, this would query DynamoDB
    let sample_products = get_sample_products();
    let mut filtered_products = sample_products;

    // Apply filters
    if let Some(category) = &filters.category {
        filtered_products.retain(|p| p.category.to_lowercase() == category.to_lowercase());
    }

    if let Some(min_price) = filters.min_price {
        filtered_products.retain(|p| p.price >= min_price);
    }

    if let Some(max_price) = filters.max_price {
        filtered_products.retain(|p| p.price <= max_price);
    }

    if let Some(search) = &filters.search {
        let search_lower = search.to_lowercase();
        filtered_products.retain(|p| {
            p.title.to_lowercase().contains(&search_lower) ||
            p.description.to_lowercase().contains(&search_lower) ||
            p.tags.iter().any(|tag| tag.to_lowercase().contains(&search_lower))
        });
    }

    // Apply sorting
    match filters.sort_by.as_deref() {
        Some("price") => {
            if filters.sort_order.as_deref() == Some("desc") {
                filtered_products.sort_by(|a, b| b.price.partial_cmp(&a.price).unwrap());
            } else {
                filtered_products.sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap());
            }
        },
        Some("rating") => {
            filtered_products.sort_by(|a, b| b.rating.partial_cmp(&a.rating).unwrap());
        },
        Some("created_at") => {
            if filters.sort_order.as_deref() == Some("desc") {
                filtered_products.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            } else {
                filtered_products.sort_by(|a, b| a.created_at.cmp(&b.created_at));
            }
        },
        _ => {} // Keep original order
    }

    // Apply pagination
    let page = pagination.page.unwrap_or(1);
    let limit = pagination.limit.unwrap_or(20);
    let start = ((page - 1) * limit) as usize;
    let end = (start + limit as usize).min(filtered_products.len());

    let paginated_products = if start < filtered_products.len() {
        filtered_products[start..end].to_vec()
    } else {
        vec![]
    };

    Ok(ApiResponse::success(paginated_products))
}

/// Handle getting a single product by ID
async fn handle_get_product(id: &str) -> Result<ApiResponse<Product>> {
    let product_id = Uuid::parse_str(id)
        .map_err(|_| ApiError::ValidationError("Invalid product ID format".to_string()))?;

    // For demo purposes, return sample data
    // In a real implementation, this would query DynamoDB
    let sample_products = get_sample_products();
    let product = sample_products
        .into_iter()
        .find(|p| p.id == product_id)
        .ok_or_else(|| ApiError::product_not_found(product_id))?;

    Ok(ApiResponse::success(product))
}

/// Handle creating a new product
async fn handle_create_product(body: &str) -> Result<ApiResponse<Product>> {
    let request: CreateProductRequest = serde_json::from_str(body)
        .map_err(|_| ApiError::ValidationError("Invalid request body".to_string()))?;

    // Validate the request
    validator::Validate::validate(&request)?;

    // Create new product
    let now = chrono::Utc::now();
    let product = Product {
        id: Uuid::new_v4(),
        title: request.title,
        description: request.description,
        price: request.price,
        category: request.category,
        seller_id: Uuid::new_v4(), // In real app, get from JWT token
        images: request.images,
        stock_quantity: request.stock_quantity,
        rating: 0.0,
        review_count: 0,
        created_at: now,
        updated_at: now,
        is_active: true,
        tags: request.tags,
    };

    // In a real implementation, this would save to DynamoDB
    tracing::info!("Created product: {}", product.id);

    Ok(ApiResponse::success(product))
}

/// Handle updating an existing product
async fn handle_update_product(id: &str, body: &str) -> Result<ApiResponse<Product>> {
    let product_id = Uuid::parse_str(id)
        .map_err(|_| ApiError::ValidationError("Invalid product ID format".to_string()))?;

    let request: CreateProductRequest = serde_json::from_str(body)
        .map_err(|_| ApiError::ValidationError("Invalid request body".to_string()))?;

    // Validate the request
    validator::Validate::validate(&request)?;

    // In a real implementation, this would update the product in DynamoDB
    // For demo purposes, create a new product with the updated data
    let now = chrono::Utc::now();
    let product = Product {
        id: product_id,
        title: request.title,
        description: request.description,
        price: request.price,
        category: request.category,
        seller_id: Uuid::new_v4(), // In real app, get from JWT token
        images: request.images,
        stock_quantity: request.stock_quantity,
        rating: 4.2, // Keep existing rating
        review_count: 15, // Keep existing review count
        created_at: now.checked_sub_signed(chrono::Duration::days(30)).unwrap_or(now),
        updated_at: now,
        is_active: true,
        tags: request.tags,
    };

    tracing::info!("Updated product: {}", product.id);

    Ok(ApiResponse::success(product))
}

/// Handle deleting a product
async fn handle_delete_product(id: &str) -> Result<ApiResponse<()>> {
    let product_id = Uuid::parse_str(id)
        .map_err(|_| ApiError::ValidationError("Invalid product ID format".to_string()))?;

    // In a real implementation, this would delete the product from DynamoDB
    tracing::info!("Deleted product: {}", product_id);

    Ok(ApiResponse::success(()))
}

/// Generate sample products for demonstration
fn get_sample_products() -> Vec<Product> {
    let now = chrono::Utc::now();
    
    vec![
        Product {
            id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap(),
            title: "Wireless Bluetooth Headphones".to_string(),
            description: "High-quality wireless headphones with noise cancellation".to_string(),
            price: 79.99,
            category: "electronics".to_string(),
            seller_id: Uuid::new_v4(),
            images: vec!["https://example.com/headphones.jpg".to_string()],
            stock_quantity: 50,
            rating: 4.5,
            review_count: 234,
            created_at: now.checked_sub_signed(chrono::Duration::days(10)).unwrap_or(now),
            updated_at: now,
            is_active: true,
            tags: vec!["audio".to_string(), "wireless".to_string(), "bluetooth".to_string()],
        },
        Product {
            id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440002").unwrap(),
            title: "Classic Cotton T-Shirt".to_string(),
            description: "Comfortable 100% cotton t-shirt in various colors".to_string(),
            price: 24.99,
            category: "fashion".to_string(),
            seller_id: Uuid::new_v4(),
            images: vec!["https://example.com/tshirt.jpg".to_string()],
            stock_quantity: 100,
            rating: 4.2,
            review_count: 156,
            created_at: now.checked_sub_signed(chrono::Duration::days(5)).unwrap_or(now),
            updated_at: now,
            is_active: true,
            tags: vec!["clothing".to_string(), "cotton".to_string(), "casual".to_string()],
        },
    ]
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(service_fn(function_handler)).await
}