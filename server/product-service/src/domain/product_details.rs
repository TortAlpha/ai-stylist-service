use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ============================================================
// COMMON DETAILS
// ============================================================

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
pub enum ProductCondition {
    NewWithTags,
    Excellent,
    Good,
    Fair,
}

// ============================================================
// CLOTHING
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "VARCHAR", rename_all = "lowercase")]
pub enum ProductFit {
    Regular,
    Slim,
    Oversized,
    Relaxed,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ClothingDetails {
    pub product_id: Uuid,
    pub size: Option<String>,
    pub fit: Option<ProductFit>,
}

// ============================================================
// FOOTWEAR
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "VARCHAR", rename_all = "UPPERCASE")]
pub enum SizeSystem {
    EU,
    US,
    UK,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FootwearDetails {
    pub product_id: Uuid,
    pub shoe_size: Option<String>,
    pub size_system: SizeSystem,
}

// ============================================================
// BAGS
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "VARCHAR", rename_all = "lowercase")]
pub enum HandleType {
    Shoulder,
    Crossbody,
    Hand,
    Backpack,
    Tote,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BagDetails {
    pub product_id: Uuid,
    pub width_cm: Option<Decimal>,
    pub height_cm: Option<Decimal>,
    pub depth_cm: Option<Decimal>,
    pub handle_type: Option<HandleType>,
}

// ============================================================
// JEWELRY
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct JewelryDetails {
    pub product_id: Uuid,
    pub metal: Option<String>,
    pub stone: Option<String>,
    pub clasp_type: Option<String>,
}
