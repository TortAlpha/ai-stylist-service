use actix_multipart::Multipart;
use actix_web::{HttpResponse, web};
use tracing::{debug, info};
use uuid::Uuid;

use crate::domain::error::ServiceError;
use crate::domain::response_dto::product::{
    AdminProductDTO, ProductFilterOptions, ProductPreviewResponse,
};
use crate::domain::response_dto::product_details::AvailableSizesResponse;
use crate::domain::utils::api_response::ApiResponse;
use crate::domain::utils::pagination::PaginatedResponse;
use crate::domain::utils::query::{
    AvailableSizesQuery, DeleteProductQuery, FilterOptionsQuery, ProductListQuery,
};
use crate::extractors::auth::AdminUser;
use crate::extractors::product::extract_product_multipart;
use crate::handler::product_photo_handler;
use crate::service::product::ProductService;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/admin/products")
            // CRUD
            .route("", web::post().to(create_product_admin))
            .route("", web::get().to(get_product_previews_by_query_admin))
            // Filters must be registered before /{id}, otherwise static paths can be parsed as ids.
            .route("/available-sizes", web::get().to(available_sizes))
            .route("/filter-options", web::get().to(filter_options))
            .route("/{id}", web::get().to(get_product_by_id_admin))
            .route("/{id}", web::put().to(update_product_by_id_admin))
            .route("/{id}", web::delete().to(delete_product_by_id_admin))
            // Photos
            .route(
                "/{id}/images",
                web::post().to(product_photo_handler::upload_images),
            )
            .route(
                "/{id}/images/{image_id}",
                web::delete().to(product_photo_handler::delete_image),
            )
            .route(
                "/{id}/preview",
                web::post().to(product_photo_handler::upload_preview),
            )
            .route(
                "/{id}/preview",
                web::delete().to(product_photo_handler::delete_preview),
            ),
    );
}

#[utoipa::path(
    get,
    path = "/api/admin/products",
    tag = "Products",
    params(ProductListQuery),
    responses(
        (status = 200, description = "Paginated product previews", body = PaginatedResponse<ProductPreviewResponse>),
    ),
    security(("bearer" = []))
)]
pub async fn get_product_previews_by_query_admin(
    _admin: AdminUser,
    query: web::Query<ProductListQuery>,
    service: web::Data<ProductService>,
) -> Result<HttpResponse, ServiceError> {
    debug!(
        page = ?query.page,
        per_page = ?query.per_page,
        brand_id = ?query.brand_id,
        category_id = ?query.category_id,
        "list products request"
    );
    let result = service.get_product_previews_by_query_admin(&query).await?;
    info!(
        items = result.items.len(),
        total = result.total,
        "listed products"
    );
    Ok(HttpResponse::Ok().json(ApiResponse::ok(result)))
}

#[utoipa::path(
    get,
    path = "/api/admin/products/{id}",
    tag = "Products",
    params(("id" = Uuid, Path, description = "Product UUID")),
    responses(
        (status = 200, description = "Full product details", body = AdminProductDTO),
        (status = 404, description = "Product not found"),
    ),
    security(("bearer" = []))
)]
pub async fn get_product_by_id_admin(
    _admin: AdminUser,
    id: web::Path<Uuid>,
    service: web::Data<ProductService>,
) -> Result<HttpResponse, ServiceError> {
    let product_id = id.into_inner();
    debug!(%product_id, "get product by id request");
    let product = service.get_product_by_id_admin(product_id).await?;
    info!(%product_id, "product loaded");
    Ok(HttpResponse::Ok().json(ApiResponse::ok(product)))
}

#[utoipa::path(
    post,
    path = "/api/admin/products",
    tag = "Products",
    request_body(
        content_type = "multipart/form-data",
        description = "Multipart: `metadata` (JSON — CreateProductRequest), `preview` (image file), `images` (image files)"
    ),
    responses(
        (status = 201, description = "Product created", body = AdminProductDTO),
        (status = 400, description = "Validation error"),
    ),
    security(("bearer" = []))
)]
pub async fn create_product_admin(
    _admin: AdminUser,
    service: web::Data<ProductService>,
    mut payload: Multipart,
) -> Result<HttpResponse, ServiceError> {
    let mp = extract_product_multipart(&mut payload).await?;
    debug!(
        has_preview = mp.preview.is_some(),
        images_count = mp.images.len(),
        "create product request"
    );
    let product = service
        .create_product_admin(&mp.metadata, mp.preview, mp.images)
        .await?;
    info!(product_id = %product.id, "product created");
    Ok(HttpResponse::Created().json(ApiResponse::ok(product)))
}

#[utoipa::path(
    put,
    path = "/api/admin/products/{id}",
    tag = "Products",
    params(("id" = Uuid, Path, description = "Product UUID")),
    request_body(
        content_type = "multipart/form-data",
        description = "Multipart: `metadata` (JSON — UpdateProductRequest), `preview` (image file), `images` (image files)"
    ),
    responses(
        (status = 200, description = "Product updated", body = AdminProductDTO),
        (status = 404, description = "Product not found"),
        (status = 409, description = "Version conflict"),
    ),
    security(("bearer" = []))
)]
pub async fn update_product_by_id_admin(
    _admin: AdminUser,
    id: web::Path<Uuid>,
    service: web::Data<ProductService>,
    mut payload: Multipart,
) -> Result<HttpResponse, ServiceError> {
    let product_id = id.into_inner();
    let mp = extract_product_multipart(&mut payload).await?;
    debug!(
        %product_id,
        has_preview = mp.preview.is_some(),
        images_count = mp.images.len(),
        "update product request"
    );
    let product = service
        .update(product_id, &mp.metadata, mp.preview, mp.images)
        .await?;
    info!(product_id = %product.id, version = product.version, "product updated");
    Ok(HttpResponse::Ok().json(ApiResponse::ok(product)))
}

#[utoipa::path(
    delete,
    path = "/api/admin/products/{id}",
    tag = "Products",
    params(
        ("id" = Uuid, Path, description = "Product UUID"),
        DeleteProductQuery,
    ),
    responses(
        (status = 200, description = "Product soft-deleted"),
        (status = 404, description = "Product not found"),
        (status = 409, description = "Version conflict"),
    ),
    security(("bearer" = []))
)]
pub async fn delete_product_by_id_admin(
    _admin: AdminUser,
    id: web::Path<Uuid>,
    query: web::Query<DeleteProductQuery>,
    service: web::Data<ProductService>,
) -> Result<HttpResponse, ServiceError> {
    let product_id = id.into_inner();
    debug!(
        %product_id,
        expected_version = query.expected_version,
        "delete product request"
    );
    service
        .soft_delete(product_id, query.expected_version)
        .await?;
    info!(%product_id, "product soft-deleted");
    let deleted = ();
    Ok(HttpResponse::Ok().json(ApiResponse::ok(deleted)))
}

#[utoipa::path(
    get,
    path = "/api/admin/products/filter-options",
    tag = "Products",
    params(FilterOptionsQuery),
    responses(
        (status = 200, description = "Available filter options", body = ProductFilterOptions),
    ),
    security(("bearer" = []))
)]
pub async fn filter_options(
    query: web::Query<FilterOptionsQuery>,
    service: web::Data<ProductService>,
) -> Result<HttpResponse, ServiceError> {
    debug!("filter options request");
    let options = service.filter_options(&query).await?;
    info!(
        colors = options.colors.len(),
        conditions = options.conditions.len(),
        materials = options.materials.len(),
        "filter options computed"
    );
    Ok(HttpResponse::Ok().json(ApiResponse::ok(options)))
}

#[utoipa::path(
    get,
    path = "/api/admin/products/available-sizes",
    tag = "Products",
    params(AvailableSizesQuery),
    responses(
        (status = 200, description = "Available size values for filters", body = AvailableSizesResponse),
    ),
    security(("bearer" = []))
)]
pub async fn available_sizes(
    query: web::Query<AvailableSizesQuery>,
    service: web::Data<ProductService>,
) -> Result<HttpResponse, ServiceError> {
    debug!(
        category_id = ?query.category_id,
        size_group = ?query.size_group,
        "available sizes request"
    );
    let sizes = service.available_sizes(&query).await?;
    info!(
        size_group = %sizes.size_group,
        values = sizes.values.len(),
        values2 = sizes.values2.len(),
        "available sizes computed"
    );
    Ok(HttpResponse::Ok().json(ApiResponse::ok(sizes)))
}
