use serde::Deserialize;

use super::super::category::{Gender, ProductType, SizeGroup};

#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct CreateCategoryRequest {
    pub name: String,
    pub code: String,
    pub parent_id: Option<i32>,
    pub gender: Gender,
    pub product_type: ProductType,
    pub size_group: SizeGroup,
}

#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct UpdateCategoryRequest {
    pub name: Option<String>,
    pub code: Option<String>,
    pub parent_id: Option<i32>,
    pub gender: Option<Gender>,
    pub product_type: Option<ProductType>,
    pub size_group: Option<SizeGroup>,
}
