use actix_web::dev::Payload;
use actix_web::{FromRequest, HttpRequest};
use std::future::Future;
use std::pin::Pin;
use uuid::Uuid;

use crate::domain::error::ServiceError;

/// Extractor: reads X-User-Id and X-User-Role headers set by nginx auth_request.
#[derive(Debug, Clone)]
pub struct ValidatedUser {
    pub user_id: Uuid,
    pub role: String,
}

impl FromRequest for ValidatedUser {
    type Error = ServiceError;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let user_id = req
            .headers()
            .get("X-User-Id")
            .and_then(|v| v.to_str().ok())
            .map(String::from);

        let role = req
            .headers()
            .get("X-User-Role")
            .and_then(|v| v.to_str().ok())
            .map(String::from);

        Box::pin(async move {
            let user_id_str = user_id
                .ok_or_else(|| ServiceError::Unauthorized("Missing X-User-Id header".into()))?;

            let user_id = Uuid::parse_str(&user_id_str)
                .map_err(|_| ServiceError::Unauthorized("Invalid X-User-Id".into()))?;

            let role = role
                .ok_or_else(|| ServiceError::Unauthorized("Missing X-User-Role header".into()))?;

            Ok(ValidatedUser { user_id, role })
        })
    }
}

/// Extractor: requires admin role. Wraps ValidatedUser.
#[derive(Debug, Clone)]
pub struct AdminUser {
    pub user_id: Uuid,
}

impl FromRequest for AdminUser {
    type Error = ServiceError;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let user_fut = ValidatedUser::from_request(req, payload);

        Box::pin(async move {
            let user = user_fut.await?;

            if user.role != "admin" {
                return Err(ServiceError::Forbidden("Admin access required".into()));
            }

            Ok(AdminUser {
                user_id: user.user_id,
            })
        })
    }
}
