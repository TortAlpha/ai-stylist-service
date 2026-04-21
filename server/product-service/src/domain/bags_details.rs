use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, utoipa::ToSchema)]
#[sqlx(type_name = "VARCHAR", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum HandleType {
    Shoulder,
    Crossbody,
    Hand,
    Backpack,
    Tote,
}

impl HandleType {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Shoulder => "shoulder",
            Self::Crossbody => "crossbody",
            Self::Hand => "hand",
            Self::Backpack => "backpack",
            Self::Tote => "tote",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BagDetails {
    pub product_id: Uuid,
    pub width_cm: Option<Decimal>,
    pub height_cm: Option<Decimal>,
    pub depth_cm: Option<Decimal>,
    pub handle_type: Option<HandleType>,
    pub bag_size_label: Option<String>,
}
