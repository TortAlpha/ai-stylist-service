use async_trait::async_trait;
use uuid::Uuid;
use crate::domain::address::Address;
use crate::domain::request_dto::{CreateAddressRequest, UpdateAddressRequest};

#[cfg_attr(feature = "test-mocks", mockall::automock)]
#[async_trait]
pub trait AddressRepository: Send + Sync {
    async fn create(&self, owner_id: Uuid, req: &CreateAddressRequest) -> sqlx::Result<Address>;
    async fn find_by_id(&self, id: Uuid) -> sqlx::Result<Option<Address>>;
    async fn find_by_owner(&self, owner_id: Uuid) -> sqlx::Result<Vec<Address>>;
    async fn update(&self, id: Uuid, req: &UpdateAddressRequest) -> sqlx::Result<Option<Address>>;
    async fn delete(&self, id: Uuid) -> sqlx::Result<bool>;
}
