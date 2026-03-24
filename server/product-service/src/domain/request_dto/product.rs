use rust_decimal::Decimal;
use serde::Deserialize;

use super::product_details::{CreateProductDetailsRequest, CreateTypeDetailsRequest};

#[derive(Debug, Clone, Deserialize)]
pub struct CreateProductRequest {
    pub name: String,
    pub brand_id: i32,
    pub category_id: i32,
    pub product_type: String,
    pub status: Option<String>,                // defaults to "intake"
    pub purchase_price: Option<Decimal>,
    pub purchase_location: Option<String>,
    pub currency: Option<String>,              // defaults to "RSD"
    pub ai_notes: Option<String>,
    pub images_path: Option<String>,
    pub preview_image_key: Option<String>,
    pub product_url: Option<String>,

    // Common details
    pub details: CreateProductDetailsRequest,

    // Type-specific details
    pub type_details: Option<CreateTypeDetailsRequest>,

    // Tag IDs
    pub style_tag_ids: Vec<i32>,
    pub vibe_tag_ids: Vec<i32>,
    pub season_ids: Vec<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateProductRequest {
    pub name: Option<String>,
    pub brand_id: Option<i32>,
    pub category_id: Option<i32>,
    pub product_type: Option<String>,
    pub status: Option<String>,
    pub purchase_price: Option<Decimal>,
    pub purchase_location: Option<String>,
    pub currency: Option<String>,
    pub ai_notes: Option<String>,
    pub images_path: Option<String>,
    pub preview_image_key: Option<String>,
    pub product_url: Option<String>,

    // Optimistic locking — must match current version
    pub expected_version: i32,
}