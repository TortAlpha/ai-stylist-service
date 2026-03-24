use std::sync::Arc;

use crate::domain::error::ServiceError;
use crate::repo::traits::category_repo::CategoryRepository;

type Result<T> = std::result::Result<T, ServiceError>;

pub struct CategoryService {
    repo: Arc<dyn CategoryRepository>,
}

impl CategoryService {
    pub fn new(repo: Arc<dyn CategoryRepository>) -> Self {
        Self { repo }
    }
}
