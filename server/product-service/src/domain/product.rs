use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ============================================================
// PRODUCT STATUS
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProductStatus {
    pub id: i32,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub sort_order: i32,
}

// ============================================================
// PRODUCT TYPE
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProductType {
    pub id: i32,
    pub code: String,
    pub name: String,
}

// ============================================================
// PRODUCT
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Product {
    pub id: Uuid,
    pub sku: String,
    pub name: String,
    pub brand_id: i32,
    pub category_id: i32,
    pub type_id: i32,
    pub status_id: i32,
    pub original_price: Decimal,
    pub discount: Decimal,
    pub final_price: Decimal,
    pub currency: String,
    pub in_stock: bool,
    pub quantity: i32,
    pub images_path: Option<String>,
    pub preview_image_url: Option<String>,
    pub product_url: Option<String>,
    pub version: i32,
    pub reserved_until: Option<DateTime<Utc>>,
    pub reserved_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================
// SIMILAR PRODUCTS
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SimilarProduct {
    pub product_id: Uuid,
    pub similar_product_id: Uuid,
    pub similarity_score: f64,
}

// ============================================================
// PRODUCT HISTORY (audit log)
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProductHistory {
    pub id: i64,
    pub product_id: Uuid,
    pub version: i32,
    pub name: Option<String>,
    pub brand_id: Option<i32>,
    pub category_id: Option<i32>,
    pub type_id: Option<i32>,
    pub status_id: Option<i32>,
    pub original_price: Option<Decimal>,
    pub discount: Option<Decimal>,
    pub final_price: Option<Decimal>,
    pub currency: Option<String>,
    pub in_stock: Option<bool>,
    pub quantity: Option<i32>,
    pub images_path: Option<String>,
    pub preview_image_url: Option<String>,
    pub product_url: Option<String>,
    pub reserved_until: Option<DateTime<Utc>>,
    pub reserved_by: Option<String>,
    pub changed_by: Option<String>,
    pub changed_at: DateTime<Utc>,
}

// ============================================================
// VIEW: FULL PRODUCT CARD
// Maps to v_product_full / v_product_storefront views
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProductFull {
    pub id: Uuid,
    pub sku: String,
    pub name: String,
    pub original_price: Decimal,
    pub discount: Decimal,
    pub final_price: Decimal,
    pub currency: String,
    pub in_stock: bool,
    pub quantity: i32,
    pub preview_image_url: Option<String>,
    pub product_url: Option<String>,
    pub version: i32,

    // Status
    pub status_code: String,
    pub status_name: String,

    // Product type
    pub type_code: String,
    pub type_name: String,

    // Reservation
    pub reserved_until: Option<DateTime<Utc>>,
    pub reserved_by: Option<String>,

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

    // Aggregated tags
    pub style_tags: Vec<String>,
    pub vibe_tags: Vec<String>,
    pub season_tags: Vec<String>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
