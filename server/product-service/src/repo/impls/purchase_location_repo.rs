use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::purchase_location::PurchaseLocation;
use crate::domain::value_objects::PurchaseLocationName;
use crate::repo::traits::purchase_location_repo::PurchaseLocationRepository;

pub struct PgPurchaseLocationRepo {
    pool: PgPool,
}

impl PgPurchaseLocationRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PurchaseLocationRepository for PgPurchaseLocationRepo {
    async fn list_all(&self) -> Result<Vec<PurchaseLocation>, sqlx::Error> {
        sqlx::query_as::<_, PurchaseLocation>(
            r#"
            SELECT id, name, created_at
            FROM purchase_location
            ORDER BY name ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await
    }

    async fn find_by_id(&self, id: i32) -> Result<Option<PurchaseLocation>, sqlx::Error> {
        sqlx::query_as::<_, PurchaseLocation>(
            r#"
            SELECT id, name, created_at
            FROM purchase_location
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    async fn find_by_name(
        &self,
        name: &PurchaseLocationName,
    ) -> Result<Option<PurchaseLocation>, sqlx::Error> {
        sqlx::query_as::<_, PurchaseLocation>(
            r#"
            SELECT id, name, created_at
            FROM purchase_location
            WHERE name = $1
            "#,
        )
        .bind(name.as_str())
        .fetch_optional(&self.pool)
        .await
    }

    async fn create(&self, name: &PurchaseLocationName) -> Result<PurchaseLocation, sqlx::Error> {
        sqlx::query_as::<_, PurchaseLocation>(
            r#"
            INSERT INTO purchase_location (name)
            VALUES ($1)
            RETURNING id, name, created_at
            "#,
        )
        .bind(name.as_str())
        .fetch_one(&self.pool)
        .await
    }

    async fn get_or_create_by_name(
        &self,
        name: &PurchaseLocationName,
    ) -> Result<PurchaseLocation, sqlx::Error> {
        sqlx::query_as::<_, PurchaseLocation>(
            r#"
            WITH inserted AS (
                INSERT INTO purchase_location (name)
                VALUES ($1)
                ON CONFLICT (name) DO NOTHING
                RETURNING id, name, created_at
            )
            SELECT id, name, created_at
            FROM inserted
            UNION ALL
            SELECT id, name, created_at
            FROM purchase_location
            WHERE name = $1
            LIMIT 1
            "#,
        )
        .bind(name.as_str())
        .fetch_one(&self.pool)
        .await
    }

    async fn update_by_id(
        &self,
        id: i32,
        new_name: &PurchaseLocationName,
    ) -> Result<PurchaseLocation, sqlx::Error> {
        sqlx::query_as::<_, PurchaseLocation>(
            r#"
            UPDATE purchase_location
            SET name = $2
            WHERE id = $1
            RETURNING id, name, created_at
            "#,
        )
        .bind(id)
        .bind(new_name.as_str())
        .fetch_one(&self.pool)
        .await
    }

    async fn delete_by_id(&self, id: i32) -> Result<bool, sqlx::Error> {
        let r = sqlx::query("DELETE FROM purchase_location WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(r.rows_affected() > 0)
    }
}
