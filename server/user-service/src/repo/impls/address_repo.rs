use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::address::Address;
use crate::domain::request_dto::{CreateAddressRequest, UpdateAddressRequest};
use crate::repo::traits::address_repo::AddressRepository;

pub struct PgAddressRepo {
    pool: PgPool,
}

impl PgAddressRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AddressRepository for PgAddressRepo {
    async fn create(&self, owner_id: Uuid, req: &CreateAddressRequest) -> sqlx::Result<Address> {
        sqlx::query_as::<_, Address>(
            r#"
            INSERT INTO addresses (owner_id, street, building_num, floor_num, apartment_num, post_index)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(owner_id)
        .bind(&req.street)
        .bind(&req.building_num)
        .bind(req.floor_num)
        .bind(&req.apartment_num)
        .bind(&req.post_index)
        .fetch_one(&self.pool)
        .await
    }

    async fn find_by_id(&self, id: Uuid) -> sqlx::Result<Option<Address>> {
        sqlx::query_as::<_, Address>("SELECT * FROM addresses WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    async fn find_by_owner(&self, owner_id: Uuid) -> sqlx::Result<Vec<Address>> {
        sqlx::query_as::<_, Address>(
            "SELECT * FROM addresses WHERE owner_id = $1 ORDER BY created_at",
        )
        .bind(owner_id)
        .fetch_all(&self.pool)
        .await
    }

    async fn update(&self, id: Uuid, req: &UpdateAddressRequest) -> sqlx::Result<Option<Address>> {
        sqlx::query_as::<_, Address>(
            r#"
            UPDATE addresses
            SET street        = COALESCE($1, street),
                building_num  = COALESCE($2, building_num),
                floor_num     = COALESCE($3, floor_num),
                apartment_num = COALESCE($4, apartment_num),
                post_index    = COALESCE($5, post_index)
            WHERE id = $6
            RETURNING *
            "#,
        )
        .bind(&req.street)
        .bind(&req.building_num)
        .bind(req.floor_num)
        .bind(&req.apartment_num)
        .bind(&req.post_index)
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    async fn delete(&self, id: Uuid) -> sqlx::Result<bool> {
        let result = sqlx::query("DELETE FROM addresses WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}
