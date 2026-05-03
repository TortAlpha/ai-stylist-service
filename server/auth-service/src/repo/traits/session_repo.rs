use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::session::Session;

#[cfg_attr(feature = "test-mocks", mockall::automock)]
#[async_trait]
pub trait SessionRepository: Send + Sync {
    async fn create(
        &self,
        user_id: Uuid,
        refresh_token: &str,
        role: &str,
        device_type: &str,
    ) -> sqlx::Result<Session>;
    async fn find_by_refresh_token(&self, refresh_token: &str) -> sqlx::Result<Option<Session>>;
    /// Look up a session by its previous (rotated-out) refresh token within
    /// the grace window and bump `last_activity_time` atomically.
    async fn touch_by_previous_refresh_token(
        &self,
        refresh_token: &str,
    ) -> sqlx::Result<Option<Session>>;
    async fn delete_by_refresh_token(&self, refresh_token: &str) -> sqlx::Result<bool>;
    async fn delete_all_by_user(&self, user_id: Uuid) -> sqlx::Result<u64>;
    async fn update_refresh_token(
        &self,
        old_token: &str,
        new_token: &str,
    ) -> sqlx::Result<Option<Session>>;
}
