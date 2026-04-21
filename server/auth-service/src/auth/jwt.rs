use chrono::Utc;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use uuid::Uuid;

use crate::domain::claims::Claims;
use crate::domain::error::ServiceError;

pub struct JwtManager {
    secret: String,
    ttl_seconds: i64,
}

impl JwtManager {
    pub fn new(secret: String, ttl_seconds: i64) -> Self {
        Self {
            secret,
            ttl_seconds,
        }
    }

    pub fn encode_access_token(&self, user_id: Uuid, role: &str) -> Result<String, ServiceError> {
        let now = Utc::now().timestamp();
        let claims = Claims {
            sub: user_id,
            role: role.to_string(),
            iat: now,
            exp: now + self.ttl_seconds,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| ServiceError::Internal(format!("JWT encode failed: {e}")))
    }

    pub fn decode_access_token(&self, token: &str) -> Result<Claims, ServiceError> {
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default(),
        )
        .map(|data| data.claims)
        .map_err(|e| ServiceError::Unauthorized(format!("Invalid token: {e}")))
    }
}
