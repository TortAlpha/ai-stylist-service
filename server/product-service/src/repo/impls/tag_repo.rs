use async_trait::async_trait;
use sqlx::PgPool;

use crate::repo::traits::tag_repo::TagRepository;

pub struct PgTagRepo {
    pool: PgPool,
}

impl PgTagRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TagRepository for PgTagRepo {
}
