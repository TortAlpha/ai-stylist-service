use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct StyleTag {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct VibeTag {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Season {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProductStyleTag {
    pub product_id: Uuid,
    pub style_tag_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProductVibeTag {
    pub product_id: Uuid,
    pub vibe_tag_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProductSeason {
    pub product_id: Uuid,
    pub season_id: i32,
}
