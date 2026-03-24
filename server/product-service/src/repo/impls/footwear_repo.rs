use async_trait::async_trait;
use sqlx::PgPool;

use crate::repo::traits::footwear_repo::FootwearRepository;

pub struct PgFootwearRepo {
    pool: PgPool,
}

impl PgFootwearRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl FootwearRepository for PgFootwearRepo {
}
