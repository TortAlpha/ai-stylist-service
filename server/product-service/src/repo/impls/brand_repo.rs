use async_trait::async_trait;
use sqlx::PgPool;

use crate::repo::traits::brand_repo::BrandRepository;

pub struct PgBrandRepo {
    pool: PgPool,
}

impl PgBrandRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl BrandRepository for PgBrandRepo {
}
