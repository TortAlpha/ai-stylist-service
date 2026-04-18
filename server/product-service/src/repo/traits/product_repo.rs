use async_trait::async_trait;
use sqlx::{Error, Postgres, Transaction};
use uuid::Uuid;

use crate::domain::product::ProductFull;
use crate::domain::request_dto::product::{CreateProductRequest, UpdateProductRequest};
use crate::domain::response_dto::product::ProductFilterOptions;
use crate::domain::response_dto::product_details::AvailableSizesResponse;
use crate::domain::utils::query::{AvailableSizesQuery, FilterOptionsQuery, ProductListQuery};

#[async_trait]
pub trait ProductRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<ProductFull>, Error>;

    async fn search_by_query(
        &self,
        query: &ProductListQuery,
    ) -> Result<(Vec<ProductFull>, i64), Error>;

    async fn create(&self, req: &CreateProductRequest) -> Result<ProductFull, Error>;

    /// Same as `create` but on a caller-provided transaction, so the
    /// caller can enqueue additional work (e.g. jobs) before committing.
    async fn create_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        req: &CreateProductRequest,
    ) -> Result<ProductFull, Error>;

    async fn update(&self, id: Uuid, req: &UpdateProductRequest) -> Result<ProductFull, Error>;

    async fn update_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        id: Uuid,
        req: &UpdateProductRequest,
    ) -> Result<ProductFull, Error>;

    async fn soft_delete(&self, id: Uuid, expected_version: i32) -> Result<bool, Error>;

    async fn filter_options(
        &self,
        query: &FilterOptionsQuery,
    ) -> Result<ProductFilterOptions, Error>;

    async fn available_sizes(
        &self,
        query: &AvailableSizesQuery,
    ) -> Result<AvailableSizesResponse, Error>;
}
