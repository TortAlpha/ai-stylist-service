use serde::Deserialize;
use uuid::Uuid;

use crate::domain::error::ServiceError;

#[derive(Debug, Deserialize)]
pub struct UserAuthInfo {
    pub id: Uuid,
    pub email: String,
    pub password: String,
    pub role: String,
    pub is_active: bool,
}

pub struct UserClient {
    base_url: String,
    client: reqwest::Client,
}

impl UserClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::Client::new(),
        }
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<UserAuthInfo>, ServiceError> {
        let url = format!("{}/internal/users/by-email", self.base_url);
        let resp = self
            .client
            .get(&url)
            .query(&[("email", email)])
            .send()
            .await
            .map_err(|e| ServiceError::Internal(format!("User service request failed: {e}")))?;

        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        if !resp.status().is_success() {
            return Err(ServiceError::Internal(format!(
                "User service returned status {}",
                resp.status()
            )));
        }

        let user = resp
            .json::<UserAuthInfo>()
            .await
            .map_err(|e| ServiceError::Internal(format!("Failed to parse user response: {e}")))?;

        Ok(Some(user))
    }
}
