//! Handler for the `upload_product_preview` job kind.
//!
//! Payload shape: `{ "product_id": "<uuid>" }`
//! Blobs: exactly one image blob paired with its content-type.

use tracing::{debug, info};
use uuid::Uuid;

use crate::domain::product_photo::UploadFile;
use crate::service::product_photo_service::ProductPhotoService;

use super::Job;

pub fn parse(job: &Job) -> Result<(Uuid, UploadFile), String> {
    debug!(job_id = job.id, kind = %job.kind, "jobs:parse upload_product_preview");
    let product_id_str = job
        .payload
        .get("product_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "payload.product_id missing or not a string".to_string())?;
    let product_id = Uuid::parse_str(product_id_str)
        .map_err(|e| format!("payload.product_id invalid uuid: {e}"))?;

    let blobs = job
        .blobs
        .as_ref()
        .ok_or_else(|| "blobs column is null".to_string())?;
    let blob_types = job
        .blob_types
        .as_ref()
        .ok_or_else(|| "blob_types column is null".to_string())?;

    if blobs.len() != 1 || blob_types.len() != 1 {
        return Err(format!(
            "preview job expects exactly 1 blob, got {} blobs and {} types",
            blobs.len(),
            blob_types.len()
        ));
    }

    let file = UploadFile {
        data: blobs[0].clone(),
        content_type: blob_types[0].clone(),
    };

    debug!(
        job_id = job.id,
        %product_id,
        content_type = %file.content_type,
        size_bytes = file.data.len(),
        "jobs:parse upload_product_preview done"
    );
    Ok((product_id, file))
}

pub async fn handle(
    job: &Job,
    photo_service: &ProductPhotoService,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    debug!(job_id = job.id, "jobs:handle upload_product_preview start");
    let (product_id, file) = parse(job)?;
    let key = photo_service
        .upload_preview(product_id, file)
        .await
        .map_err(|e| format!("upload_preview failed: {e}"))?;
    info!(
        job_id = job.id,
        %product_id,
        %key,
        "jobs:handle upload_product_preview success"
    );
    Ok(())
}
