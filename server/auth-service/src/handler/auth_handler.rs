use actix_web::{web, HttpRequest, HttpResponse};

use crate::domain::error::ServiceError;
use crate::domain::request_dto::*;
use crate::service::auth_service::AuthService;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/login", web::post().to(login))
            .route("/refresh", web::post().to(refresh))
            .route("/logout", web::post().to(logout))
            .route("/verify", web::get().to(verify)),
    );
}

async fn login(
    svc: web::Data<AuthService>,
    body: web::Json<LoginRequest>,
) -> Result<HttpResponse, ServiceError> {
    let result = svc.login(&body).await?;
    Ok(HttpResponse::Ok().json(result))
}

async fn refresh(
    svc: web::Data<AuthService>,
    body: web::Json<RefreshRequest>,
) -> Result<HttpResponse, ServiceError> {
    let result = svc.refresh(&body).await?;
    Ok(HttpResponse::Ok().json(result))
}

async fn logout(
    svc: web::Data<AuthService>,
    body: web::Json<RefreshRequest>,
) -> Result<HttpResponse, ServiceError> {
    svc.logout(&body.refresh_token).await?;
    Ok(HttpResponse::NoContent().finish())
}

/// Called by nginx `auth_request`. Reads Bearer token from Authorization header,
/// validates it, and returns user info via response headers for `auth_request_set`.
async fn verify(
    svc: web::Data<AuthService>,
    req: HttpRequest,
) -> Result<HttpResponse, ServiceError> {
    let header = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| ServiceError::Unauthorized("Missing Authorization header".into()))?;

    let token = header
        .strip_prefix("Bearer ")
        .ok_or_else(|| ServiceError::Unauthorized("Invalid Authorization format".into()))?;

    let validate_req = ValidateRequest {
        access_token: token.to_string(),
    };
    let result = svc.validate(&validate_req).await?;

    Ok(HttpResponse::Ok()
        .insert_header(("X-User-Id", result.user_id.to_string()))
        .insert_header(("X-User-Role", result.role))
        .finish())
}
