use std::sync::Arc;

use argon2::{Argon2, PasswordHash, PasswordVerifier};
use rand::RngCore;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::auth::jwt::JwtManager;
use crate::clients::user_client::UserClient;
use crate::domain::error::ServiceError;
use crate::domain::request_dto::*;
use crate::domain::response_dto::*;
use crate::repo::traits::session_repo::SessionRepository;

pub struct AuthService {
    session_repo: Arc<dyn SessionRepository>,
    jwt_manager: Arc<JwtManager>,
    user_client: Arc<UserClient>,
}

impl AuthService {
    pub fn new(
        session_repo: Arc<dyn SessionRepository>,
        jwt_manager: Arc<JwtManager>,
        user_client: Arc<UserClient>,
    ) -> Self {
        Self {
            session_repo,
            jwt_manager,
            user_client,
        }
    }

    fn generate_refresh_token() -> String {
        let mut bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut bytes);
        hex::encode(bytes)
    }

    fn verify_password(password: &str, password_hash: &str) -> Result<(), ServiceError> {
        let parsed_hash = PasswordHash::new(password_hash)
            .map_err(|e| ServiceError::Internal(format!("Invalid password hash: {e}")))?;
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| ServiceError::Unauthorized("Invalid email or password".into()))
    }

    pub async fn login(&self, req: &LoginRequest) -> Result<AuthResponse, ServiceError> {
        debug!(email = %req.email, "service:auth login");
        let user = self
            .user_client
            .find_by_email(&req.email)
            .await?
            .ok_or_else(|| ServiceError::Unauthorized("Invalid email or password".into()))?;

        if !user.is_active {
            warn!(user_id = %user.id, email = %req.email, "service:auth login for deactivated account");
            return Err(ServiceError::Unauthorized("Account is deactivated".into()));
        }

        Self::verify_password(&req.password, &user.password)?;

        let access_token = self.jwt_manager.encode_access_token(user.id, &user.role)?;
        let refresh_token = Self::generate_refresh_token();
        let device_type = req.device_type.as_deref().unwrap_or("unknown");

        self.session_repo
            .create(user.id, &refresh_token, &user.role, device_type)
            .await
            .map_err(|e| ServiceError::Internal(format!("Failed to create session: {e}")))?;

        info!(user_id = %user.id, role = %user.role, "service:auth login succeeded");
        Ok(AuthResponse {
            access_token,
            refresh_token,
        })
    }

    pub async fn refresh(&self, req: &RefreshRequest) -> Result<AuthResponse, ServiceError> {
        debug!("service:auth refresh");
        let session = self
            .session_repo
            .find_by_refresh_token(&req.refresh_token)
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?
            .ok_or_else(|| ServiceError::Unauthorized("Invalid refresh token".into()))?;

        let new_refresh = Self::generate_refresh_token();
        let new_access = self
            .jwt_manager
            .encode_access_token(session.user_id, &session.role)?;

        self.session_repo
            .update_refresh_token(&req.refresh_token, &new_refresh)
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;

        info!(user_id = %session.user_id, role = %session.role, "service:auth refresh succeeded");
        Ok(AuthResponse {
            access_token: new_access,
            refresh_token: new_refresh,
        })
    }

    pub async fn logout(&self, refresh_token: &str) -> Result<(), ServiceError> {
        debug!("service:auth logout");
        self.session_repo
            .delete_by_refresh_token(refresh_token)
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;
        info!("service:auth logout succeeded");
        Ok(())
    }

    pub async fn logout_all(&self, user_id: Uuid) -> Result<u64, ServiceError> {
        debug!(%user_id, "service:auth logout_all");
        let deleted = self
            .session_repo
            .delete_all_by_user(user_id)
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;
        info!(%user_id, deleted_sessions = deleted, "service:auth logout_all succeeded");
        Ok(deleted)
    }

    pub async fn validate(&self, req: &ValidateRequest) -> Result<ValidateResponse, ServiceError> {
        debug!("service:auth validate");
        let claims = self.jwt_manager.decode_access_token(&req.access_token)?;
        info!(user_id = %claims.sub, role = %claims.role, "service:auth validate succeeded");
        Ok(ValidateResponse {
            user_id: claims.sub,
            role: claims.role,
        })
    }
}
