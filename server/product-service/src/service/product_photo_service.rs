use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::domain::error::ServiceError;
use crate::domain::product_photo::UploadFile;
use crate::domain::utils::mappers::{ImageVariantUrls, ProductImageUrls, ResolvedImageUrls};
use crate::repo::traits::product_photo_repo::ProductPhotoRepository;
use crate::service::utils::image_processor::build_variants;
use crate::service::utils::validators::{
    ValidatedUploadFile, validate_files_not_empty, validate_upload_file, validate_upload_files,
};
use crate::storage::traits::ImageStorage;

const PRESIGNED_URL_TTL: Duration = Duration::from_secs(3600);
const WEBP_CONTENT_TYPE: &str = "image/webp";

type Result<T> = std::result::Result<T, ServiceError>;

pub struct ProductPhotoService {
    repo: Arc<dyn ProductPhotoRepository>,
    image_storage: Arc<dyn ImageStorage>,
    images_bucket: String,
    preview_bucket: String,
}

impl ProductPhotoService {
    pub fn new(
        repo: Arc<dyn ProductPhotoRepository>,
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

    pub async fn enqueue_images_upload(
        &self,
        product_id: Uuid,
        files: Vec<UploadFile>,
    ) -> Result<i64> {
        debug!(%product_id, files = files.len(), "service:enqueue_images_upload");
        validate_files_not_empty(files.len())?;

        self.verify_product_exists(product_id).await?;
        let job_id = self
            .repo
            .enqueue_images_upload(product_id, files)
            .await
            .map_err(ServiceError::from)?;
        info!(%product_id, %job_id, "service:enqueue_images_upload queued");
        Ok(job_id)
    }

    pub async fn enqueue_preview_upload(&self, product_id: Uuid, file: UploadFile) -> Result<i64> {
        debug!(%product_id, "service:enqueue_preview_upload");
        self.verify_product_exists(product_id).await?;
        let job_id = self
            .repo
            .enqueue_preview_upload(product_id, file)
            .await
            .map_err(ServiceError::from)?;
        info!(%product_id, %job_id, "service:enqueue_preview_upload queued");
        Ok(job_id)
    }

    /// Upload product images. Each file is processed into 3 webp variants
    /// (thumb/medium/full) stored at `products/{id}/{n}/{variant}.webp`.
    /// Returns the number of source files processed.
    pub async fn upload_images(&self, product_id: Uuid, files: Vec<UploadFile>) -> Result<usize> {
        debug!(%product_id, files = files.len(), "service:upload_images");
        let validated: Vec<ValidatedUploadFile> = validate_upload_files(files)?;

        self.verify_product_exists(product_id).await?;

        let prefix = image_prefix(product_id);
        let existing_indices =
            list_image_indices(&*self.image_storage, &self.images_bucket, &prefix).await?;
        let start_index = existing_indices.iter().max().map(|m| m + 1).unwrap_or(0);
        let count = validated.len();

        for (i, file) in validated.into_iter().enumerate() {
            let idx = start_index + i;
            let variants = build_variants(&file.data)?;
            for vb in variants {
                let key = format!("{}{}/{}.webp", prefix, idx, vb.variant.name());
                self.image_storage
                    .put_object(&self.images_bucket, &key, vb.data, WEBP_CONTENT_TYPE)
                    .await
                    .map_err(ServiceError::from)?;
                debug!(%product_id, %key, "service:upload_images variant stored");
            }
        }

        info!(%product_id, uploaded = count, "service:upload_images succeeded");
        Ok(count)
    }

    /// Upload a preview image. Stored as 3 webp variants at
    /// `previews/{id}/{variant}.webp`, replacing any existing preview.
    pub async fn upload_preview(&self, product_id: Uuid, file: UploadFile) -> Result<String> {
        debug!(%product_id, "service:upload_preview");
        let file = validate_upload_file(file)?;
        self.verify_product_exists(product_id).await?;

        let variants = build_variants(&file.data)?;
        for vb in variants {
            let key = format!("previews/{}/{}.webp", product_id, vb.variant.name());
            self.image_storage
                .put_object(&self.preview_bucket, &key, vb.data, WEBP_CONTENT_TYPE)
                .await
                .map_err(ServiceError::from)?;
            debug!(%product_id, %key, "service:upload_preview variant stored");
        }

        info!(%product_id, "service:upload_preview succeeded");
        Ok(format!("previews/{}/", product_id))
    }

    /// Delete all regular product images. Preview images are stored in a
    /// separate bucket/prefix and are intentionally left untouched.
    pub async fn delete_product_images(&self, product_id: Uuid) -> Result<()> {
        debug!(%product_id, "service:delete_product_images");

        let prefix = image_prefix(product_id);
        self.image_storage
            .delete_prefix(&self.images_bucket, &prefix)
            .await
            .map_err(ServiceError::from)?;

        info!(%product_id, "service:delete_product_images succeeded");
        Ok(())
    }

    /// Delete the preview image set for a product. No-op if no preview exists.
    pub async fn delete_preview(&self, product_id: Uuid) -> Result<()> {
        debug!(%product_id, "service:delete_preview");
        self.verify_product_exists(product_id).await?;

        let prefix = format!("previews/{}/", product_id);
        self.image_storage
            .delete_prefix(&self.preview_bucket, &prefix)
            .await
            .map_err(ServiceError::from)?;

        info!(%product_id, "service:delete_preview succeeded");
        Ok(())
    }

    /// Delete a single image (all variants) by its storage index.
    /// Returns `NotFound` if no image exists at that index.
    pub async fn delete_image(&self, product_id: Uuid, image_id: usize) -> Result<()> {
        debug!(%product_id, image_id, "service:delete_image");
        self.verify_product_exists(product_id).await?;

        let prefix = format!("{}{}/", image_prefix(product_id), image_id);
        let keys = self
            .image_storage
            .list_keys(&self.images_bucket, &prefix)
            .await
            .map_err(ServiceError::from)?;

        if keys.is_empty() {
            warn!(%product_id, image_id, "service:delete_image not found");
            return Err(ServiceError::NotFound(format!(
                "image {image_id} not found for product {product_id}"
            )));
        }

        self.image_storage
            .delete_prefix(&self.images_bucket, &prefix)
            .await
            .map_err(ServiceError::from)?;

        info!(%product_id, image_id, "service:delete_image succeeded");
        Ok(())
    }

    pub async fn resolve_preview_urls(&self, product_id: Uuid) -> Result<ImageVariantUrls> {
        debug!(%product_id, "service:resolve_preview_urls");
        let prefix = format!("previews/{}/", product_id);
        let keys = self
            .image_storage
            .list_keys(&self.preview_bucket, &prefix)
            .await
            .map_err(ServiceError::from)?;

        if keys.is_empty() {
            return Err(ServiceError::NotFound(format!(
                "preview for product {product_id} not found"
            )));
        }

        self.presign_variants(&self.preview_bucket, &keys).await
    }

    pub async fn resolve_image_urls_at(
        &self,
        product_id: Uuid,
        image_id: usize,
    ) -> Result<ProductImageUrls> {
        debug!(%product_id, image_id, "service:resolve_image_urls_at");
        let prefix = format!("{}{}/", image_prefix(product_id), image_id);
        let keys = self
            .image_storage
            .list_keys(&self.images_bucket, &prefix)
            .await
            .map_err(ServiceError::from)?;

        if keys.is_empty() {
            return Err(ServiceError::NotFound(format!(
                "image {image_id} not found for product {product_id}"
            )));
        }

        let variants = self.presign_variants(&self.images_bucket, &keys).await?;
        Ok(ProductImageUrls {
            id: image_id,
            thumb: variants.thumb,
            medium: variants.medium,
            full: variants.full,
        })
    }

    /// Resolve preview + all image variant URLs for a product.
    pub async fn resolve_image_urls(&self, product_id: Uuid) -> ResolvedImageUrls {
        debug!(%product_id, "service:resolve_image_urls");
        let preview_url = self.resolve_preview_urls(product_id).await.ok();

        let prefix = image_prefix(product_id);
        let indices = list_image_indices(&*self.image_storage, &self.images_bucket, &prefix)
            .await
            .unwrap_or_default();

        let mut image_urls = Vec::with_capacity(indices.len());
        for i in indices {
            if let Ok(v) = self.resolve_image_urls_at(product_id, i).await {
                image_urls.push(v);
            }
        }

        ResolvedImageUrls {
            preview_url,
            image_urls,
        }
    }

    /// Build `ImageVariantUrls` from a set of keys under one image directory.
    /// Missing variants fall back to any variant that is present.
    async fn presign_variants(&self, bucket: &str, keys: &[String]) -> Result<ImageVariantUrls> {
        let find = |name: &str| -> Option<&String> {
            keys.iter()
                .find(|k| k.ends_with(&format!("/{}.webp", name)))
        };

        let thumb_key = find("thumb")
            .or_else(|| find("medium"))
            .or_else(|| find("full"));
        let medium_key = find("medium")
            .or_else(|| find("full"))
            .or_else(|| find("thumb"));
        let full_key = find("full")
            .or_else(|| find("medium"))
            .or_else(|| find("thumb"));

        let (thumb, medium, full) = match (thumb_key, medium_key, full_key) {
            (Some(t), Some(m), Some(f)) => (t, m, f),
            _ => {
                return Err(ServiceError::NotFound(
                    "image variants not found".to_string(),
                ));
            }
        };

        let thumb_url = self
            .image_storage
            .presigned_get_url(bucket, thumb, PRESIGNED_URL_TTL)
            .await
            .map_err(ServiceError::from)?;
        let medium_url = self
            .image_storage
            .presigned_get_url(bucket, medium, PRESIGNED_URL_TTL)
            .await
            .map_err(ServiceError::from)?;
        let full_url = self
            .image_storage
            .presigned_get_url(bucket, full, PRESIGNED_URL_TTL)
            .await
            .map_err(ServiceError::from)?;

        Ok(ImageVariantUrls {
            thumb: thumb_url,
            medium: medium_url,
            full: full_url,
        })
    }

    async fn verify_product_exists(&self, id: Uuid) -> Result<()> {
        debug!(product_id = %id, "service:verify_product_exists");
        let exists = self
            .repo
            .product_exists(id)
            .await
            .map_err(ServiceError::from)?;

        if !exists {
            warn!(product_id = %id, "service:verify_product_exists not found");
            return Err(ServiceError::NotFound(format!("product {id} not found")));
        }

        Ok(())
    }
}

fn image_prefix(product_id: Uuid) -> String {
    format!("products/{}/", product_id)
}

/// Enumerate numeric sub-directory indices under `products/{id}/`.
/// With the variant layout, each image lives at `products/{id}/{n}/{variant}.webp`
/// — this scans keys and extracts the sorted unique `{n}` values.
async fn list_image_indices(
    storage: &dyn ImageStorage,
    bucket: &str,
    prefix: &str,
) -> Result<Vec<usize>> {
    let keys = storage
        .list_keys(bucket, prefix)
        .await
        .map_err(ServiceError::from)?;
    let mut indices: Vec<usize> = keys
        .iter()
        .filter_map(|k| {
            k.strip_prefix(prefix)
                .and_then(|rest| rest.split('/').next())
                .and_then(|seg| seg.parse::<usize>().ok())
        })
        .collect();
    indices.sort_unstable();
    indices.dedup();
    Ok(indices)
}
