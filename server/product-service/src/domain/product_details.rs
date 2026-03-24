use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProductDetails {
    pub product_id: Uuid,
    pub material: Option<String>,
    pub condition: ProductCondition,
    pub color: Option<String>,
    pub year_of_release: Option<i32>,
    pub is_vintage: bool,
    pub is_collab: bool,
    pub collab_name: Option<String>,
    pub is_limited_edition: bool,
    pub special_notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "VARCHAR", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ProductCondition {
    NewWithTags,
    Excellent,
    Good,
    Fair,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "VARCHAR", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ProductStatus {
    Intake,
    Inspection,
    Rejected,
    Preparation,
    PhotoQueue,
    PhotoDone,
    Ready,
    Reserved,
    Sold,
    Returned,
}