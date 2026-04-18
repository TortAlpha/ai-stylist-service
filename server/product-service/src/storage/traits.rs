use std::time::Duration;

use async_trait::async_trait;

use super::error::StorageError;

#[async_trait]
pub trait ImageStorage: Send + Sync {
    /// Upload a file to the given bucket under the given key.
    async fn put_object(
        &self,
        bucket: &str,
        key: &str,
        data: Vec<u8>,
        content_type: &str,
    ) -> Result<(), StorageError>;

    /// Delete a single object.
    async fn delete_object(&self, bucket: &str, key: &str) -> Result<(), StorageError>;

    /// Delete all objects under a prefix (folder).
    async fn delete_prefix(&self, bucket: &str, prefix: &str) -> Result<(), StorageError>;

    /// List object keys under a prefix.
    async fn list_keys(&self, bucket: &str, prefix: &str) -> Result<Vec<String>, StorageError>;

    /// Generate a presigned GET URL valid for `ttl`.
    async fn presigned_get_url(
        &self,
        bucket: &str,
        key: &str,
        ttl: Duration,
    ) -> Result<String, StorageError>;

    /// Build a public URL (no signing) for a key.
    /// Used when the bucket is behind a public-read CDN/proxy.
    fn public_url(&self, bucket: &str, key: &str) -> String;
}
