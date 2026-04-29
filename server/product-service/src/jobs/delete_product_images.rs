//! Handler for the `delete_product_images` job kind.
//!
//! Payload shape: `{ "product_id": "<uuid>" }`

use tracing::{debug, info};
use uuid::Uuid;

use crate::service::product_photo_service::ProductPhotoService;

use super::Job;

pub fn parse(job: &Job) -> Result<Uuid, String> {
    debug!(job_id = job.id, kind = %job.kind, "jobs:parse delete_product_images");
    let product_id_str = job
        .payload
        .get("product_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "payload.product_id missing or not a string".to_string())?;
    let product_id = Uuid::parse_str(product_id_str)
        .map_err(|e| format!("payload.product_id invalid uuid: {e}"))?;

    debug!(
        job_id = job.id,
        %product_id,
        "jobs:parse delete_product_images done"
    );
    Ok(product_id)
}

pub async fn handle(
    job: &Job,
    photo_service: &ProductPhotoService,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    debug!(job_id = job.id, "jobs:handle delete_product_images start");
    let product_id = parse(job)?;
    photo_service
        .delete_product_images(product_id)
        .await
        .map_err(|e| format!("delete_product_images failed: {e}"))?;
    info!(
        job_id = job.id,
        %product_id,
        "jobs:handle delete_product_images success"
    );
    Ok(())
}
