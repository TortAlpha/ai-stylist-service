use async_trait::async_trait;
use sqlx::PgPool;

use crate::repo::traits::product_repo::ProductRepository;

pub struct PgProductRepo {
    pool: PgPool,
}

impl PgProductRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProductRepository for PgProductRepo {
}
