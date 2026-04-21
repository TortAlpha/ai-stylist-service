use serde::Deserialize;

use super::super::brand::BrandTier;

#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct CreateBrandRequest {
    pub name: String,
    pub code: String,
    pub tier: BrandTier,
    pub country: Option<String>,
}

#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct UpdateBrandRequest {
    pub name: Option<String>,
    pub code: Option<String>,
    pub tier: Option<BrandTier>,
    pub country: Option<String>,
}
