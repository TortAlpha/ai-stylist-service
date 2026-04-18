use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use product_service::domain::error::ServiceError;
use product_service::domain::purchase_location::PurchaseLocation;
use product_service::domain::value_objects::PurchaseLocationName;
use product_service::repo::traits::purchase_location_repo::PurchaseLocationRepository;
use product_service::service::purchase_location_service::PurchaseLocationService;

fn test_location(id: i32, name: &str) -> PurchaseLocation {
    PurchaseLocation {
        id,
        name: name.to_string(),
        created_at: Utc::now(),
    }
}

struct RepoNeverCalled;

#[async_trait]
impl PurchaseLocationRepository for RepoNeverCalled {
    async fn list_all(&self) -> Result<Vec<PurchaseLocation>, sqlx::Error> {
        panic!("unexpected call to list_all");
    }

    async fn find_by_id(&self, _id: i32) -> Result<Option<PurchaseLocation>, sqlx::Error> {
        panic!("unexpected call to find_by_id");
    }

    async fn find_by_name(
        &self,
        _name: &PurchaseLocationName,
    ) -> Result<Option<PurchaseLocation>, sqlx::Error> {
        panic!("unexpected call to find_by_name");
    }

    async fn create(&self, _name: &PurchaseLocationName) -> Result<PurchaseLocation, sqlx::Error> {
        panic!("unexpected call to create");
    }

    async fn get_or_create_by_name(
        &self,
        _name: &PurchaseLocationName,
    ) -> Result<PurchaseLocation, sqlx::Error> {
        panic!("unexpected call to get_or_create_by_name");
    }

    async fn update_by_id(
        &self,
        _id: i32,
        _new_name: &PurchaseLocationName,
    ) -> Result<PurchaseLocation, sqlx::Error> {
        panic!("unexpected call to update_by_id");
    }

    async fn delete_by_id(&self, _id: i32) -> Result<bool, sqlx::Error> {
        panic!("unexpected call to delete_by_id");
    }
}

struct RepoCreateAssertsName;

#[async_trait]
impl PurchaseLocationRepository for RepoCreateAssertsName {
    async fn list_all(&self) -> Result<Vec<PurchaseLocation>, sqlx::Error> {
        panic!("unexpected call to list_all");
    }

    async fn find_by_id(&self, _id: i32) -> Result<Option<PurchaseLocation>, sqlx::Error> {
        panic!("unexpected call to find_by_id");
    }

    async fn find_by_name(
        &self,
        _name: &PurchaseLocationName,
    ) -> Result<Option<PurchaseLocation>, sqlx::Error> {
        panic!("unexpected call to find_by_name");
    }

    async fn create(&self, name: &PurchaseLocationName) -> Result<PurchaseLocation, sqlx::Error> {
        assert_eq!(name.as_str(), "Novi Sad");
        Ok(test_location(1, "Novi Sad"))
    }

    async fn get_or_create_by_name(
        &self,
        _name: &PurchaseLocationName,
    ) -> Result<PurchaseLocation, sqlx::Error> {
        panic!("unexpected call to get_or_create_by_name");
    }

    async fn update_by_id(
        &self,
        _id: i32,
        _new_name: &PurchaseLocationName,
    ) -> Result<PurchaseLocation, sqlx::Error> {
        panic!("unexpected call to update_by_id");
    }

    async fn delete_by_id(&self, _id: i32) -> Result<bool, sqlx::Error> {
        panic!("unexpected call to delete_by_id");
    }
}

struct RepoGetByIdNotFound;

#[async_trait]
impl PurchaseLocationRepository for RepoGetByIdNotFound {
    async fn list_all(&self) -> Result<Vec<PurchaseLocation>, sqlx::Error> {
        panic!("unexpected call to list_all");
    }

    async fn find_by_id(&self, id: i32) -> Result<Option<PurchaseLocation>, sqlx::Error> {
        assert_eq!(id, 42);
        Ok(None)
    }

    async fn find_by_name(
        &self,
        _name: &PurchaseLocationName,
    ) -> Result<Option<PurchaseLocation>, sqlx::Error> {
        panic!("unexpected call to find_by_name");
    }

    async fn create(&self, _name: &PurchaseLocationName) -> Result<PurchaseLocation, sqlx::Error> {
        panic!("unexpected call to create");
    }

    async fn get_or_create_by_name(
        &self,
        _name: &PurchaseLocationName,
    ) -> Result<PurchaseLocation, sqlx::Error> {
        panic!("unexpected call to get_or_create_by_name");
    }

    async fn update_by_id(
        &self,
        _id: i32,
        _new_name: &PurchaseLocationName,
    ) -> Result<PurchaseLocation, sqlx::Error> {
        panic!("unexpected call to update_by_id");
    }

    async fn delete_by_id(&self, _id: i32) -> Result<bool, sqlx::Error> {
        panic!("unexpected call to delete_by_id");
    }
}

struct RepoGetOrCreateAssertsName;

#[async_trait]
impl PurchaseLocationRepository for RepoGetOrCreateAssertsName {
    async fn list_all(&self) -> Result<Vec<PurchaseLocation>, sqlx::Error> {
        panic!("unexpected call to list_all");
    }

    async fn find_by_id(&self, _id: i32) -> Result<Option<PurchaseLocation>, sqlx::Error> {
        panic!("unexpected call to find_by_id");
    }

    async fn find_by_name(
        &self,
        _name: &PurchaseLocationName,
    ) -> Result<Option<PurchaseLocation>, sqlx::Error> {
        panic!("unexpected call to find_by_name");
    }

    async fn create(&self, _name: &PurchaseLocationName) -> Result<PurchaseLocation, sqlx::Error> {
        panic!("unexpected call to create");
    }

    async fn get_or_create_by_name(
        &self,
        name: &PurchaseLocationName,
    ) -> Result<PurchaseLocation, sqlx::Error> {
        assert_eq!(name.as_str(), "Belgrade Flea Market");
        Ok(test_location(7, "Belgrade Flea Market"))
    }

    async fn update_by_id(
        &self,
        _id: i32,
        _new_name: &PurchaseLocationName,
    ) -> Result<PurchaseLocation, sqlx::Error> {
        panic!("unexpected call to update_by_id");
    }

    async fn delete_by_id(&self, _id: i32) -> Result<bool, sqlx::Error> {
        panic!("unexpected call to delete_by_id");
    }
}

struct RepoUpdateAssertsArgs;

#[async_trait]
impl PurchaseLocationRepository for RepoUpdateAssertsArgs {
    async fn list_all(&self) -> Result<Vec<PurchaseLocation>, sqlx::Error> {
        panic!("unexpected call to list_all");
    }

    async fn find_by_id(&self, _id: i32) -> Result<Option<PurchaseLocation>, sqlx::Error> {
        panic!("unexpected call to find_by_id");
    }

    async fn find_by_name(
        &self,
        _name: &PurchaseLocationName,
    ) -> Result<Option<PurchaseLocation>, sqlx::Error> {
        panic!("unexpected call to find_by_name");
    }

    async fn create(&self, _name: &PurchaseLocationName) -> Result<PurchaseLocation, sqlx::Error> {
        panic!("unexpected call to create");
    }

    async fn get_or_create_by_name(
        &self,
        _name: &PurchaseLocationName,
    ) -> Result<PurchaseLocation, sqlx::Error> {
        panic!("unexpected call to get_or_create_by_name");
    }

    async fn update_by_id(
        &self,
        id: i32,
        new_name: &PurchaseLocationName,
    ) -> Result<PurchaseLocation, sqlx::Error> {
        assert_eq!(id, 11);
        assert_eq!(new_name.as_str(), "Belgrade Center");
        Ok(test_location(11, "Belgrade Center"))
    }

    async fn delete_by_id(&self, _id: i32) -> Result<bool, sqlx::Error> {
        panic!("unexpected call to delete_by_id");
    }
}

#[tokio::test]
async fn create_trims_name() {
    let service = PurchaseLocationService::new(Arc::new(RepoCreateAssertsName));

    let location = service
        .create("  Novi Sad  ")
        .await
        .expect("create should succeed");

    assert_eq!(location.name, "Novi Sad");
}

#[tokio::test]
async fn create_rejects_blank_name() {
    let service = PurchaseLocationService::new(Arc::new(RepoNeverCalled));

    let err = service
        .create("   ")
        .await
        .expect_err("blank name should be rejected");

    assert!(matches!(err, ServiceError::BadRequest(_)));
}

#[tokio::test]
async fn get_by_id_returns_not_found() {
    let service = PurchaseLocationService::new(Arc::new(RepoGetByIdNotFound));

    let err = service
        .get_by_id(42)
        .await
        .expect_err("missing id should return not found");

    assert!(matches!(err, ServiceError::NotFound(_)));
}

#[tokio::test]
async fn get_by_id_rejects_non_positive_id() {
    let service = PurchaseLocationService::new(Arc::new(RepoNeverCalled));

    let err = service
        .get_by_id(0)
        .await
        .expect_err("non-positive id should be rejected");

    assert!(matches!(err, ServiceError::BadRequest(_)));
}

#[tokio::test]
async fn get_or_create_trims_name() {
    let service = PurchaseLocationService::new(Arc::new(RepoGetOrCreateAssertsName));

    let location = service
        .get_or_create_by_name(" Belgrade Flea Market ")
        .await
        .expect("get_or_create should succeed");

    assert_eq!(location.id, 7);
}

#[tokio::test]
async fn update_by_id_validates_and_trims_name() {
    let service = PurchaseLocationService::new(Arc::new(RepoUpdateAssertsArgs));

    let location = service
        .update_by_id(11, "  Belgrade Center  ")
        .await
        .expect("update_by_id should succeed");

    assert_eq!(location.id, 11);
    assert_eq!(location.name, "Belgrade Center");
}
