use std::sync::Arc;

use crate::domain::error::ServiceError;
use crate::domain::purchase_location::PurchaseLocation;
use crate::domain::value_objects::PurchaseLocationName;
use crate::repo::traits::purchase_location_repo::PurchaseLocationRepository;
use crate::service::utils::validators::validate_positive_i32;

type Result<T> = std::result::Result<T, ServiceError>;

pub struct PurchaseLocationService {
    repo: Arc<dyn PurchaseLocationRepository>,
}

impl PurchaseLocationService {
    pub fn new(repo: Arc<dyn PurchaseLocationRepository>) -> Self {
        Self { repo }
    }

    pub async fn list_all(&self) -> Result<Vec<PurchaseLocation>> {
        Ok(self.repo.list_all().await?)
    }

    pub async fn get_by_id(&self, id: i32) -> Result<PurchaseLocation> {
        let id = validate_positive_i32(id, "purchase_location id")?;

        match self.repo.find_by_id(id).await? {
            Some(location) => Ok(location),
            None => Err(ServiceError::NotFound(format!(
                "purchase_location id={} not found",
                id
            ))),
        }
    }

    pub async fn find_by_name(&self, name: &str) -> Result<Option<PurchaseLocation>> {
        let name = PurchaseLocationName::parse(name)?;
        Ok(self.repo.find_by_name(&name).await?)
    }

    pub async fn create(&self, name: &str) -> Result<PurchaseLocation> {
        let name = PurchaseLocationName::parse(name)?;
        Ok(self.repo.create(&name).await?)
    }

    pub async fn get_or_create_by_name(&self, name: &str) -> Result<PurchaseLocation> {
        let name = PurchaseLocationName::parse(name)?;
        Ok(self.repo.get_or_create_by_name(&name).await?)
    }

    pub async fn update_by_id(&self, id: i32, new_name: &str) -> Result<PurchaseLocation> {
        let id = validate_positive_i32(id, "purchase_location id")?;
        let new_name = PurchaseLocationName::parse(new_name)?;
        Ok(self.repo.update_by_id(id, &new_name).await?)
    }

    pub async fn delete_by_id(&self, id: i32) -> Result<()> {
        let id = validate_positive_i32(id, "purchase_location id")?;
        let deleted = self.repo.delete_by_id(id).await?;
        if !deleted {
            return Err(ServiceError::NotFound(format!(
                "purchase_location id={id} not found"
            )));
        }
        Ok(())
    }
}
