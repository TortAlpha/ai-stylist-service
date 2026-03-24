use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Address {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub street: String,
    pub building_num: String,
    pub floor_num: Option<i32>,
    pub apartment_num: Option<String>,
    pub post_index: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
