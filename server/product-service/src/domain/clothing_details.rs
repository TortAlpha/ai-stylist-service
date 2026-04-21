use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, utoipa::ToSchema)]
#[sqlx(type_name = "VARCHAR", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ClothingFit {
    Regular,
    Slim,
    Oversized,
    Relaxed,
}

impl ClothingFit {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Regular => "regular",
            Self::Slim => "slim",
            Self::Oversized => "oversized",
            Self::Relaxed => "relaxed",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ClothingDetails {
    pub product_id: Uuid,
    pub fit: Option<ClothingFit>,
}
