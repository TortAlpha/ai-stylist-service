use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct ValidateResponse {
    pub user_id: Uuid,
    pub role: String,
}
