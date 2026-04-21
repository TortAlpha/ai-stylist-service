use async_trait::async_trait;
use sqlx::Error;
use uuid::Uuid;

use crate::domain::product_photo::UploadFile;

#[cfg_attr(any(test, feature = "test-mocks"), mockall::automock)]
#[async_trait]
pub trait ProductPhotoRepository: Send + Sync {
    async fn product_exists(&self, id: Uuid) -> Result<bool, Error>;

    async fn enqueue_images_upload(
        &self,
        product_id: Uuid,
        files: Vec<UploadFile>,
    ) -> Result<i64, Error>;

    async fn enqueue_preview_upload(
        &self,
        product_id: Uuid,
        file: UploadFile,
    ) -> Result<i64, Error>;
}
