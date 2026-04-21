use std::sync::Arc;

use crate::domain::error::ServiceError;
use crate::domain::request_dto::category::{CreateCategoryRequest, UpdateCategoryRequest};
use crate::domain::response_dto::category::CategoryFullResponse;
use crate::domain::utils::query::CategoryListQuery;
use crate::domain::value_objects::{CategoryName, EntityCode};
use crate::repo::traits::category_repo::CategoryRepository;

type Result<T> = std::result::Result<T, ServiceError>;

pub struct CategoryService {
    repo: Arc<dyn CategoryRepository>,
}

impl CategoryService {
    pub fn new(repo: Arc<dyn CategoryRepository>) -> Self {
        Self { repo }
    }

    pub async fn get_by_id(&self, id: i32) -> Result<CategoryFullResponse> {
        let category = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| ServiceError::NotFound(format!("category id={id} not found")))?;
        Ok(category.into())
    }

    pub async fn list(&self, query: &CategoryListQuery) -> Result<Vec<CategoryFullResponse>> {
        let items = match query.parent_id {
            Some(_) => self.repo.list_by_parent(query.parent_id).await?,
            None => self.repo.list_all().await?,
        };
        Ok(items.into_iter().map(Into::into).collect())
    }

    pub async fn create(&self, req: &CreateCategoryRequest) -> Result<CategoryFullResponse> {
        CategoryName::parse(&req.name)?;
        EntityCode::parse(&req.code, "category code")?;

        if let Some(pid) = req.parent_id {
            self.repo.find_by_id(pid).await?.ok_or_else(|| {
                ServiceError::BadRequest(format!("parent category id={pid} not found"))
            })?;
        }

        Ok(self.repo.create(req).await?.into())
    }

    pub async fn update_by_id(
        &self,
        id: i32,
        req: &UpdateCategoryRequest,
    ) -> Result<CategoryFullResponse> {
        if let Some(ref name) = req.name {
            CategoryName::parse(name)?;
        }
        if let Some(ref code) = req.code {
            EntityCode::parse(code, "category code")?;
        }
        if let Some(Some(pid)) = req.parent_id.as_ref().map(|&p| Some(p)) {
            if pid == id {
                return Err(ServiceError::BadRequest(
                    "category cannot be its own parent".into(),
                ));
            }
            self.repo.find_by_id(pid).await?.ok_or_else(|| {
                ServiceError::BadRequest(format!("parent category id={pid} not found"))
            })?;
        }

        Ok(self.repo.update_by_id(id, req).await?.into())
    }

    pub async fn delete_by_id(&self, id: i32) -> Result<()> {
        if self.repo.has_children(id).await? {
            return Err(ServiceError::Conflict(
                "cannot delete category that has child categories".into(),
            ));
        }

        let deleted = self.repo.delete_by_id(id).await?;
        if !deleted {
            return Err(ServiceError::NotFound(format!(
                "category id={id} not found"
            )));
        }
        Ok(())
    }
}
