use actix_multipart::Multipart;
use actix_web::{HttpResponse, web};
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::domain::error::ServiceError;
use crate::domain::utils::api_response::ApiResponse;
use crate::extractors::auth::AdminUser;
use crate::extractors::product_photo::extract_photo_files;
use crate::service::product_photo_service::ProductPhotoService;

#[utoipa::path(
    post,
    path = "/api/admin/products/{id}/images",
    tag = "Product Photos",
    params(("id" = Uuid, Path, description = "Product UUID")),
    request_body(content_type = "multipart/form-data", description = "Image files to upload"),
    responses(
        (status = 202, description = "Images upload queued"),
    ),
    security(("bearer" = []))
)]
pub async fn upload_images(
    _admin: AdminUser,
    id: web::Path<Uuid>,
    service: web::Data<ProductPhotoService>,
    mut payload: Multipart,
) -> Result<HttpResponse, ServiceError> {
    let product_id = id.into_inner();
    debug!(%product_id, "upload images request");
    let files = extract_photo_files(&mut payload).await?;
    debug!(%product_id, files = files.len(), "images extracted from multipart");
    let job_id = service.enqueue_images_upload(product_id, files).await?;
    info!(%product_id, %job_id, "images upload queued");

    Ok(
        HttpResponse::Accepted().json(ApiResponse::ok(serde_json::json!({
            "queued": true,
            "job_id": job_id,
        }))),
    )
}

#[utoipa::path(
    post,
    path = "/api/admin/products/{id}/preview",
    tag = "Product Photos",
    params(("id" = Uuid, Path, description = "Product UUID")),
    request_body(content_type = "multipart/form-data", description = "Single preview image file"),
    responses(
        (status = 202, description = "Preview upload queued"),
        (status = 400, description = "No file provided"),
    ),
    security(("bearer" = []))
)]
pub async fn upload_preview(
    _admin: AdminUser,
    id: web::Path<Uuid>,
    service: web::Data<ProductPhotoService>,
    mut payload: Multipart,
) -> Result<HttpResponse, ServiceError> {
    let product_id = id.into_inner();
    debug!(%product_id, "upload preview request");
    let mut files = extract_photo_files(&mut payload).await?;
    if files.is_empty() {
        warn!(%product_id, "preview upload request contains no files");
        return Err(ServiceError::BadRequest("no file provided".into()));
    }
    if files.len() > 1 {
        warn!(
            %product_id,
            files = files.len(),
            "preview upload received multiple files; using first one"
        );
    }

    let file = files.swap_remove(0);
    let job_id = service.enqueue_preview_upload(product_id, file).await?;
    info!(%product_id, %job_id, "preview upload queued");

    Ok(
        HttpResponse::Accepted().json(ApiResponse::ok(serde_json::json!({
            "queued": true,
            "job_id": job_id,
        }))),
    )
}
