use async_trait::async_trait;
use sqlx::PgPool;

use crate::repo::traits::jewelry_repo::JewelryRepository;

pub struct PgJewelryRepo {
    pool: PgPool,
}

impl PgJewelryRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl JewelryRepository for PgJewelryRepo {
}
