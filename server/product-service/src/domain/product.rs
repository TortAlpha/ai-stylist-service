use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use super::product_details::ProductStatus;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Product {
    pub id: Uuid,
    pub sku: String,
    pub name: String,
    pub brand_id: i32,
    pub category_id: i32,
    pub status: ProductStatus,
    pub purchase_price: Option<Decimal>,
    pub purchase_location: Option<String>,
    pub currency: String,
    pub ai_notes: Option<String>,
    pub images_path: Option<String>,
    pub image_count: i32,
    pub preview_image_key: Option<String>,
    pub product_url: Option<String>,
    pub version: i32,
    pub is_deleted: bool,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProductFull {
    pub id: Uuid,
    pub sku: String,
    pub name: String,
    pub purchase_price: Option<Decimal>,
    pub purchase_location: Option<String>,
    pub currency: String,
    pub ai_notes: Option<String>,
    pub category_id: i32,
    pub image_count: i32,
    pub preview_image_key: Option<String>,
    pub product_url: Option<String>,
    pub version: i32,

    // Status & type (string enums)
    pub status: String,
    #[sqlx(rename = "type")]
    pub product_type: String,

    // Brand
    pub brand_id: i32,
    pub brand_name: String,
    pub brand_tier: String,

    // Category
    pub category_name: String,
    pub parent_category: Option<String>,
    pub gender: String,

    // Details (common)
    pub material: Option<String>,
    pub condition: Option<String>,
    pub color: Option<String>,
    pub year_of_release: Option<i32>,
    pub is_vintage: Option<bool>,
    pub is_collab: Option<bool>,
    pub collab_name: Option<String>,
    pub is_limited_edition: Option<bool>,
    pub special_notes: Option<String>,

    // Type-specific fields (flattened from product_details)
    pub clothing_size: Option<String>,
    pub clothing_fit: Option<String>,
    pub shoe_size: Option<String>,
    pub size_system: Option<String>,
    pub insole_length_cm: Option<Decimal>,
    pub bag_width_cm: Option<Decimal>,
    pub bag_height_cm: Option<Decimal>,
    pub bag_depth_cm: Option<Decimal>,
    pub bag_handle_type: Option<String>,
    pub jewelry_metal: Option<String>,
    pub jewelry_stone: Option<String>,
    pub jewelry_clasp_type: Option<String>,

    // Aggregated tags
    pub style_tags: Vec<String>,
    pub vibe_tags: Vec<String>,
    pub season_tags: Vec<String>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SimilarProduct {
    pub product_id: Uuid,
    pub similar_product_id: Uuid,
    pub similarity_score: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProductHistory {
    pub id: i64,
    pub product_id: Uuid,
    pub version: i32,
    pub changes: serde_json::Value,
    pub changed_by: Option<String>,
    pub changed_at: DateTime<Utc>,
}