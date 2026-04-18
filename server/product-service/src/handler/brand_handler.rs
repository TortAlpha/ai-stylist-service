use actix_web::{HttpResponse, web};

use crate::domain::error::ServiceError;
use crate::domain::request_dto::brand::{CreateBrandRequest, UpdateBrandRequest};
use crate::domain::response_dto::brand::BrandResponse;
use crate::domain::utils::api_response::ApiResponse;
use crate::domain::utils::pagination::PaginatedResponse;
use crate::domain::utils::query::BrandSearchQuery;
use crate::extractors::auth::AdminUser;
use crate::service::brand_service::BrandService;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/brands").route("", web::get().to(list_brands)))
        .service(
            web::scope("/admin/brands")
                .route("", web::get().to(search_brands))
                .route("", web::post().to(create_brand))
                .route("/{id}", web::get().to(get_brand_by_id))
                .route("/{id}", web::put().to(update_brand))
                .route("/{id}", web::delete().to(delete_brand)),
        );
}

#[utoipa::path(
    get,
    path = "/api/brands",
    tag = "Brands",
    responses(
        (status = 200, description = "All brands", body = Vec<BrandResponse>),
    )
)]
pub async fn list_brands(service: web::Data<BrandService>) -> Result<HttpResponse, ServiceError> {
    let items = service.list_all().await?;
    Ok(HttpResponse::Ok().json(ApiResponse::ok(items)))
}

#[utoipa::path(
    get,
    path = "/api/admin/brands",
    tag = "Brands",
    params(BrandSearchQuery),
    responses(
        (status = 200, description = "Paginated brand search", body = PaginatedResponse<BrandResponse>),
    ),
    security(("bearer" = []))
)]
pub async fn search_brands(
    _admin: AdminUser,
    query: web::Query<BrandSearchQuery>,
    service: web::Data<BrandService>,
) -> Result<HttpResponse, ServiceError> {
    let result = service.search(&query).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::ok(result)))
}

#[utoipa::path(
    get,
    path = "/api/admin/brands/{id}",
    tag = "Brands",
    params(("id" = i32, Path, description = "Brand ID")),
    responses(
        (status = 200, description = "Brand details", body = BrandResponse),
        (status = 404, description = "Brand not found"),
    ),
    security(("bearer" = []))
)]
pub async fn get_brand_by_id(
    _admin: AdminUser,
    id: web::Path<i32>,
    service: web::Data<BrandService>,
) -> Result<HttpResponse, ServiceError> {
    let brand = service.get_by_id(id.into_inner()).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::ok(brand)))
}

#[utoipa::path(
    post,
    path = "/api/admin/brands",
    tag = "Brands",
    request_body = CreateBrandRequest,
    responses(
        (status = 201, description = "Brand created", body = BrandResponse),
        (status = 409, description = "Brand code conflict"),
    ),
    security(("bearer" = []))
)]
pub async fn create_brand(
    _admin: AdminUser,
    service: web::Data<BrandService>,
    payload: web::Json<CreateBrandRequest>,
) -> Result<HttpResponse, ServiceError> {
    let brand = service.create(&payload).await?;
    Ok(HttpResponse::Created().json(ApiResponse::ok(brand)))
}

#[utoipa::path(
    put,
    path = "/api/admin/brands/{id}",
    tag = "Brands",
    params(("id" = i32, Path, description = "Brand ID")),
    request_body = UpdateBrandRequest,
    responses(
        (status = 200, description = "Brand updated", body = BrandResponse),
        (status = 404, description = "Brand not found"),
    ),
    security(("bearer" = []))
)]
pub async fn update_brand(
    _admin: AdminUser,
    id: web::Path<i32>,
    service: web::Data<BrandService>,
    payload: web::Json<UpdateBrandRequest>,
) -> Result<HttpResponse, ServiceError> {
    let brand = service.update_by_id(id.into_inner(), &payload).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::ok(brand)))
}

#[utoipa::path(
    delete,
    path = "/api/admin/brands/{id}",
    tag = "Brands",
    params(("id" = i32, Path, description = "Brand ID")),
    responses(
        (status = 204, description = "Brand deleted"),
        (status = 404, description = "Brand not found"),
    ),
    security(("bearer" = []))
)]
pub async fn delete_brand(
    _admin: AdminUser,
    id: web::Path<i32>,
    service: web::Data<BrandService>,
) -> Result<HttpResponse, ServiceError> {
    service.delete_by_id(id.into_inner()).await?;
    Ok(HttpResponse::NoContent().finish())
}
