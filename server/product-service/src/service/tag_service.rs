use std::sync::Arc;

use crate::domain::error::ServiceError;
use crate::repo::traits::tag_repo::TagRepository;

type Result<T> = std::result::Result<T, ServiceError>;

pub struct TagService {
    repo: Arc<dyn TagRepository>,
}

impl TagService {
    pub fn new(repo: Arc<dyn TagRepository>) -> Self {
        Self { repo }
    }
}
