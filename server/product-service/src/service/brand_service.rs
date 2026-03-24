use std::sync::Arc;

use crate::domain::error::ServiceError;
use crate::repo::traits::brand_repo::BrandRepository;

type Result<T> = std::result::Result<T, ServiceError>;

pub struct BrandService {
    repo: Arc<dyn BrandRepository>,
}

impl BrandService {
    pub fn new(repo: Arc<dyn BrandRepository>) -> Self {
        Self { repo }
    }
}
