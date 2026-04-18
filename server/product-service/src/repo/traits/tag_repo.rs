use async_trait::async_trait;
use sqlx::Error;

use crate::domain::tags::{Season, StyleTag, VibeTag};

#[cfg_attr(any(test, feature = "test-mocks"), mockall::automock)]
#[async_trait]
pub trait TagRepository: Send + Sync {
    async fn list_style_tags(&self) -> Result<Vec<StyleTag>, Error>;
    async fn create_style_tag(&self, name: &str) -> Result<StyleTag, Error>;

    async fn list_vibe_tags(&self) -> Result<Vec<VibeTag>, Error>;
    async fn create_vibe_tag(&self, name: &str) -> Result<VibeTag, Error>;

    async fn list_seasons(&self) -> Result<Vec<Season>, Error>;
    async fn create_season(&self, name: &str) -> Result<Season, Error>;

    async fn delete_style_tag(&self, id: i32) -> Result<bool, Error>;
    async fn delete_vibe_tag(&self, id: i32) -> Result<bool, Error>;
    async fn delete_season(&self, id: i32) -> Result<bool, Error>;
}
