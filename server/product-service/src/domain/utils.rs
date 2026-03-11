use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Standard API response envelope
#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn err(message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.into()),
        }
    }
}

/// Paginated list response
#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,
}

/// Query parameters for listing products
#[derive(Debug, Clone, Deserialize)]
pub struct ProductListQuery {
    pub page: Option<i64>,                 // defaults to 1
    pub per_page: Option<i64>,            // defaults to 20
    pub brand_id: Option<i32>,
    pub category_id: Option<i32>,
    pub type_id: Option<i32>,
    pub status_code: Option<String>,
    pub gender: Option<String>,
    pub price_min: Option<Decimal>,
    pub price_max: Option<Decimal>,
    pub condition: Option<String>,
    pub in_stock: Option<bool>,
    pub sort_by: Option<ProductSortField>,
    pub sort_order: Option<SortOrder>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductSortField {
    Price,
    CreatedAt,
    UpdatedAt,
    Name,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    Asc,
    Desc,
}
