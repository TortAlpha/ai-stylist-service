use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::session::Session;
use crate::repo::traits::session_repo::SessionRepository;

pub struct PgSessionRepo {
    pool: PgPool,
}

impl PgSessionRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SessionRepository for PgSessionRepo {
    async fn create(
        &self,
        user_id: Uuid,
        refresh_token: &str,
        role: &str,
        device_type: &str,
    ) -> sqlx::Result<Session> {
        sqlx::query_as::<_, Session>(
            r#"
            INSERT INTO sessions (user_id, refresh_token, role, device_type)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
        )
        .bind(user_id)
        .bind(refresh_token)
        .bind(role)
        .bind(device_type)
        .fetch_one(&self.pool)
        .await
    }

    async fn find_by_refresh_token(&self, refresh_token: &str) -> sqlx::Result<Option<Session>> {
        sqlx::query_as::<_, Session>("SELECT * FROM sessions WHERE refresh_token = $1")
            .bind(refresh_token)
            .fetch_optional(&self.pool)
            .await
    }

    async fn touch_by_previous_refresh_token(
        &self,
        refresh_token: &str,
    ) -> sqlx::Result<Option<Session>> {
        sqlx::query_as::<_, Session>(
            r#"
            UPDATE sessions
            SET last_activity_time = now()
            WHERE previous_refresh_token = $1
              AND previous_rotated_at > now() - INTERVAL '30 seconds'
            RETURNING *
            "#,
        )
        .bind(refresh_token)
        .fetch_optional(&self.pool)
        .await
    }

    async fn delete_by_refresh_token(&self, refresh_token: &str) -> sqlx::Result<bool> {
        let result = sqlx::query("DELETE FROM sessions WHERE refresh_token = $1")
            .bind(refresh_token)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    async fn delete_all_by_user(&self, user_id: Uuid) -> sqlx::Result<u64> {
        let result = sqlx::query("DELETE FROM sessions WHERE user_id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected())
    }

    async fn update_refresh_token(
        &self,
        old_token: &str,
        new_token: &str,
    ) -> sqlx::Result<Option<Session>> {
        sqlx::query_as::<_, Session>(
            r#"
            UPDATE sessions
            SET previous_refresh_token = refresh_token,
                previous_rotated_at = now(),
                refresh_token = $1,
                last_activity_time = now()
            WHERE refresh_token = $2
            RETURNING *
            "#,
        )
        .bind(new_token)
        .bind(old_token)
        .fetch_optional(&self.pool)
        .await
    }
}
