use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub refresh_token: String,
    pub role: String,
    pub device_type: String,
    pub last_activity_time: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}
