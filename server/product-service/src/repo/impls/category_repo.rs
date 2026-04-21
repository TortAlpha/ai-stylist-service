use async_trait::async_trait;
use sqlx::{Error, PgPool};

use crate::domain::category::Category;
use crate::domain::request_dto::category::{CreateCategoryRequest, UpdateCategoryRequest};
use crate::repo::traits::category_repo::CategoryRepository;

pub struct PgCategoryRepo {
    pool: PgPool,
}

impl PgCategoryRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CategoryRepository for PgCategoryRepo {
    async fn find_by_id(&self, id: i32) -> Result<Option<Category>, Error> {
        sqlx::query_as::<_, Category>(
            "SELECT id, name, code, parent_id, gender, product_type, size_group FROM category WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    async fn list_all(&self) -> Result<Vec<Category>, Error> {
        sqlx::query_as::<_, Category>(
            "SELECT id, name, code, parent_id, gender, product_type, size_group FROM category ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await
    }

    async fn list_by_parent(&self, parent_id: Option<i32>) -> Result<Vec<Category>, Error> {
        match parent_id {
            Some(pid) => {
                sqlx::query_as::<_, Category>(
                    r#"
                    SELECT id, name, code, parent_id, gender, product_type, size_group
                    FROM category
                    WHERE parent_id = $1
                    ORDER BY name
                    "#,
                )
                .bind(pid)
                .fetch_all(&self.pool)
                .await
            }
            None => {
                sqlx::query_as::<_, Category>(
                    r#"
                    SELECT id, name, code, parent_id, gender, product_type, size_group
                    FROM category
                    WHERE parent_id IS NULL
                    ORDER BY name
                    "#,
                )
                .fetch_all(&self.pool)
                .await
            }
        }
    }

    async fn create(&self, req: &CreateCategoryRequest) -> Result<Category, Error> {
        sqlx::query_as::<_, Category>(
            r#"
            INSERT INTO category (name, code, parent_id, gender, product_type, size_group)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, name, code, parent_id, gender, product_type, size_group
            "#,
        )
        .bind(&req.name)
        .bind(&req.code)
        .bind(req.parent_id)
        .bind(&req.gender)
        .bind(&req.product_type)
        .bind(&req.size_group)
        .fetch_one(&self.pool)
        .await
    }

    async fn update_by_id(&self, id: i32, req: &UpdateCategoryRequest) -> Result<Category, Error> {
        sqlx::query_as::<_, Category>(
            r#"
            UPDATE category SET
                name = COALESCE($2, name),
                code = COALESCE($3, code),
                parent_id = COALESCE($4, parent_id),
                gender = COALESCE($5, gender),
                product_type = COALESCE($6, product_type),
                size_group = COALESCE($7, size_group)
            WHERE id = $1
            RETURNING id, name, code, parent_id, gender, product_type, size_group
            "#,
        )
        .bind(id)
        .bind(&req.name)
        .bind(&req.code)
        .bind(req.parent_id)
        .bind(&req.gender)
        .bind(&req.product_type)
        .bind(&req.size_group)
        .fetch_one(&self.pool)
        .await
    }

    async fn delete_by_id(&self, id: i32) -> Result<bool, Error> {
        let result = sqlx::query("DELETE FROM category WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    async fn has_children(&self, id: i32) -> Result<bool, Error> {
        let row: (bool,) =
            sqlx::query_as("SELECT EXISTS(SELECT 1 FROM category WHERE parent_id = $1)")
                .bind(id)
                .fetch_one(&self.pool)
                .await?;
        Ok(row.0)
    }
}
