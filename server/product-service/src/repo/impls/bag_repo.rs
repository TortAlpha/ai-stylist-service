use async_trait::async_trait;
use sqlx::PgPool;

use crate::repo::traits::bag_repo::BagRepository;

pub struct PgBagRepo {
    pool: PgPool,
}

impl PgBagRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl BagRepository for PgBagRepo {
}
