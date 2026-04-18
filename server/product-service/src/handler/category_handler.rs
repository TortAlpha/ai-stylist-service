use actix_web::{HttpResponse, web};

use crate::domain::error::ServiceError;
use crate::domain::request_dto::category::{CreateCategoryRequest, UpdateCategoryRequest};
use crate::domain::response_dto::category::CategoryFullResponse;
use crate::domain::utils::api_response::ApiResponse;
use crate::domain::utils::query::CategoryListQuery;
use crate::extractors::auth::AdminUser;
use crate::service::category_service::CategoryService;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/categories").route("", web::get().to(list_categories)))
        .service(
            web::scope("/admin/categories")
                .route("", web::get().to(list_categories_admin))
                .route("", web::post().to(create_category))
                .route("/{id}", web::get().to(get_category_by_id))
                .route("/{id}", web::put().to(update_category))
                .route("/{id}", web::delete().to(delete_category)),
        );
}

#[utoipa::path(
    get,
    path = "/api/categories",
    tag = "Categories",
    params(CategoryListQuery),
    responses(
        (status = 200, description = "Category list", body = Vec<CategoryFullResponse>),
    )
)]
pub async fn list_categories(
    query: web::Query<CategoryListQuery>,
    service: web::Data<CategoryService>,
) -> Result<HttpResponse, ServiceError> {
    let items = service.list(&query).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::ok(items)))
}

#[utoipa::path(
    get,
    path = "/api/admin/categories",
    tag = "Categories",
    params(CategoryListQuery),
    responses(
        (status = 200, description = "Category list (admin)", body = Vec<CategoryFullResponse>),
    ),
    security(("bearer" = []))
)]
pub async fn list_categories_admin(
    _admin: AdminUser,
    query: web::Query<CategoryListQuery>,
    service: web::Data<CategoryService>,
) -> Result<HttpResponse, ServiceError> {
    let items = service.list(&query).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::ok(items)))
}

#[utoipa::path(
    get,
    path = "/api/admin/categories/{id}",
    tag = "Categories",
    params(("id" = i32, Path, description = "Category ID")),
    responses(
        (status = 200, description = "Category details", body = CategoryFullResponse),
        (status = 404, description = "Category not found"),
    ),
    security(("bearer" = []))
)]
pub async fn get_category_by_id(
    _admin: AdminUser,
    id: web::Path<i32>,
    service: web::Data<CategoryService>,
) -> Result<HttpResponse, ServiceError> {
    let category = service.get_by_id(id.into_inner()).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::ok(category)))
}

#[utoipa::path(
    post,
    path = "/api/admin/categories",
    tag = "Categories",
    request_body = CreateCategoryRequest,
    responses(
        (status = 201, description = "Category created", body = CategoryFullResponse),
    ),
    security(("bearer" = []))
)]
pub async fn create_category(
    _admin: AdminUser,
    service: web::Data<CategoryService>,
    payload: web::Json<CreateCategoryRequest>,
) -> Result<HttpResponse, ServiceError> {
    let category = service.create(&payload).await?;
    Ok(HttpResponse::Created().json(ApiResponse::ok(category)))
}

#[utoipa::path(
    put,
    path = "/api/admin/categories/{id}",
    tag = "Categories",
    params(("id" = i32, Path, description = "Category ID")),
    request_body = UpdateCategoryRequest,
    responses(
        (status = 200, description = "Category updated", body = CategoryFullResponse),
        (status = 404, description = "Category not found"),
    ),
    security(("bearer" = []))
)]
pub async fn update_category(
    _admin: AdminUser,
    id: web::Path<i32>,
    service: web::Data<CategoryService>,
    payload: web::Json<UpdateCategoryRequest>,
) -> Result<HttpResponse, ServiceError> {
    let category = service.update_by_id(id.into_inner(), &payload).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::ok(category)))
}

#[utoipa::path(
    delete,
    path = "/api/admin/categories/{id}",
    tag = "Categories",
    params(("id" = i32, Path, description = "Category ID")),
    responses(
        (status = 204, description = "Category deleted"),
        (status = 404, description = "Category not found"),
    ),
    security(("bearer" = []))
)]
pub async fn delete_category(
    _admin: AdminUser,
    id: web::Path<i32>,
    service: web::Data<CategoryService>,
) -> Result<HttpResponse, ServiceError> {
    service.delete_by_id(id.into_inner()).await?;
    Ok(HttpResponse::NoContent().finish())
}
