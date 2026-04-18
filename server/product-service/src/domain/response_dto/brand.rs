use chrono::{DateTime, Utc};
use serde::Serialize;

use super::super::brand::Brand;

/// Full brand response (admin endpoints)
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct BrandResponse {
    pub id: i32,
    pub name: String,
    pub code: String,
    pub tier: String,
    pub country: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Short brand info embedded in product responses
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct BrandShortResponse {
    pub id: i32,
    pub name: String,
    pub tier: String,
}

impl From<Brand> for BrandResponse {
    fn from(b: Brand) -> Self {
        Self {
            id: b.id,
            name: b.name,
            code: b.code,
            tier: b.tier.as_str().to_string(),
            country: b.country,
            created_at: b.created_at,
        }
    }
}
