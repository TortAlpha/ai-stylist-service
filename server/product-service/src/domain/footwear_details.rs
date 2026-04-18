use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Default, Serialize, Deserialize, sqlx::Type, utoipa::ToSchema)]
#[sqlx(type_name = "VARCHAR", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ShoeWidth {
    Narrow,
    #[default]
    Regular,
    Wide,
}

impl ShoeWidth {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Narrow => "narrow",
            Self::Regular => "regular",
            Self::Wide => "wide",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FootwearDetails {
    pub product_id: Uuid,
    pub shoe_width: Option<ShoeWidth>,
    pub insole_length_cm: Option<Decimal>,
}
