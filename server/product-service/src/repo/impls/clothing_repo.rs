use async_trait::async_trait;
use sqlx::PgPool;

use crate::repo::traits::clothing_repo::ClothingRepository;

pub struct PgClothingRepo {
    pool: PgPool,
}

impl PgClothingRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ClothingRepository for PgClothingRepo {
}
