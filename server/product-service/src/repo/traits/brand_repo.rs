use async_trait::async_trait;
use sqlx::Error;

use crate::domain::brand::Brand;
use crate::domain::request_dto::brand::{CreateBrandRequest, UpdateBrandRequest};
use crate::domain::utils::pagination::PaginationParams;

#[cfg_attr(any(test, feature = "test-mocks"), mockall::automock)]
#[async_trait]
pub trait BrandRepository: Send + Sync {
    async fn find_by_id(&self, id: i32) -> Result<Option<Brand>, Error>;
    async fn list_all(&self) -> Result<Vec<Brand>, Error>;
    async fn search(
        &self,
        search: &str,
        pagination: &PaginationParams,
    ) -> Result<(Vec<Brand>, i64), Error>;
    async fn create(&self, req: &CreateBrandRequest) -> Result<Brand, Error>;
    async fn update_by_id(&self, id: i32, req: &UpdateBrandRequest) -> Result<Brand, Error>;
    async fn delete_by_id(&self, id: i32) -> Result<bool, Error>;
}
