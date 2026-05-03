use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::brand::BrandShortResponse;
use super::category::CategoryResponse;
use super::product_details::ProductAttributesResponse;
use super::product_details::SizeResponse;
use super::product_details::TypeDetailsResponse;
use super::tag::ProductTagsResponse;
use crate::domain::utils::mappers::{ImageVariantUrls, ProductImageUrls};

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ProductPreviewResponse {
    pub id: Uuid,
    pub sku: String,
    pub name: String,
    pub purchase_price: Option<Decimal>,
    pub currency: String,
    pub preview_url: Option<ImageVariantUrls>,
    pub brand_name: String,
    pub product_type: String,
    pub category: String,
    pub color: Option<String>,
    pub size: SizeResponse,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct AdminProductPreviewResponse {
    pub id: Uuid,
    pub sku: String,
    pub name: String,
    pub purchase_price: Option<Decimal>,
    pub currency: String,
    pub preview_url: Option<ImageVariantUrls>,
    pub brand_name: String,
    pub product_type: String,
    pub category: String,
    pub status: String,
    pub condition: Option<String>,
    pub color: Option<String>,
    pub size: SizeResponse,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct AdminProductDTO {
    pub id: Uuid,
    pub sku: String,
    pub name: String,
    pub purchase_price: Option<Decimal>,
    pub purchase_location: Option<String>,
    pub currency: String,
    pub ai_notes: Option<String>,
    pub preview_url: Option<ImageVariantUrls>,
    pub image_urls: Vec<ProductImageUrls>,
    pub brand: BrandShortResponse,
    pub brand_id: i32,
    pub product_type: String,
    pub category: CategoryResponse,
    pub category_id: i32,
    pub status: String,
    pub version: i32,
    pub details: ProductAttributesResponse,
    pub size: SizeResponse,
    pub type_details: TypeDetailsResponse,
    pub tags: ProductTagsResponse,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct MarketplaceProductDTO {
    //TODO
}

/// Aggregated filter options derived from current product data.
/// Fields are populated based on which products match the current filter scope.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ProductFilterOptions {
    pub colors: Vec<String>,
    pub conditions: Vec<String>,
    pub materials: Vec<String>,
    pub price_min: Option<Decimal>,
    pub price_max: Option<Decimal>,
    pub brand_ids: Vec<i32>,
    pub category_ids: Vec<i32>,
}
