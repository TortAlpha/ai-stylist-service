use crate::domain::request_dto::{CreateUserRequest, UpdateUserRequest};
use crate::domain::user::User;
use async_trait::async_trait;
use uuid::Uuid;

#[cfg_attr(feature = "test-mocks", mockall::automock)]
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, req: &CreateUserRequest, password_hash: &str) -> sqlx::Result<User>;

    async fn find_by_id(&self, id: Uuid) -> sqlx::Result<Option<User>>;
    async fn find_by_email(&self, email: &str) -> sqlx::Result<Option<User>>;
    async fn update(&self, id: Uuid, req: &UpdateUserRequest) -> sqlx::Result<Option<User>>;
    async fn soft_delete(&self, id: Uuid) -> sqlx::Result<bool>;
}
