use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub name: String,
    pub surname: String,
    pub email: String,
    pub phone_number: Option<String>,
    pub is_active: bool,
    pub role: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Internal response for auth-service (includes password hash)
#[derive(Debug, Serialize)]
pub struct UserAuthResponse {
    pub id: Uuid,
    pub email: String,
    pub password: String,
    pub role: String,
    pub is_active: bool,
}

#[derive(Debug, Serialize)]
pub struct AddressResponse {
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
