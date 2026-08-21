//! Handler for the `upload_product_images` job kind.
//!
//! Payload shape: `{ "product_id": "<uuid>" }`
//! Blobs: raw image bytes paired with content-types in `blob_types`.

use sqlx::PgPool;
use tracing::{debug, info};
use uuid::Uuid;

use crate::domain::product_photo::UploadFile;
use crate::service::product_photo_service::ProductPhotoService;

use super::Job;

/// Parse a `Job` row into a `(product_id, files)` pair. Pure function —
/// extracted from `handle` so it can be unit-tested without a real
/// `ProductPhotoService` / database.
pub fn parse(job: &Job) -> Result<(Uuid, Vec<UploadFile>), String> {
    debug!(job_id = job.id, kind = %job.kind, "jobs:parse upload_product_images");
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

    if blobs.len() != blob_types.len() {
        return Err(format!(
            "blobs/blob_types length mismatch: {} vs {}",
            blobs.len(),
            blob_types.len()
        ));
    }

    if blobs.is_empty() {
        return Err("no files in job".to_string());
    }

    let files = blobs
        .iter()
        .zip(blob_types.iter())
        .map(|(data, content_type)| UploadFile {
            data: data.clone(),
            content_type: content_type.clone(),
        })
        .collect();

    debug!(
        job_id = job.id,
        %product_id,
        files = blobs.len(),
        "jobs:parse upload_product_images done"
    );
    Ok((product_id, files))
}

pub async fn handle(
    job: &Job,
    photo_service: &ProductPhotoService,
    pool: &PgPool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    debug!(job_id = job.id, "jobs:handle upload_product_images start");
    let (product_id, files) = parse(job)?;
    let new_indices = photo_service
        .upload_images(product_id, files)
        .await
        .map_err(|e| format!("upload_images failed: {e}"))?;
    let uploaded = new_indices.len();
    info!(
        job_id = job.id,
        %product_id,
        uploaded,
        "jobs:handle upload_product_images success"
    );

    // Enqueue per-image embedding for exactly the indices we just wrote.
    // Done in a short transaction so a worker crash between the upload and
    // the enqueue doesn't leave embeddings behind: the backfill job
    // (`SERVICE_MODE=reindex-images`) recovers from that case.
    if uploaded > 0 {
        let indices_i32: Vec<i32> = new_indices.iter().map(|i| *i as i32).collect();
        let mut tx = pool
            .begin()
            .await
            .map_err(|e| format!("begin tx for embed enqueue: {e}"))?;
        super::enqueue_embed_product_images(&mut tx, product_id, &indices_i32)
            .await
            .map_err(|e| format!("enqueue embed_product_images: {e}"))?;
        tx.commit()
            .await
            .map_err(|e| format!("commit tx for embed enqueue: {e}"))?;
    }
    Ok(())
}
