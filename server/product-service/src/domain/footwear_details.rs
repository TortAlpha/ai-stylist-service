use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Default, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "VARCHAR", rename_all = "UPPERCASE")]
pub enum SizeSystem {
    #[default]
    EU,
    US,
    UK,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FootwearDetails {
    pub product_id: Uuid,
    pub shoe_size: Option<String>,
    pub size_system: SizeSystem,
    pub insole_length_cm: Option<Decimal>,
}