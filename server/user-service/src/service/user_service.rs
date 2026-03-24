use std::sync::Arc;

use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use uuid::Uuid;

use crate::domain::address::Address;
use crate::domain::error::ServiceError;
use crate::domain::request_dto::*;
use crate::domain::user::User;
use crate::repo::traits::address_repo::AddressRepository;
use crate::repo::traits::user_repo::UserRepository;

pub struct UserService {
    user_repo: Arc<dyn UserRepository>,
    address_repo: Arc<dyn AddressRepository>,
}

impl UserService {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        address_repo: Arc<dyn AddressRepository>,
    ) -> Self {
        Self {
            user_repo,
            address_repo,
        }
    }

    fn hash_password(password: &str) -> Result<String, ServiceError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|h| h.to_string())
            .map_err(|e| ServiceError::Internal(format!("Password hashing failed: {e}")))
    }

    pub async fn register(&self, req: &CreateUserRequest) -> Result<User, ServiceError> {
        let password_hash = Self::hash_password(&req.password)?;
        let user = self.user_repo.create(req, &password_hash).await?;
        Ok(user)
    }


    pub async fn get_by_id(&self, id: Uuid) -> Result<User, ServiceError> {
        self.user_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| ServiceError::NotFound(format!("User {id} not found")))
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, ServiceError> {
        Ok(self.user_repo.find_by_email(email).await?)
    }

    pub async fn update(&self, id: Uuid, req: &UpdateUserRequest) -> Result<User, ServiceError> {
        self.user_repo
            .update(id, req)
            .await?
            .ok_or_else(|| ServiceError::NotFound(format!("User {id} not found")))
    }

    pub async fn soft_delete(&self, id: Uuid) -> Result<(), ServiceError> {
        if !self.user_repo.soft_delete(id).await? {
            return Err(ServiceError::NotFound(format!("User {id} not found")));
        }
        Ok(())
    }

    pub async fn create_address(
        &self,
        owner_id: Uuid,
        req: &CreateAddressRequest,
    ) -> Result<Address, ServiceError> {
        self.get_by_id(owner_id).await?;
        let address = self.address_repo.create(owner_id, req).await?;
        Ok(address)
    }

    pub async fn get_addresses(&self, owner_id: Uuid) -> Result<Vec<Address>, ServiceError> {
        Ok(self.address_repo.find_by_owner(owner_id).await?)
    }

    pub async fn update_address(
        &self,
        id: Uuid,
        req: &UpdateAddressRequest,
    ) -> Result<Address, ServiceError> {
        self.address_repo
            .update(id, req)
            .await?
            .ok_or_else(|| ServiceError::NotFound(format!("Address {id} not found")))
    }

    pub async fn delete_address(&self, id: Uuid) -> Result<(), ServiceError> {
        if !self.address_repo.delete(id).await? {
            return Err(ServiceError::NotFound(format!("Address {id} not found")));
        }
        Ok(())
    }
}
