use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct PurchaseLocation {
    pub id: i32,
    pub name: String,
    pub created_at: DateTime<Utc>,
}
