//! Handler for the `embed_product_images` job kind.
//!
//! Payload shape: `{ "product_id": "<uuid>", "image_indices": [0, 1, 2] }`
//! No blobs — we re-download `medium.webp` from S3 to compute the
//! `image_hash` against the canonical webp variant (so re-uploading the
//! same source picture in a different raw format doesn't trigger
//! reindex).
//!
//! Skip logic: per index, compute SHA256 of `medium.webp` bytes; if it
//! matches the stored row's `image_hash`, skip the embed call. Sends only
//! changed images in a single Cohere batch, then UPSERTs row-by-row.

use std::sync::Arc;

use pgvector::Vector;
use serde::Deserialize;
use sqlx::PgPool;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::embeddings::{ImageInput, MultimodalEmbedder};
use crate::jobs::text_embedder::sha256_hex;
use crate::storage::traits::ImageStorage;

use super::Job;

#[derive(Debug, Deserialize)]
pub struct Payload {
    pub product_id: Uuid,
    pub image_indices: Vec<i32>,
}

pub fn parse(job: &Job) -> Result<Payload, String> {
    debug!(job_id = job.id, kind = %job.kind, "jobs:parse embed_product_images");
    serde_json::from_value::<Payload>(job.payload.clone()).map_err(|e| format!("payload: {e}"))
}

pub fn medium_key(product_id: Uuid, image_idx: i32) -> String {
    format!("products/{product_id}/{image_idx}/medium.webp")
}

pub struct EmbedImagesDeps {
    pub pool: PgPool,
    pub storage: Arc<dyn ImageStorage>,
    pub images_bucket: String,
    pub embedder: Arc<dyn MultimodalEmbedder>,
}

pub async fn handle(
    job: &Job,
    deps: &EmbedImagesDeps,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let payload = parse(job)?;
    let product_id = payload.product_id;
    debug!(
        job_id = job.id,
        %product_id,
        indices = ?payload.image_indices,
        "jobs:handle embed_product_images start"
    );

    // For each index: fetch medium.webp, compute hash, compare with stored row.
    let mut to_embed: Vec<Pending> = Vec::with_capacity(payload.image_indices.len());
    for image_idx in &payload.image_indices {
        let key = medium_key(product_id, *image_idx);
        let bytes = match deps.storage.get_object(&deps.images_bucket, &key).await {
            Ok(b) => b,
            Err(e) => {
                // Missing object isn't a permanent failure for the job — log and skip.
                warn!(
                    %product_id, image_idx, %key, error = %e,
                    "embed_product_images: medium.webp not available, skipping"
                );
                continue;
            }
        };
        let hash = sha256_hex(&bytes);

        let stored: Option<String> = sqlx::query_scalar(
            r#"
            SELECT image_hash
            FROM product_image_embeddings
            WHERE product_id = $1 AND image_idx = $2
            "#,
        )
        .bind(product_id)
        .bind(*image_idx)
        .fetch_optional(&deps.pool)
        .await?;

        if stored.as_deref() == Some(&hash) {
            debug!(%product_id, image_idx, "embed_product_images: unchanged, skip");
            continue;
        }
        to_embed.push(Pending {
            image_idx: *image_idx,
            bytes,
            hash,
        });
    }

    if to_embed.is_empty() {
        info!(%product_id, "embed_product_images: nothing to embed");
        return Ok(());
    }

    let images: Vec<ImageInput> = to_embed
        .iter()
        .map(|p| ImageInput::webp(p.bytes.clone()))
        .collect();
    let vectors = deps.embedder.embed_image(&images).await?;
    if vectors.len() != to_embed.len() {
        return Err(format!(
            "embed_product_images: vector count mismatch {} vs {}",
            vectors.len(),
            to_embed.len()
        )
        .into());
    }

    let mut written = 0usize;
    for (p, raw) in to_embed.into_iter().zip(vectors.into_iter()) {
        let pgv = Vector::from(raw);
        sqlx::query(
            r#"
            INSERT INTO product_image_embeddings (product_id, image_idx, embedding, image_hash, updated_at)
            VALUES ($1, $2, $3, $4, now())
            ON CONFLICT (product_id, image_idx) DO UPDATE
                SET embedding  = EXCLUDED.embedding,
                    image_hash = EXCLUDED.image_hash,
                    updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(product_id)
        .bind(p.image_idx)
        .bind(pgv)
        .bind(p.hash)
        .execute(&deps.pool)
        .await?;
        written += 1;
    }
    info!(%product_id, written, "embed_product_images: done");
    Ok(())
}

struct Pending {
    image_idx: i32,
    bytes: Vec<u8>,
    hash: String,
}
