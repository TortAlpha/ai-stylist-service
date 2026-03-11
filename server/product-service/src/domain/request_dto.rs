use rust_decimal::Decimal;
use serde::Deserialize;

use super::product_details::ProductCondition;

/// Request body for creating a new product
#[derive(Debug, Clone, Deserialize)]
pub struct CreateProductRequest {
    pub name: String,
    pub brand_id: i32,
    pub category_id: i32,
    pub type_id: i32,
    pub status_id: i32,
    pub original_price: Decimal,
    pub discount: Option<Decimal>,             // defaults to 0
    pub currency: Option<String>,              // defaults to "EUR"
    pub quantity: Option<i32>,                 // defaults to 1
    pub images_path: Option<String>,
    pub preview_image_url: Option<String>,
    pub product_url: Option<String>,

    // Common details
    pub details: CreateProductDetailsRequest,

    // Tag IDs
    pub style_tag_ids: Vec<i32>,
    pub vibe_tag_ids: Vec<i32>,
    pub season_ids: Vec<i32>,
}

/// Common product details within create request
#[derive(Debug, Clone, Deserialize)]
pub struct CreateProductDetailsRequest {
    pub condition: ProductCondition,
    pub material: Option<String>,
    pub color: Option<String>,
    pub year_of_release: Option<i32>,
    pub is_vintage: Option<bool>,
    pub is_collab: Option<bool>,
    pub collab_name: Option<String>,
    pub is_limited_edition: Option<bool>,
    pub special_notes: Option<String>,
}

/// Request body for updating a product
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateProductRequest {
    pub name: Option<String>,
    pub brand_id: Option<i32>,
    pub category_id: Option<i32>,
    pub type_id: Option<i32>,
    pub status_id: Option<i32>,
    pub original_price: Option<Decimal>,
    pub discount: Option<Decimal>,
    pub currency: Option<String>,
    pub quantity: Option<i32>,
    pub images_path: Option<String>,
    pub preview_image_url: Option<String>,
    pub product_url: Option<String>,

    // Optimistic locking — must match current version
    pub expected_version: i32,
}

/// Request body for updating common product details
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateProductDetailsRequest {
    pub condition: Option<ProductCondition>,
    pub material: Option<String>,
    pub color: Option<String>,
    pub year_of_release: Option<i32>,
    pub is_vintage: Option<bool>,
    pub is_collab: Option<bool>,
    pub collab_name: Option<String>,
    pub is_limited_edition: Option<bool>,
    pub special_notes: Option<String>,
}

/// Request body for reserving a product
#[derive(Debug, Clone, Deserialize)]
pub struct ReserveProductRequest {
    pub reserved_by: String,
    pub duration_minutes: Option<i64>,         // defaults to 30 min
}

/// Request body for purchasing a product
#[derive(Debug, Clone, Deserialize)]
pub struct PurchaseProductRequest {
    pub quantity: Option<i32>,                 // defaults to 1
}
