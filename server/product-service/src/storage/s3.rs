use std::time::Duration;

use async_trait::async_trait;
use aws_sdk_s3::Client;
use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::primitives::ByteStream;

use super::error::StorageError;
use super::traits::ImageStorage;

pub struct S3ImageStorage {
    client: Client,
    presign_client: Client,
    public_url: String,
}

impl S3ImageStorage {
    pub fn new(client: Client, presign_client: Client, public_url: String) -> Self {
        Self {
            client,
            presign_client,
            public_url,
        }
    }
}

#[async_trait]
impl ImageStorage for S3ImageStorage {
    async fn put_object(
        &self,
        bucket: &str,
        key: &str,
        data: Vec<u8>,
        content_type: &str,
    ) -> Result<(), StorageError> {
        self.client
            .put_object()
            .bucket(bucket)
            .key(key)
            .body(ByteStream::from(data))
            .content_type(content_type)
            .send()
            .await
            .map_err(|e| StorageError::Internal(format!("S3 put_object failed: {e}")))?;
        Ok(())
    }

    async fn delete_object(&self, bucket: &str, key: &str) -> Result<(), StorageError> {
        self.client
            .delete_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| StorageError::Internal(format!("S3 delete_object failed: {e}")))?;
        Ok(())
    }

    async fn get_object(&self, bucket: &str, key: &str) -> Result<Vec<u8>, StorageError> {
        let resp = self
            .client
            .get_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| {
                let msg = format!("S3 get_object failed: {e}");
                let s = e.to_string();
                if s.contains("NoSuchKey") || s.contains("404") {
                    StorageError::NotFound(msg)
                } else {
                    StorageError::Internal(msg)
                }
            })?;
        let bytes = resp
            .body
            .collect()
            .await
            .map_err(|e| StorageError::Internal(format!("S3 get_object body: {e}")))?;
        Ok(bytes.into_bytes().to_vec())
    }

    async fn delete_prefix(&self, bucket: &str, prefix: &str) -> Result<(), StorageError> {
        let keys = self.list_keys(bucket, prefix).await?;
        for key in keys {
            self.delete_object(bucket, &key).await?;
        }
        Ok(())
    }

    async fn list_keys(&self, bucket: &str, prefix: &str) -> Result<Vec<String>, StorageError> {
        let mut keys = Vec::new();
        let mut continuation_token: Option<String> = None;

        loop {
            let mut req = self.client.list_objects_v2().bucket(bucket).prefix(prefix);

            if let Some(token) = continuation_token.take() {
                req = req.continuation_token(token);
            }

            let resp = req
                .send()
                .await
                .map_err(|e| StorageError::Internal(format!("S3 list_objects failed: {e}")))?;

            for obj in resp.contents() {
                if let Some(key) = obj.key() {
                    keys.push(key.to_string());
                }
            }

            if resp.is_truncated() == Some(true) {
                continuation_token = resp.next_continuation_token().map(String::from);
            } else {
                break;
            }
        }

        Ok(keys)
    }

    async fn presigned_get_url(
        &self,
        bucket: &str,
        key: &str,
        ttl: Duration,
    ) -> Result<String, StorageError> {
        let presigning = PresigningConfig::builder()
            .expires_in(ttl)
            .build()
            .map_err(|e| StorageError::Internal(format!("presigning config error: {e}")))?;

        let presigned = self
            .presign_client
            .get_object()
            .bucket(bucket)
            .key(key)
            .presigned(presigning)
            .await
            .map_err(|e| StorageError::Internal(format!("S3 presigned URL failed: {e}")))?;

        Ok(presigned.uri().to_string())
    }

    fn public_url(&self, bucket: &str, key: &str) -> String {
        format!("{}/{}/{}", self.public_url, bucket, key)
    }
}
