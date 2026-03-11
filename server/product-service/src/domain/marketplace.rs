use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Marketplace {
    pub id: i32,
    pub name: String,
    pub code: String,
    pub base_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProductListing {
    pub id: i32,
    pub product_id: Uuid,
    pub marketplace_id: i32,
    pub external_id: Option<String>,
    pub external_url: Option<String>,
    pub listing_price: Option<Decimal>,
    pub sold_price: Option<Decimal>,
    pub currency: Option<String>,
    pub status: ListingStatus,
    pub listed_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "VARCHAR", rename_all = "lowercase")]
pub enum ListingStatus {
    Active,
    Paused,
    Sold,
    Removed,
}
