use std::sync::Arc;
use std::time::Duration;

use super::super::repo::traits::product_repo::ProductRepository;
use super::super::storage::traits::ImageStorage;

const PRESIGNED_URL_TTL: Duration = Duration::from_secs(3600);

pub struct UploadFile {
    pub data: Vec<u8>,
    pub content_type: String,
    pub extension: String,
}

pub struct ProductService {
    repo: Arc<dyn ProductRepository>,
    image_storage: Arc<dyn ImageStorage>,
    images_bucket: String,
    preview_bucket: String,
}

impl ProductService {
    pub fn new(
        repo: Arc<dyn ProductRepository>,
        image_storage: Arc<dyn ImageStorage>,
        images_bucket: String,
        preview_bucket: String,
    ) -> Self {
        Self {
            repo,
            image_storage,
            images_bucket,
            preview_bucket,
        }
    }
}
