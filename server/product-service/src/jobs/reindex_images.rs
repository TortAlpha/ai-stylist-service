//! One-shot image reindex backfill.
//!
//! Scans `product` rows where `status='ready' AND is_deleted=false` and,
//! for each, compares `image_count` against how many rows already exist
//! in `product_image_embeddings`. Any missing index gets an
//! `embed_product_images` job. Indices are derived by enumerating
//! `0..image_count` and subtracting the indices already present in the
//! embeddings table — this matches the S3 layout
//! (`products/{id}/{n}/medium.webp`) for sequentially-uploaded photos.
//!
//! Idempotent: re-running the backfill skips products already up-to-date.

use sqlx::PgPool;
use tracing::{info, warn};
use uuid::Uuid;

use super::enqueue_embed_product_images;

pub async fn run(pool: PgPool) -> Result<(), sqlx::Error> {
    info!("reindex_images: scan starting");

    let rows: Vec<Row> = sqlx::query_as::<_, Row>(
        r#"
        SELECT p.id, p.image_count
        FROM product p
        WHERE p.status = 'ready'
          AND p.is_deleted = false
          AND p.image_count > 0
          AND COALESCE(
                (SELECT COUNT(*) FROM product_image_embeddings WHERE product_id = p.id),
                0
              ) < p.image_count
        ORDER BY p.updated_at ASC
        "#,
    )
    .fetch_all(&pool)
    .await?;

    let mut enqueued_products = 0usize;
    let mut enqueued_indices = 0usize;
    for r in rows {
        let existing: Vec<i32> = sqlx::query_scalar(
            r#"
            SELECT image_idx FROM product_image_embeddings
            WHERE product_id = $1
            ORDER BY image_idx
            "#,
        )
        .bind(r.id)
        .fetch_all(&pool)
        .await?;
        let expected: Vec<i32> = (0..r.image_count).collect();
        let missing: Vec<i32> = expected
            .into_iter()
            .filter(|i| !existing.contains(i))
            .collect();
        if missing.is_empty() {
            continue;
        }

        let mut tx = pool.begin().await?;
        if let Err(e) = enqueue_embed_product_images(&mut tx, r.id, &missing).await {
            warn!(product_id = %r.id, error = %e, "reindex_images: enqueue failed");
            tx.rollback().await?;
            continue;
        }
        tx.commit().await?;
        enqueued_products += 1;
        enqueued_indices += missing.len();
    }

    info!(
        enqueued_products,
        enqueued_indices, "reindex_images: scan done"
    );
    Ok(())
}

#[derive(sqlx::FromRow)]
struct Row {
    id: Uuid,
    image_count: i32,
}
