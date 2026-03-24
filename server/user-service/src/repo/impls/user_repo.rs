use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::request_dto::{CreateUserRequest, UpdateUserRequest};
use crate::domain::user::User;
use crate::repo::traits::user_repo::UserRepository;

pub struct PgUserRepo {
    pool: PgPool,
}

impl PgUserRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PgUserRepo {
    async fn create(&self, req: &CreateUserRequest, password_hash: &str) -> sqlx::Result<User> {
        let role = "user";
        sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (name, surname, email, phone_number, password, role)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(&req.name)
        .bind(&req.surname)
        .bind(&req.email)
        .bind(&req.phone_number)
        .bind(password_hash)
        .bind(role)
        .fetch_one(&self.pool)
        .await
    }

    async fn find_by_id(&self, id: Uuid) -> sqlx::Result<Option<User>> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1 AND is_active = true")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    async fn find_by_email(&self, email: &str) -> sqlx::Result<Option<User>> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(&self.pool)
            .await
    }

    async fn update(&self, id: Uuid, req: &UpdateUserRequest) -> sqlx::Result<Option<User>> {
        sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET name         = COALESCE($1, name),
                surname      = COALESCE($2, surname),
                email        = COALESCE($3, email),
                phone_number = COALESCE($4, phone_number)
            WHERE id = $5 AND is_active = true
            RETURNING *
            "#,
        )
        .bind(&req.name)
        .bind(&req.surname)
        .bind(&req.email)
        .bind(&req.phone_number)
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    async fn soft_delete(&self, id: Uuid) -> sqlx::Result<bool> {
        let result =
            sqlx::query("UPDATE users SET is_active = false WHERE id = $1 AND is_active = true")
                .bind(id)
                .execute(&self.pool)
                .await?;
        Ok(result.rows_affected() > 0)
    }
}
