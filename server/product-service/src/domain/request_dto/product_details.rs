use serde::Deserialize;
use rust_decimal::Decimal;

use super::super::product_details::ProductCondition;
use super::super::clothing_details::ClothingFit;
use super::super::bags_details::HandleType;
use super::super::footwear_details::SizeSystem;

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

/// Type-specific details within create request
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CreateTypeDetailsRequest {
    Clothing {
        size: Option<String>,
        fit: Option<ClothingFit>,
    },
    Footwear {
        shoe_size: Option<String>,
        size_system: Option<SizeSystem>,
        insole_length_cm: Option<Decimal>,
    },
    Bags {
        width_cm: Option<Decimal>,
        height_cm: Option<Decimal>,
        depth_cm: Option<Decimal>,
        handle_type: Option<HandleType>,
    },
    Jewelry {
        metal: Option<String>,
        stone: Option<String>,
        clasp_type: Option<String>,
    },
    Accessories,
}