use actix_web::{HttpResponse, web};

use crate::domain::error::ServiceError;
use crate::domain::purchase_location::PurchaseLocation;
use crate::domain::request_dto::purchase_location::{
    CreatePurchaseLocationRequest, UpdatePurchaseLocationRequest,
};
use crate::domain::utils::api_response::ApiResponse;
use crate::extractors::auth::AdminUser;
use crate::service::purchase_location_service::PurchaseLocationService;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/admin/purchase-locations")
            .route("", web::get().to(list_purchase_locations))
            .route("", web::post().to(create_purchase_location))
            .route("/{id}", web::get().to(get_purchase_location_by_id))
            .route("/{id}", web::put().to(update_purchase_location_by_id))
            .route("/{id}", web::delete().to(delete_purchase_location_by_id)),
    );
}

#[utoipa::path(
    get,
    path = "/api/admin/purchase-locations",
    tag = "Purchase Locations",
    responses(
        (status = 200, description = "All purchase locations", body = Vec<PurchaseLocation>),
    ),
    security(("bearer" = []))
)]
pub async fn list_purchase_locations(
    _admin: AdminUser,
    service: web::Data<PurchaseLocationService>,
) -> Result<HttpResponse, ServiceError> {
    let items = service.list_all().await?;
    Ok(HttpResponse::Ok().json(ApiResponse::ok(items)))
}

#[utoipa::path(
    get,
    path = "/api/admin/purchase-locations/{id}",
    tag = "Purchase Locations",
    params(("id" = i32, Path, description = "Purchase location ID")),
    responses(
        (status = 200, description = "Purchase location details", body = PurchaseLocation),
        (status = 404, description = "Not found"),
    ),
    security(("bearer" = []))
)]
pub async fn get_purchase_location_by_id(
    _admin: AdminUser,
    id: web::Path<i32>,
    service: web::Data<PurchaseLocationService>,
) -> Result<HttpResponse, ServiceError> {
    let item = service.get_by_id(id.into_inner()).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::ok(item)))
}

#[utoipa::path(
    post,
    path = "/api/admin/purchase-locations",
    tag = "Purchase Locations",
    request_body = CreatePurchaseLocationRequest,
    responses(
        (status = 201, description = "Purchase location created", body = PurchaseLocation),
    ),
    security(("bearer" = []))
)]
pub async fn create_purchase_location(
    _admin: AdminUser,
    service: web::Data<PurchaseLocationService>,
    payload: web::Json<CreatePurchaseLocationRequest>,
) -> Result<HttpResponse, ServiceError> {
    let created = service.create(&payload.name).await?;
    Ok(HttpResponse::Created().json(ApiResponse::ok(created)))
}

#[utoipa::path(
    put,
    path = "/api/admin/purchase-locations/{id}",
    tag = "Purchase Locations",
    params(("id" = i32, Path, description = "Purchase location ID")),
    request_body = UpdatePurchaseLocationRequest,
    responses(
        (status = 200, description = "Purchase location updated", body = PurchaseLocation),
        (status = 404, description = "Not found"),
    ),
    security(("bearer" = []))
)]
pub async fn update_purchase_location_by_id(
    _admin: AdminUser,
    id: web::Path<i32>,
    service: web::Data<PurchaseLocationService>,
    payload: web::Json<UpdatePurchaseLocationRequest>,
) -> Result<HttpResponse, ServiceError> {
    let updated = service.update_by_id(id.into_inner(), &payload.name).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::ok(updated)))
}

#[utoipa::path(
    delete,
    path = "/api/admin/purchase-locations/{id}",
    tag = "Purchase Locations",
    params(("id" = i32, Path, description = "Purchase location ID")),
    responses(
        (status = 204, description = "Purchase location deleted"),
        (status = 404, description = "Not found"),
    ),
    security(("bearer" = []))
)]
pub async fn delete_purchase_location_by_id(
    _admin: AdminUser,
    id: web::Path<i32>,
    service: web::Data<PurchaseLocationService>,
) -> Result<HttpResponse, ServiceError> {
    service.delete_by_id(id.into_inner()).await?;
    Ok(HttpResponse::NoContent().finish())
}
