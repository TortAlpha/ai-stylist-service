use async_trait::async_trait;
use sqlx::Error;

use crate::domain::category::Category;
use crate::domain::request_dto::category::{CreateCategoryRequest, UpdateCategoryRequest};

#[cfg_attr(any(test, feature = "test-mocks"), mockall::automock)]
#[async_trait]
pub trait CategoryRepository: Send + Sync {
    async fn find_by_id(&self, id: i32) -> Result<Option<Category>, Error>;
    async fn list_all(&self) -> Result<Vec<Category>, Error>;
    async fn list_by_parent(&self, parent_id: Option<i32>) -> Result<Vec<Category>, Error>;
    async fn create(&self, req: &CreateCategoryRequest) -> Result<Category, Error>;
    async fn update_by_id(&self, id: i32, req: &UpdateCategoryRequest) -> Result<Category, Error>;
    async fn delete_by_id(&self, id: i32) -> Result<bool, Error>;
    async fn has_children(&self, id: i32) -> Result<bool, Error>;
}
