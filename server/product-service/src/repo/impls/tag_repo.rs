use async_trait::async_trait;
use sqlx::{Error, PgPool};

use crate::domain::tags::{Season, StyleTag, VibeTag};
use crate::repo::traits::tag_repo::TagRepository;

pub struct PgTagRepo {
    pool: PgPool,
}

impl PgTagRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TagRepository for PgTagRepo {
    async fn list_style_tags(&self) -> Result<Vec<StyleTag>, Error> {
        sqlx::query_as::<_, StyleTag>("SELECT id, name FROM style_tag ORDER BY name")
            .fetch_all(&self.pool)
            .await
    }

    async fn create_style_tag(&self, name: &str) -> Result<StyleTag, Error> {
        sqlx::query_as::<_, StyleTag>("INSERT INTO style_tag (name) VALUES ($1) RETURNING id, name")
            .bind(name)
            .fetch_one(&self.pool)
            .await
    }

    async fn list_vibe_tags(&self) -> Result<Vec<VibeTag>, Error> {
        sqlx::query_as::<_, VibeTag>("SELECT id, name FROM vibe_tag ORDER BY name")
            .fetch_all(&self.pool)
            .await
    }

    async fn create_vibe_tag(&self, name: &str) -> Result<VibeTag, Error> {
        sqlx::query_as::<_, VibeTag>("INSERT INTO vibe_tag (name) VALUES ($1) RETURNING id, name")
            .bind(name)
            .fetch_one(&self.pool)
            .await
    }

    async fn list_seasons(&self) -> Result<Vec<Season>, Error> {
        sqlx::query_as::<_, Season>("SELECT id, name FROM season ORDER BY name")
            .fetch_all(&self.pool)
            .await
    }

    async fn create_season(&self, name: &str) -> Result<Season, Error> {
        sqlx::query_as::<_, Season>("INSERT INTO season (name) VALUES ($1) RETURNING id, name")
            .bind(name)
            .fetch_one(&self.pool)
            .await
    }

    async fn delete_style_tag(&self, id: i32) -> Result<bool, Error> {
        let r = sqlx::query("DELETE FROM style_tag WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(r.rows_affected() > 0)
    }

    async fn delete_vibe_tag(&self, id: i32) -> Result<bool, Error> {
        let r = sqlx::query("DELETE FROM vibe_tag WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(r.rows_affected() > 0)
    }

    async fn delete_season(&self, id: i32) -> Result<bool, Error> {
        let r = sqlx::query("DELETE FROM season WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(r.rows_affected() > 0)
    }
}
