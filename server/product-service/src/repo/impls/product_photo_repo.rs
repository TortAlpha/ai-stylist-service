use async_trait::async_trait;
use sqlx::PgPool;
use tracing::{debug, info};
use uuid::Uuid;

use crate::domain::product_photo::UploadFile;
use crate::jobs;
use crate::repo::traits::product_photo_repo::ProductPhotoRepository;

pub struct PgProductPhotoRepo {
    pool: PgPool,
}

impl PgProductPhotoRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProductPhotoRepository for PgProductPhotoRepo {
    async fn product_exists(&self, id: Uuid) -> Result<bool, sqlx::Error> {
        debug!(product_id = %id, "repo:product_exists check");
        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM product WHERE id = $1 AND is_deleted = false)",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;
        debug!(product_id = %id, exists, "repo:product_exists result");
        Ok(exists)
    }

    async fn enqueue_images_upload(
        &self,
        product_id: Uuid,
        files: Vec<UploadFile>,
    ) -> Result<i64, sqlx::Error> {
        let files_count = files.len();
        let total_bytes: usize = files.iter().map(|f| f.data.len()).sum();
        debug!(
            %product_id,
            files = files_count,
            total_bytes,
            "repo:enqueue_images_upload start"
        );
        let mut tx = self.pool.begin().await?;
        let job_id = jobs::enqueue_upload_product_images(&mut tx, product_id, files).await?;
        tx.commit().await?;
        info!(
            %product_id,
            %job_id,
            files = files_count,
            total_bytes,
            "repo:enqueue_images_upload committed"
        );
        Ok(job_id)
    }

    async fn enqueue_preview_upload(
        &self,
        product_id: Uuid,
        file: UploadFile,
    ) -> Result<i64, sqlx::Error> {
        let content_type = file.content_type.clone();
        let size_bytes = file.data.len();
        debug!(
            %product_id,
            %content_type,
            size_bytes,
            "repo:enqueue_preview_upload start"
        );
        let mut tx = self.pool.begin().await?;
        let job_id = jobs::enqueue_upload_product_preview(&mut tx, product_id, file).await?;
        tx.commit().await?;
        info!(
            %product_id,
            %job_id,
            %content_type,
            size_bytes,
            "repo:enqueue_preview_upload committed"
        );
        Ok(job_id)
    }
}
