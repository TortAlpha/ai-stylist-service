use actix_web::{HttpResponse, web};

use crate::domain::error::ServiceError;
use crate::domain::request_dto::tag::CreateTagRequest;
use crate::domain::response_dto::tag::TagResponse;
use crate::domain::utils::api_response::ApiResponse;
use crate::extractors::auth::AdminUser;
use crate::service::tag_service::TagService;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/tags")
            .route("/styles", web::get().to(list_style_tags))
            .route("/vibes", web::get().to(list_vibe_tags))
            .route("/seasons", web::get().to(list_seasons)),
    )
    .service(
        web::scope("/admin/tags")
            .route("/styles", web::get().to(list_style_tags))
            .route("/styles", web::post().to(create_style_tag))
            .route("/vibes", web::get().to(list_vibe_tags))
            .route("/vibes", web::post().to(create_vibe_tag))
            .route("/seasons", web::get().to(list_seasons))
            .route("/seasons", web::post().to(create_season))
            .route("/styles/{id}", web::delete().to(delete_style_tag))
            .route("/vibes/{id}", web::delete().to(delete_vibe_tag))
            .route("/seasons/{id}", web::delete().to(delete_season)),
    );
}

#[utoipa::path(
    get,
    path = "/api/tags/styles",
    tag = "Tags",
    responses((status = 200, description = "All style tags", body = Vec<TagResponse>))
)]
pub async fn list_style_tags(service: web::Data<TagService>) -> Result<HttpResponse, ServiceError> {
    let items = service.list_style_tags().await?;
    Ok(HttpResponse::Ok().json(ApiResponse::ok(items)))
}

#[utoipa::path(
    post,
    path = "/api/admin/tags/styles",
    tag = "Tags",
    request_body = CreateTagRequest,
    responses((status = 201, description = "Style tag created", body = TagResponse)),
    security(("bearer" = []))
)]
pub async fn create_style_tag(
    _admin: AdminUser,
    service: web::Data<TagService>,
    payload: web::Json<CreateTagRequest>,
) -> Result<HttpResponse, ServiceError> {
    let tag = service.create_style_tag(&payload.name).await?;
    Ok(HttpResponse::Created().json(ApiResponse::ok(tag)))
}

#[utoipa::path(
    get,
    path = "/api/tags/vibes",
    tag = "Tags",
    responses((status = 200, description = "All vibe tags", body = Vec<TagResponse>))
)]
pub async fn list_vibe_tags(service: web::Data<TagService>) -> Result<HttpResponse, ServiceError> {
    let items = service.list_vibe_tags().await?;
    Ok(HttpResponse::Ok().json(ApiResponse::ok(items)))
}

#[utoipa::path(
    post,
    path = "/api/admin/tags/vibes",
    tag = "Tags",
    request_body = CreateTagRequest,
    responses((status = 201, description = "Vibe tag created", body = TagResponse)),
    security(("bearer" = []))
)]
pub async fn create_vibe_tag(
    _admin: AdminUser,
    service: web::Data<TagService>,
    payload: web::Json<CreateTagRequest>,
) -> Result<HttpResponse, ServiceError> {
    let tag = service.create_vibe_tag(&payload.name).await?;
    Ok(HttpResponse::Created().json(ApiResponse::ok(tag)))
}

#[utoipa::path(
    get,
    path = "/api/tags/seasons",
    tag = "Tags",
    responses((status = 200, description = "All seasons", body = Vec<TagResponse>))
)]
pub async fn list_seasons(service: web::Data<TagService>) -> Result<HttpResponse, ServiceError> {
    let items = service.list_seasons().await?;
    Ok(HttpResponse::Ok().json(ApiResponse::ok(items)))
}

#[utoipa::path(
    post,
    path = "/api/admin/tags/seasons",
    tag = "Tags",
    request_body = CreateTagRequest,
    responses((status = 201, description = "Season created", body = TagResponse)),
    security(("bearer" = []))
)]
pub async fn create_season(
    _admin: AdminUser,
    service: web::Data<TagService>,
    payload: web::Json<CreateTagRequest>,
) -> Result<HttpResponse, ServiceError> {
    let tag = service.create_season(&payload.name).await?;
    Ok(HttpResponse::Created().json(ApiResponse::ok(tag)))
}

#[utoipa::path(
    delete,
    path = "/api/admin/tags/styles/{id}",
    tag = "Tags",
    params(("id" = i32, Path, description = "Style tag ID")),
    responses((status = 204, description = "Style tag deleted")),
    security(("bearer" = []))
)]
pub async fn delete_style_tag(
    _admin: AdminUser,
    id: web::Path<i32>,
    service: web::Data<TagService>,
) -> Result<HttpResponse, ServiceError> {
    service.delete_style_tag(id.into_inner()).await?;
    Ok(HttpResponse::NoContent().finish())
}

#[utoipa::path(
    delete,
    path = "/api/admin/tags/vibes/{id}",
    tag = "Tags",
    params(("id" = i32, Path, description = "Vibe tag ID")),
    responses((status = 204, description = "Vibe tag deleted")),
    security(("bearer" = []))
)]
pub async fn delete_vibe_tag(
    _admin: AdminUser,
    id: web::Path<i32>,
    service: web::Data<TagService>,
) -> Result<HttpResponse, ServiceError> {
    service.delete_vibe_tag(id.into_inner()).await?;
    Ok(HttpResponse::NoContent().finish())
}

#[utoipa::path(
    delete,
    path = "/api/admin/tags/seasons/{id}",
    tag = "Tags",
    params(("id" = i32, Path, description = "Season ID")),
    responses((status = 204, description = "Season deleted")),
    security(("bearer" = []))
)]
pub async fn delete_season(
    _admin: AdminUser,
    id: web::Path<i32>,
    service: web::Data<TagService>,
) -> Result<HttpResponse, ServiceError> {
    service.delete_season(id.into_inner()).await?;
    Ok(HttpResponse::NoContent().finish())
}
