use actix_web::{HttpResponse, web};
use tracing::{debug, info};
use uuid::Uuid;
use validator::Validate;

use crate::domain::error::ServiceError;
use crate::domain::mappers::{address_to_response, user_to_response};
use crate::domain::request_dto::*;
use crate::service::user_service::UserService;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/users")
            .route("", web::post().to(register))
            .route("/{id}", web::get().to(get_user))
            .route("/{id}", web::patch().to(update_user))
            .route("/{id}", web::delete().to(delete_user))
            .route("/{id}/addresses", web::get().to(get_addresses))
            .route("/{id}/addresses", web::post().to(create_address))
            .route(
                "/{id}/addresses/{address_id}",
                web::patch().to(update_address),
            )
            .route(
                "/{id}/addresses/{address_id}",
                web::delete().to(delete_address),
            ),
    );
}

async fn register(
    svc: web::Data<UserService>,
    body: web::Json<CreateUserRequest>,
) -> Result<HttpResponse, ServiceError> {
    debug!(email = %body.email, "user register request");
    body.validate()
        .map_err(|e| ServiceError::BadRequest(e.to_string()))?;
    let user = svc.register(&body).await?;
    info!(user_id = %user.id, "user registered");
    Ok(HttpResponse::Created().json(user_to_response(&user)))
}

async fn get_user(
    svc: web::Data<UserService>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ServiceError> {
    let user_id = path.into_inner();
    debug!(%user_id, "get user request");
    let user = svc.get_by_id(user_id).await?;
    info!(%user_id, "user loaded");
    Ok(HttpResponse::Ok().json(user_to_response(&user)))
}

async fn update_user(
    svc: web::Data<UserService>,
    path: web::Path<Uuid>,
    body: web::Json<UpdateUserRequest>,
) -> Result<HttpResponse, ServiceError> {
    let user_id = path.into_inner();
    debug!(%user_id, "update user request");
    body.validate()
        .map_err(|e| ServiceError::BadRequest(e.to_string()))?;
    let user = svc.update(user_id, &body).await?;
    info!(user_id = %user.id, "user updated");
    Ok(HttpResponse::Ok().json(user_to_response(&user)))
}

async fn delete_user(
    svc: web::Data<UserService>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ServiceError> {
    let user_id = path.into_inner();
    debug!(%user_id, "delete user request");
    svc.soft_delete(user_id).await?;
    info!(%user_id, "user soft-deleted");
    Ok(HttpResponse::NoContent().finish())
}

async fn get_addresses(
    svc: web::Data<UserService>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, ServiceError> {
    let user_id = path.into_inner();
    debug!(%user_id, "list addresses request");
    let addresses = svc.get_addresses(user_id).await?;
    info!(%user_id, count = addresses.len(), "addresses loaded");
    let resp: Vec<_> = addresses.iter().map(address_to_response).collect();
    Ok(HttpResponse::Ok().json(resp))
}

async fn create_address(
    svc: web::Data<UserService>,
    path: web::Path<Uuid>,
    body: web::Json<CreateAddressRequest>,
) -> Result<HttpResponse, ServiceError> {
    let user_id = path.into_inner();
    debug!(%user_id, "create address request");
    body.validate()
        .map_err(|e| ServiceError::BadRequest(e.to_string()))?;
    let address = svc.create_address(user_id, &body).await?;
    info!(user_id = %address.owner_id, address_id = %address.id, "address created");
    Ok(HttpResponse::Created().json(address_to_response(&address)))
}

async fn update_address(
    svc: web::Data<UserService>,
    path: web::Path<(Uuid, Uuid)>,
    body: web::Json<UpdateAddressRequest>,
) -> Result<HttpResponse, ServiceError> {
    let (user_id, address_id) = path.into_inner();
    debug!(%user_id, %address_id, "update address request");
    body.validate()
        .map_err(|e| ServiceError::BadRequest(e.to_string()))?;
    let address = svc.update_address(address_id, &body).await?;
    info!(user_id = %address.owner_id, address_id = %address.id, "address updated");
    Ok(HttpResponse::Ok().json(address_to_response(&address)))
}

async fn delete_address(
    svc: web::Data<UserService>,
    path: web::Path<(Uuid, Uuid)>,
) -> Result<HttpResponse, ServiceError> {
    let (user_id, address_id) = path.into_inner();
    debug!(%user_id, %address_id, "delete address request");
    svc.delete_address(address_id).await?;
    info!(%user_id, %address_id, "address deleted");
    Ok(HttpResponse::NoContent().finish())
}
