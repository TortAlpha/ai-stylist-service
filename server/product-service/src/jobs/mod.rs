//! Lightweight Postgres-backed job queue.
//!
//! Workers poll the `jobs` table with `FOR UPDATE SKIP LOCKED`, run a
//! handler, and either delete the row on success or schedule a retry
//! with exponential backoff. The row lock is held for the duration of
//! the handler, so other workers won't pick up the same job.
//!
//! Atomicity guarantee: callers can `enqueue_*` inside an existing
//! `Transaction<'_, Postgres>`, so a job is created if and only if the
//! surrounding business write commits.

pub mod upload_product_images;
pub mod upload_product_preview;

use std::sync::Arc;
use std::time::Duration;

use serde_json::Value as JsonValue;
use sqlx::{PgPool, Postgres, Transaction};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::domain::product_photo::UploadFile;
use crate::service::product_photo_service::ProductPhotoService;

pub const KIND_UPLOAD_PRODUCT_IMAGES: &str = "upload_product_images";
pub const KIND_UPLOAD_PRODUCT_PREVIEW: &str = "upload_product_preview";

const POLL_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Debug, sqlx::FromRow)]
pub struct JobRow {
    pub id: i64,
    pub kind: String,
    pub payload: JsonValue,
    pub blobs: Option<Vec<Vec<u8>>>,
    pub blob_types: Option<Vec<String>>,
    pub attempts: i32,
    pub max_attempts: i32,
}

impl JobRow {
    /// Construct a job row in-memory. Intended for tests only.
    pub fn for_test(
        kind: impl Into<String>,
        payload: JsonValue,
        blobs: Option<Vec<Vec<u8>>>,
        blob_types: Option<Vec<String>>,
    ) -> Self {
        Self {
            id: 0,
            kind: kind.into(),
            payload,
            blobs,
            blob_types,
            attempts: 0,
            max_attempts: 5,
        }
    }
}

/// Enqueue an `upload_product_images` job inside an existing transaction.
/// The job will only become visible to workers after the transaction commits.
pub async fn enqueue_upload_product_images(
    tx: &mut Transaction<'_, Postgres>,
    product_id: Uuid,
    files: Vec<UploadFile>,
) -> Result<i64, sqlx::Error> {
    let payload = serde_json::json!({ "product_id": product_id });
    let (blobs, blob_types): (Vec<Vec<u8>>, Vec<String>) =
        files.into_iter().map(|f| (f.data, f.content_type)).unzip();
    let files_count = blobs.len();
    let total_bytes: usize = blobs.iter().map(Vec::len).sum();
    debug!(
        %product_id,
        files = files_count,
        total_bytes,
        "jobs:enqueue upload_product_images"
    );

    let id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO jobs (kind, payload, blobs, blob_types)
        VALUES ($1, $2, $3, $4)
        RETURNING id::BIGINT
        "#,
    )
    .bind(KIND_UPLOAD_PRODUCT_IMAGES)
    .bind(payload)
    .bind(&blobs)
    .bind(&blob_types)
    .fetch_one(tx.as_mut())
    .await?;

    info!(
        %product_id,
        job_id = id,
        files = files_count,
        total_bytes,
        "jobs:enqueue upload_product_images created"
    );
    Ok(id)
}

/// Enqueue an `upload_product_preview` job inside an existing transaction.
pub async fn enqueue_upload_product_preview(
    tx: &mut Transaction<'_, Postgres>,
    product_id: Uuid,
    file: UploadFile,
) -> Result<i64, sqlx::Error> {
    let payload = serde_json::json!({ "product_id": product_id });
    let size_bytes = file.data.len();
    let content_type = file.content_type.clone();
    debug!(
        %product_id,
        %content_type,
        size_bytes,
        "jobs:enqueue upload_product_preview"
    );

    let id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO jobs (kind, payload, blobs, blob_types)
        VALUES ($1, $2, $3, $4)
        RETURNING id::BIGINT
        "#,
    )
    .bind(KIND_UPLOAD_PRODUCT_PREVIEW)
    .bind(payload)
    .bind(&vec![file.data])
    .bind(&vec![file.content_type])
    .fetch_one(tx.as_mut())
    .await?;

    info!(
        %product_id,
        job_id = id,
        %content_type,
        size_bytes,
        "jobs:enqueue upload_product_preview created"
    );
    Ok(id)
}

/// Spawn the background worker loop. Runs until the process exits.
pub fn spawn_worker(pool: PgPool, photo_service: Arc<ProductPhotoService>) {
    tokio::spawn(async move {
        info!("jobs worker started");
        loop {
            match process_one(&pool, &photo_service).await {
                Ok(true) => {} // got a job — immediately try for more
                Ok(false) => tokio::time::sleep(POLL_INTERVAL).await,
                Err(e) => {
                    error!(error = %e, "jobs worker loop error");
                    tokio::time::sleep(POLL_INTERVAL).await;
                }
            }
        }
    });
}

/// Try to process a single job. Returns `Ok(true)` if a job was handled
/// (success or failure), `Ok(false)` if the queue was empty.
async fn process_one(
    pool: &PgPool,
    photo_service: &ProductPhotoService,
) -> Result<bool, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let job: Option<JobRow> = sqlx::query_as(
        r#"
        SELECT id, kind, payload, blobs, blob_types, attempts, max_attempts
        FROM jobs
        WHERE status = 'pending' AND run_at <= now()
        ORDER BY run_at
        LIMIT 1
        FOR UPDATE SKIP LOCKED
        "#,
    )
    .fetch_optional(&mut *tx)
    .await?;

    let Some(job) = job else {
        tx.commit().await?;
        return Ok(false);
    };

    debug!(
        job_id = job.id,
        kind = %job.kind,
        attempts = job.attempts,
        max_attempts = job.max_attempts,
        "jobs worker picked job"
    );
    let result = handle(&job, photo_service).await;

    match result {
        Ok(()) => {
            sqlx::query("DELETE FROM jobs WHERE id = $1")
                .bind(job.id)
                .execute(&mut *tx)
                .await?;
            info!(job_id = job.id, kind = %job.kind, "job done");
        }
        Err(err) => {
            let next_attempts = job.attempts + 1;
            if next_attempts >= job.max_attempts {
                warn!(job_id = job.id, kind = %job.kind, error = %err, "job failed permanently");
                sqlx::query(
                    r#"
                    UPDATE jobs
                    SET status = 'failed',
                        attempts = $2,
                        last_error = $3,
                        updated_at = now()
                    WHERE id = $1
                    "#,
                )
                .bind(job.id)
                .bind(next_attempts)
                .bind(err.to_string())
                .execute(&mut *tx)
                .await?;
            } else {
                let backoff_secs = (2_i64.pow(next_attempts as u32)) * 30;
                warn!(
                    job_id = job.id,
                    kind = %job.kind,
                    attempt = next_attempts,
                    retry_in_s = backoff_secs,
                    error = %err,
                    "job failed, retrying"
                );
                sqlx::query(
                    r#"
                    UPDATE jobs
                    SET attempts = $2,
                        last_error = $3,
                        run_at = now() + make_interval(secs => $4),
                        updated_at = now()
                    WHERE id = $1
                    "#,
                )
                .bind(job.id)
                .bind(next_attempts)
                .bind(err.to_string())
                .bind(backoff_secs as f64)
                .execute(&mut *tx)
                .await?;
            }
        }
    }

    tx.commit().await?;
    Ok(true)
}

async fn handle(
    job: &JobRow,
    photo_service: &ProductPhotoService,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    debug!(job_id = job.id, kind = %job.kind, "jobs:dispatch handler");
    match job.kind.as_str() {
        KIND_UPLOAD_PRODUCT_IMAGES => upload_product_images::handle(job, photo_service).await,
        KIND_UPLOAD_PRODUCT_PREVIEW => upload_product_preview::handle(job, photo_service).await,
        other => {
            warn!(job_id = job.id, kind = other, "jobs:unknown kind");
            Err(format!("unknown job kind: {other}").into())
        }
    }
}

// Re-exported so the handler module can access JobRow internals.
pub(crate) use JobRow as Job;
