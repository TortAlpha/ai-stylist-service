use actix_web::{web, HttpResponse};

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
    let result = svc.validate(&body).await?;
    Ok(HttpResponse::Ok().json(result))
}
