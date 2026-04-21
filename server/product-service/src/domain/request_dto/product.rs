use rust_decimal::Decimal;
use serde::Deserialize;

use super::super::product_details::ProductStatus;
use super::product_details::{CreateProductDetailsRequest, UpdateProductDetailsRequest};

#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct CreateProductRequest {
    pub name: String,
    pub brand_id: i32,
    pub category_id: i32,
    pub status: Option<ProductStatus>, // defaults to "intake"
    pub purchase_price: Option<Decimal>,
    pub purchase_location_id: Option<i32>,
    pub currency: Option<String>, // defaults to "RSD"
    pub ai_notes: Option<String>,
    pub details: CreateProductDetailsRequest,
    #[serde(default)]
    pub style_tag_ids: Vec<i32>,
    #[serde(default)]
    pub vibe_tag_ids: Vec<i32>,
    #[serde(default)]
    pub season_ids: Vec<i32>,
}

#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct UpdateProductRequest {
    pub name: Option<String>,
    pub brand_id: Option<i32>,
    pub category_id: Option<i32>,
    pub status: Option<ProductStatus>,
    pub purchase_price: Option<Decimal>,
    pub purchase_location_id: Option<i32>,
    pub currency: Option<String>,
    pub ai_notes: Option<String>,
    pub details: Option<UpdateProductDetailsRequest>,
    pub style_tag_ids: Option<Vec<i32>>,
    pub vibe_tag_ids: Option<Vec<i32>>,
    pub season_ids: Option<Vec<i32>>,
    pub expected_version: i32,
}
