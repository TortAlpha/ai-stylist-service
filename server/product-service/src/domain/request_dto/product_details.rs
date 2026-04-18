use rust_decimal::Decimal;
use serde::Deserialize;

use super::super::product_details::{ProductCondition, TypeDetailsInput};
use super::super::size_info::SizeSystem;

#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
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
    pub size: SizeInput,
    pub type_details: TypeDetailsInput,
}

#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
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
    pub size: Option<SizeInput>,
    pub type_details: Option<TypeDetailsInput>,
}

/// Unified size input — validated against category.size_group at the service layer
#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct SizeInput {
    pub size_value: Option<String>,
    pub size_value2: Option<String>,
    pub size_system: Option<SizeSystem>,
    pub measurement_cm: Option<Decimal>,
}
