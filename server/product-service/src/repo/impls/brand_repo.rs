use async_trait::async_trait;
use sqlx::{Error, PgPool};

use crate::domain::brand::Brand;
use crate::domain::request_dto::brand::{CreateBrandRequest, UpdateBrandRequest};
use crate::domain::utils::pagination::PaginationParams;
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
    async fn find_by_id(&self, id: i32) -> Result<Option<Brand>, Error> {
        sqlx::query_as::<_, Brand>(
            "SELECT id, name, code, tier, country, created_at FROM brand WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    async fn list_all(&self) -> Result<Vec<Brand>, Error> {
        sqlx::query_as::<_, Brand>(
            "SELECT id, name, code, tier, country, created_at FROM brand ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await
    }

    async fn search(
        &self,
        search: &str,
        pagination: &PaginationParams,
    ) -> Result<(Vec<Brand>, i64), Error> {
        let (_, per_page, offset) = pagination.resolve();
        let pattern = format!("%{}%", search);

        let count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM brand WHERE name ILIKE $1 OR code ILIKE $1")
                .bind(&pattern)
                .fetch_one(&self.pool)
                .await?;

        let items = sqlx::query_as::<_, Brand>(
            r#"
            SELECT id, name, code, tier, country, created_at
            FROM brand
            WHERE name ILIKE $1 OR code ILIKE $1
            ORDER BY name
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(&pattern)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok((items, count.0))
    }

    async fn create(&self, req: &CreateBrandRequest) -> Result<Brand, Error> {
        sqlx::query_as::<_, Brand>(
            r#"
            INSERT INTO brand (name, code, tier, country)
            VALUES ($1, $2, $3, $4)
            RETURNING id, name, code, tier, country, created_at
            "#,
        )
        .bind(&req.name)
        .bind(&req.code)
        .bind(&req.tier)
        .bind(&req.country)
        .fetch_one(&self.pool)
        .await
    }

    async fn update_by_id(&self, id: i32, req: &UpdateBrandRequest) -> Result<Brand, Error> {
        sqlx::query_as::<_, Brand>(
            r#"
            UPDATE brand SET
                name = COALESCE($2, name),
                code = COALESCE($3, code),
                tier = COALESCE($4, tier),
                country = COALESCE($5, country)
            WHERE id = $1
            RETURNING id, name, code, tier, country, created_at
            "#,
        )
        .bind(id)
        .bind(&req.name)
        .bind(&req.code)
        .bind(&req.tier)
        .bind(&req.country)
        .fetch_one(&self.pool)
        .await
    }

    async fn delete_by_id(&self, id: i32) -> Result<bool, Error> {
        let result = sqlx::query("DELETE FROM brand WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}
