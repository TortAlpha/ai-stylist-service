use actix_web::{web, HttpResponse};
use tracing::{debug, info};

use crate::domain::error::ServiceError;
use crate::domain::request_dto::ValidateRequest;
use crate::service::auth_service::AuthService;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/internal")
            .route("/auth/validate", web::post().to(validate)),
    );
}

async fn validate(
    svc: web::Data<AuthService>,
    body: web::Json<ValidateRequest>,
) -> Result<HttpResponse, ServiceError> {
    debug!("internal auth validate request");
    let result = svc.validate(&body).await?;
    info!(user_id = %result.user_id, role = %result.role, "internal auth validate succeeded");
    Ok(HttpResponse::Ok().json(result))
}
