use std::sync::Arc;

use crate::domain::error::ServiceError;
use crate::domain::request_dto::brand::{CreateBrandRequest, UpdateBrandRequest};
use crate::domain::response_dto::brand::BrandResponse;
use crate::domain::utils::pagination::PaginatedResponse;
use crate::domain::utils::query::BrandSearchQuery;
use crate::domain::value_objects::{BrandName, EntityCode};
use crate::repo::traits::brand_repo::BrandRepository;

type Result<T> = std::result::Result<T, ServiceError>;

pub struct BrandService {
    repo: Arc<dyn BrandRepository>,
}

impl BrandService {
    pub fn new(repo: Arc<dyn BrandRepository>) -> Self {
        Self { repo }
    }

    pub async fn get_by_id(&self, id: i32) -> Result<BrandResponse> {
        let brand = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| ServiceError::NotFound(format!("brand id={id} not found")))?;
        Ok(brand.into())
    }

    pub async fn list_all(&self) -> Result<Vec<BrandResponse>> {
        let items = self.repo.list_all().await?;
        Ok(items.into_iter().map(Into::into).collect())
    }

    pub async fn search(
        &self,
        query: &BrandSearchQuery,
    ) -> Result<PaginatedResponse<BrandResponse>> {
        let pagination = query.pagination();
        let (items, total) = self.repo.search(&query.search, &pagination).await?;
        let items: Vec<BrandResponse> = items.into_iter().map(Into::into).collect();
        Ok(pagination.paginate(items, total))
    }

    pub async fn create(&self, req: &CreateBrandRequest) -> Result<BrandResponse> {
        BrandName::parse(&req.name)?;
        EntityCode::parse(&req.code, "brand code")?;
        Ok(self.repo.create(req).await?.into())
    }

    pub async fn update_by_id(&self, id: i32, req: &UpdateBrandRequest) -> Result<BrandResponse> {
        if let Some(ref name) = req.name {
            BrandName::parse(name)?;
        }
        if let Some(ref code) = req.code {
            EntityCode::parse(code, "brand code")?;
        }
        Ok(self.repo.update_by_id(id, req).await?.into())
    }

    pub async fn delete_by_id(&self, id: i32) -> Result<()> {
        let deleted = self.repo.delete_by_id(id).await?;
        if !deleted {
            return Err(ServiceError::NotFound(format!("brand id={id} not found")));
        }
        Ok(())
    }
}
