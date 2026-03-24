use rust_decimal::Decimal;
use serde::{Deserialize};

/// Query parameters for listing products
#[derive(Debug, Clone, Deserialize)]
pub struct ProductListQuery {
    pub page: Option<i64>,                 // defaults to 1
    pub per_page: Option<i64>,            // defaults to 20
    pub brand_id: Option<i32>,
    pub category_id: Option<i32>,
    pub product_type: Option<String>,
    pub status: Option<String>,
    pub gender: Option<String>,
    pub price_min: Option<Decimal>,
    pub price_max: Option<Decimal>,
    pub condition: Option<String>,
    pub clothing_size: Option<String>,
    pub shoe_size: Option<String>,
    pub shoe_size_system: Option<String>,
    pub sort_by: Option<ProductSortField>,
    pub sort_order: Option<SortOrder>,
}

/// Query parameters for available sizes endpoint
#[derive(Debug, Clone, Deserialize)]
pub struct SizesQuery {
    pub product_type: String,
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