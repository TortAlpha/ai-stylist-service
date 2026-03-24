use async_trait::async_trait;
use aws_sdk_s3::Client;

use super::traits::{ImageStorage, };

pub struct S3ImageStorage {
    client: Client,
    internal_url: String,
    public_url: String,
}

impl S3ImageStorage {
    pub fn new(client: Client, internal_url: String, public_url: String) -> Self {
        Self { client, internal_url, public_url }
    }
}

#[async_trait]
impl ImageStorage for S3ImageStorage {
}
