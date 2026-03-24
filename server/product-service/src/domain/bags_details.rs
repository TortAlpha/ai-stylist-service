use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

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