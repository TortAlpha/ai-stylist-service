use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Brand {
    pub id: i32,
    pub name: String,
    pub code: String,
    pub tier: BrandTier,
    pub country: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, utoipa::ToSchema)]
#[sqlx(type_name = "VARCHAR", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum BrandTier {
    Mass,
    Premium,
    Luxury,
}

impl BrandTier {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Mass => "mass",
            Self::Premium => "premium",
            Self::Luxury => "luxury",
        }
    }
}
