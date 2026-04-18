#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use chrono::Utc;
    use uuid::Uuid;

    use user_service::domain::address::Address;
    use user_service::domain::request_dto::*;
    use user_service::domain::user::User;
    use user_service::repo::traits::address_repo::MockAddressRepository;
    use user_service::repo::traits::user_repo::MockUserRepository;
    use user_service::service::user_service::UserService;

    fn mock_user() -> User {
        User {
            id: Uuid::new_v4(),
            name: "John".into(),
            surname: "Doe".into(),
            email: "john@example.com".into(),
            phone_number: None,
            is_active: true,
            password: "$argon2id$hash".into(),
            role: "user".into(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn mock_address(owner_id: Uuid) -> Address {
        Address {
            id: Uuid::new_v4(),
            owner_id,
            street: "Main St".into(),
            building_num: "1".into(),
            floor_num: Some(2),
            apartment_num: Some("3A".into()),
            post_index: "11000".into(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_register_user() {
        let mut user_repo = MockUserRepository::new();
        let address_repo = MockAddressRepository::new();

        let expected_user = mock_user();
        let ret = expected_user.clone();

        user_repo
            .expect_create()
            .returning(move |_, _| Ok(ret.clone()));

        let svc = UserService::new(Arc::new(user_repo), Arc::new(address_repo));

        let req = CreateUserRequest {
            name: "John".into(),
            surname: "Doe".into(),
            email: "john@example.com".into(),
            password: "password123".into(),
            phone_number: None,
        };

        let result = svc.register(&req).await.unwrap();
        assert_eq!(result.email, expected_user.email);
    }

    #[tokio::test]
    async fn test_get_user_not_found() {
        let mut user_repo = MockUserRepository::new();
        let address_repo = MockAddressRepository::new();

        user_repo.expect_find_by_id().returning(|_| Ok(None));

        let svc = UserService::new(Arc::new(user_repo), Arc::new(address_repo));
        let result = svc.get_by_id(Uuid::new_v4()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_addresses() {
        let user_repo = MockUserRepository::new();
        let mut address_repo = MockAddressRepository::new();

        let user_id = Uuid::new_v4();
        let addr = mock_address(user_id);

        let addr_clone = addr.clone();
        address_repo
            .expect_find_by_owner()
            .returning(move |_| Ok(vec![addr_clone.clone()]));

        let svc = UserService::new(Arc::new(user_repo), Arc::new(address_repo));
        let addresses = svc.get_addresses(user_id).await.unwrap();
        assert_eq!(addresses.len(), 1);
    }

    #[tokio::test]
    async fn test_soft_delete_user() {
        let mut user_repo = MockUserRepository::new();
        let address_repo = MockAddressRepository::new();

        user_repo.expect_soft_delete().returning(|_| Ok(true));

        let svc = UserService::new(Arc::new(user_repo), Arc::new(address_repo));
        let result = svc.soft_delete(Uuid::new_v4()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_soft_delete_user_not_found() {
        let mut user_repo = MockUserRepository::new();
        let address_repo = MockAddressRepository::new();

        user_repo.expect_soft_delete().returning(|_| Ok(false));

        let svc = UserService::new(Arc::new(user_repo), Arc::new(address_repo));
        let result = svc.soft_delete(Uuid::new_v4()).await;
        assert!(result.is_err());
    }
}
