use std::sync::Arc;

use crate::domain::error::ServiceError;
use crate::domain::response_dto::tag::TagResponse;
use crate::domain::value_objects::TagName;
use crate::repo::traits::tag_repo::TagRepository;

type Result<T> = std::result::Result<T, ServiceError>;

pub struct TagService {
    repo: Arc<dyn TagRepository>,
}

impl TagService {
    pub fn new(repo: Arc<dyn TagRepository>) -> Self {
        Self { repo }
    }

    // Style tags
    pub async fn list_style_tags(&self) -> Result<Vec<TagResponse>> {
        Ok(self
            .repo
            .list_style_tags()
            .await?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    pub async fn create_style_tag(&self, name: &str) -> Result<TagResponse> {
        let parsed = TagName::parse(name)?;
        Ok(self.repo.create_style_tag(parsed.as_str()).await?.into())
    }

    // Vibe tags
    pub async fn list_vibe_tags(&self) -> Result<Vec<TagResponse>> {
        Ok(self
            .repo
            .list_vibe_tags()
            .await?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    pub async fn create_vibe_tag(&self, name: &str) -> Result<TagResponse> {
        let parsed = TagName::parse(name)?;
        Ok(self.repo.create_vibe_tag(parsed.as_str()).await?.into())
    }

    // Seasons
    pub async fn list_seasons(&self) -> Result<Vec<TagResponse>> {
        Ok(self
            .repo
            .list_seasons()
            .await?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    pub async fn create_season(&self, name: &str) -> Result<TagResponse> {
        let parsed = TagName::parse(name)?;
        Ok(self.repo.create_season(parsed.as_str()).await?.into())
    }

    // Deletes
    pub async fn delete_style_tag(&self, id: i32) -> Result<()> {
        if !self.repo.delete_style_tag(id).await? {
            return Err(ServiceError::NotFound(format!(
                "style tag id={id} not found"
            )));
        }
        Ok(())
    }

    pub async fn delete_vibe_tag(&self, id: i32) -> Result<()> {
        if !self.repo.delete_vibe_tag(id).await? {
            return Err(ServiceError::NotFound(format!(
                "vibe tag id={id} not found"
            )));
        }
        Ok(())
    }

    pub async fn delete_season(&self, id: i32) -> Result<()> {
        if !self.repo.delete_season(id).await? {
            return Err(ServiceError::NotFound(format!("season id={id} not found")));
        }
        Ok(())
    }
}
