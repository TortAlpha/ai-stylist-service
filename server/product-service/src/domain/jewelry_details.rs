use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct JewelryDetails {
    pub product_id: Uuid,
    pub metal: Option<String>,
    pub stone: Option<String>,
    pub clasp_type: Option<String>,
}
