#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub s3_images_bucket: String,
    pub s3_preview_bucket: String,
    pub s3_internal_url: String,
    pub s3_public_url: String,
}

impl Config {
    pub fn from_env() -> Self {
        let s3_internal_url =
            std::env::var("AWS_ENDPOINT_URL").unwrap_or_else(|_| "http://localhost:9000".into());
        let s3_public_url =
            std::env::var("S3_PUBLIC_URL").unwrap_or_else(|_| s3_internal_url.clone());

        Self {
            database_url: std::env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            port: std::env::var("SERVICE_PORT")
                .unwrap_or_else(|_| "8081".into())
                .parse()
                .expect("SERVICE_PORT must be a number"),
            s3_images_bucket: std::env::var("S3_IMAGES_BUCKET")
                .unwrap_or_else(|_| "ava-product-images".into()),
            s3_preview_bucket: std::env::var("S3_PREVIEW_BUCKET")
                .unwrap_or_else(|_| "ava-product-previews".into()),
            s3_internal_url,
            s3_public_url,
        }
    }
}
