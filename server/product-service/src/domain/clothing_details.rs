use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "VARCHAR", rename_all = "lowercase")]
pub enum ClothingFit {
    Regular,
    Slim,
    Oversized,
    Relaxed,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ClothingDetails {
    pub product_id: Uuid,
    pub size: Option<String>,
    pub fit: Option<ClothingFit>,
}