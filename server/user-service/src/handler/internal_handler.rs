use actix_web::{web, HttpResponse};
use serde::Deserialize;

use crate::domain::error::ServiceError;
use crate::domain::mappers::user_to_auth_response;
use crate::service::user_service::UserService;

#[derive(Deserialize)]
pub struct EmailQuery {
    pub email: String,
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/internal")
            .route("/users/by-email", web::get().to(get_by_email)),
    );
}

async fn get_by_email(
    svc: web::Data<UserService>,
    query: web::Query<EmailQuery>,
) -> Result<HttpResponse, ServiceError> {
    match svc.find_by_email(&query.email).await? {
        Some(user) => Ok(HttpResponse::Ok().json(user_to_auth_response(&user))),
        None => Err(ServiceError::NotFound(format!(
            "User with email {} not found",
            query.email
        ))),
    }
}
