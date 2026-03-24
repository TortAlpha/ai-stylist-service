use serde::Serialize;
use uuid::Uuid;
use rust_decimal::Decimal;
use chrono::{DateTime, Utc};

use super::super::product::ProductFull;

use super::brand::BrandShortResponse;
use super::category::CategoryResponse;
use super::tag::ProductTagsResponse;
use super::product_details::TypeDetailsResponse;
use super::product_details::ProductAttributesResponse;

#[derive(Debug, Serialize)]
pub struct ProductPreviewResponse {
    pub id: Uuid,
    pub sku: String,
    pub name: String,
    pub purchase_price: Option<Decimal>,
    pub currency: String,
    pub preview_url: Option<String>,
    pub image_urls: Vec<String>,
    pub product_url: Option<String>,
    pub brand: BrandShortResponse,
    pub product_type: String,
    pub category: String,
    pub condition: String,
    pub color: Option<String>,
    /// Human-readable size label (e.g. "43 EU", "M", "30x20x10 cm")
    pub size_label: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ProductResponse {
    pub id: Uuid,
    pub sku: String,
    pub name: String,
    pub purchase_price: Option<Decimal>,
    pub purchase_location: Option<String>,
    pub currency: String,
    pub ai_notes: Option<String>,
    pub preview_url: Option<String>,
    pub image_urls: Vec<String>,
    pub product_url: Option<String>,
    pub brand: BrandShortResponse,
    pub brand_id: i32,
    pub product_type: String,
    pub category: CategoryResponse,
    pub category_id: i32,
    pub status: String,
    pub version: i32,
    pub details: ProductAttributesResponse,
    pub type_details: TypeDetailsResponse,
    pub tags: ProductTagsResponse,
    pub created_at: DateTime<Utc>,
}

/// Wrapper for list endpoint — ProductFull + resolved preview URL
#[derive(Debug, Serialize)]
pub struct ProductListItem {
    #[serde(flatten)]
    pub product: ProductFull,
    pub preview_url: Option<String>,
}
