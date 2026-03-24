use async_trait::async_trait;
use sqlx::PgPool;

use crate::repo::traits::category_repo::CategoryRepository;

pub struct PgCategoryRepo {
    pool: PgPool,
}

impl PgCategoryRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CategoryRepository for PgCategoryRepo {
}
