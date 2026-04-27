#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use chrono::Utc;
    use uuid::Uuid;

    use auth_service::auth::jwt::JwtManager;
    use auth_service::clients::user_client::UserClient;
    use auth_service::domain::request_dto::*;
    use auth_service::domain::session::Session;
    use auth_service::repo::traits::session_repo::MockSessionRepository;
    use auth_service::service::auth_service::AuthService;

    fn mock_jwt() -> Arc<JwtManager> {
        Arc::new(JwtManager::new("test-secret".into(), 900))
    }

    fn mock_session(user_id: Uuid) -> Session {
        Session {
            id: Uuid::new_v4(),
            user_id,
            refresh_token: "test-refresh-token".into(),
            previous_refresh_token: None,
            previous_rotated_at: None,
            role: "user".into(),
            device_type: "test".into(),
            last_activity_time: Utc::now(),
            created_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_validate_valid_token() {
        let session_repo = MockSessionRepository::new();
        let jwt = mock_jwt();
        let user_client = Arc::new(UserClient::new("http://localhost:9999".into()));

        let svc = AuthService::new(Arc::new(session_repo), jwt.clone(), user_client);

        let user_id = Uuid::new_v4();
        let token = jwt.encode_access_token(user_id, "admin").unwrap();

        let req = ValidateRequest {
            access_token: token,
        };
        let result = svc.validate(&req).await.unwrap();

        assert_eq!(result.user_id, user_id);
        assert_eq!(result.role, "admin");
    }

    #[tokio::test]
    async fn test_validate_invalid_token() {
        let session_repo = MockSessionRepository::new();
        let jwt = mock_jwt();
        let user_client = Arc::new(UserClient::new("http://localhost:9999".into()));

        let svc = AuthService::new(Arc::new(session_repo), jwt, user_client);

        let req = ValidateRequest {
            access_token: "invalid-token".into(),
        };
        let result = svc.validate(&req).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_logout() {
        let mut session_repo = MockSessionRepository::new();
        let jwt = mock_jwt();
        let user_client = Arc::new(UserClient::new("http://localhost:9999".into()));

        session_repo
            .expect_delete_by_refresh_token()
            .returning(|_| Ok(true));

        let svc = AuthService::new(Arc::new(session_repo), jwt, user_client);
        let result = svc.logout("some-token").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_logout_all() {
        let mut session_repo = MockSessionRepository::new();
        let jwt = mock_jwt();
        let user_client = Arc::new(UserClient::new("http://localhost:9999".into()));

        session_repo
            .expect_delete_all_by_user()
            .returning(|_| Ok(3));

        let svc = AuthService::new(Arc::new(session_repo), jwt, user_client);
        let result = svc.logout_all(Uuid::new_v4()).await.unwrap();
        assert_eq!(result, 3);
    }

    #[tokio::test]
    async fn test_refresh_valid() {
        let mut session_repo = MockSessionRepository::new();
        let jwt = mock_jwt();
        let user_client = Arc::new(UserClient::new("http://localhost:9999".into()));

        let user_id = Uuid::new_v4();
        let session = mock_session(user_id);

        let session_clone = session.clone();
        session_repo
            .expect_find_by_refresh_token()
            .returning(move |_| Ok(Some(session_clone.clone())));

        session_repo
            .expect_update_refresh_token()
            .returning(move |_, new_token| {
                let mut s = mock_session(user_id);
                s.refresh_token = new_token.to_string();
                Ok(Some(s))
            });

        let svc = AuthService::new(Arc::new(session_repo), jwt, user_client);

        let req = RefreshRequest {
            refresh_token: "test-refresh-token".into(),
        };
        let result = svc.refresh(&req).await.unwrap();

        assert!(!result.access_token.is_empty());
        assert!(!result.refresh_token.is_empty());
        assert_ne!(result.refresh_token, "test-refresh-token");
    }

    #[tokio::test]
    async fn test_refresh_invalid_token() {
        let mut session_repo = MockSessionRepository::new();
        let jwt = mock_jwt();
        let user_client = Arc::new(UserClient::new("http://localhost:9999".into()));

        session_repo
            .expect_find_by_refresh_token()
            .returning(|_| Ok(None));
        session_repo
            .expect_touch_by_previous_refresh_token()
            .returning(|_| Ok(None));

        let svc = AuthService::new(Arc::new(session_repo), jwt, user_client);

        let req = RefreshRequest {
            refresh_token: "nonexistent".into(),
        };
        let result = svc.refresh(&req).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_refresh_grace_window_returns_current_token() {
        let mut session_repo = MockSessionRepository::new();
        let jwt = mock_jwt();
        let user_client = Arc::new(UserClient::new("http://localhost:9999".into()));

        let user_id = Uuid::new_v4();
        let mut current_session = mock_session(user_id);
        current_session.refresh_token = "current-token".into();
        current_session.previous_refresh_token = Some("rotated-token".into());
        current_session.previous_rotated_at = Some(Utc::now());

        session_repo
            .expect_find_by_refresh_token()
            .returning(|_| Ok(None));
        let session_clone = current_session.clone();
        session_repo
            .expect_touch_by_previous_refresh_token()
            .returning(move |_| Ok(Some(session_clone.clone())));

        let svc = AuthService::new(Arc::new(session_repo), jwt, user_client);
        let req = RefreshRequest {
            refresh_token: "rotated-token".into(),
        };
        let result = svc.refresh(&req).await.unwrap();

        assert_eq!(result.refresh_token, "current-token");
        assert!(!result.access_token.is_empty());
    }
}
