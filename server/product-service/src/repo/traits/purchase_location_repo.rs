use async_trait::async_trait;
use sqlx::Error;

use crate::domain::purchase_location::PurchaseLocation;
use crate::domain::value_objects::PurchaseLocationName;

#[cfg_attr(any(test, feature = "test-mocks"), mockall::automock)]
#[async_trait]
pub trait PurchaseLocationRepository: Send + Sync {
    async fn list_all(&self) -> Result<Vec<PurchaseLocation>, Error>;

    async fn find_by_id(&self, id: i32) -> Result<Option<PurchaseLocation>, Error>;

    async fn find_by_name(
        &self,
        name: &PurchaseLocationName,
    ) -> Result<Option<PurchaseLocation>, Error>;

    async fn create(&self, name: &PurchaseLocationName) -> Result<PurchaseLocation, Error>;

    async fn get_or_create_by_name(
        &self,
        name: &PurchaseLocationName,
    ) -> Result<PurchaseLocation, Error>;

    async fn update_by_id(
        &self,
        id: i32,
        new_name: &PurchaseLocationName,
    ) -> Result<PurchaseLocation, Error>;

    async fn delete_by_id(&self, id: i32) -> Result<bool, Error>;
}
