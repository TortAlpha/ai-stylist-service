use std::time::Duration;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ServiceMode {
    /// Run the HTTP server (default).
    Server,
    /// Run the scheduled text embedder worker (no HTTP server).
    TextEmbedder,
    /// One-shot image reindex: enqueue embed_product_images for any
    /// `ready` product whose image rows are behind `image_count`, then exit.
    ReindexImages,
}

impl ServiceMode {
    fn from_env() -> Self {
        match std::env::var("SERVICE_MODE")
            .unwrap_or_else(|_| "server".into())
            .to_ascii_lowercase()
            .as_str()
        {
            "text-embedder" | "text_embedder" => Self::TextEmbedder,
            "reindex-images" | "reindex_images" => Self::ReindexImages,
            _ => Self::Server,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ImageEmbedAggregation {
    /// One row per image, MAX over images at query time. Default for MVP.
    PerImage,
    /// One vector per product (centroid of per-image vectors). Future toggle.
    Centroid,
}

impl ImageEmbedAggregation {
    fn from_env() -> Self {
        match std::env::var("IMAGE_EMBED_AGGREGATION")
            .unwrap_or_else(|_| "per_image".into())
            .to_ascii_lowercase()
            .as_str()
        {
            "centroid" => Self::Centroid,
            _ => Self::PerImage,
        }
    }
}

#[derive(Clone, Debug)]
pub struct EmbedConfig {
    pub provider: String,
    pub model: String,
    pub dim: usize,
    pub api_key: String,
    pub base_url: String,
    pub timeout: Duration,
    /// Minimum delay between Cohere image-embed HTTP calls. Cohere v2
    /// throttles `input_type=image` separately from text — trial tier is
    /// ~5/min, production ~40/min. Default 13_000ms is safe for trial.
    pub image_throttle: Duration,
}

#[derive(Clone, Debug)]
pub struct TextEmbedderConfig {
    pub batch_size: usize,
    pub interval: Duration,
    pub daily_cap: usize,
}

#[derive(Clone, Debug)]
pub struct ImageEmbedderConfig {
    pub aggregation: ImageEmbedAggregation,
    pub daily_cap: usize,
}

#[derive(Clone, Debug)]
pub struct HybridConfig {
    pub text_weight: f32,
    pub image_weight: f32,
}

#[derive(Clone)]
pub struct Config {
    pub mode: ServiceMode,
    pub database_url: String,
    pub port: u16,
    pub s3_images_bucket: String,
    pub s3_preview_bucket: String,
    pub s3_internal_url: String,
    pub s3_public_url: String,
    pub internal_api_token: String,
    pub embed: EmbedConfig,
    pub text_embedder: TextEmbedderConfig,
    pub image_embedder: ImageEmbedderConfig,
    pub hybrid: HybridConfig,
}

fn parse_env_or<T: std::str::FromStr>(key: &str, default: T) -> T {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

impl Config {
    pub fn from_env() -> Self {
        let s3_internal_url =
            std::env::var("AWS_ENDPOINT_URL").unwrap_or_else(|_| "http://localhost:9000".into());
        let s3_public_url =
            std::env::var("S3_PUBLIC_URL").unwrap_or_else(|_| s3_internal_url.clone());

        let embed = EmbedConfig {
            provider: std::env::var("MULTIMODAL_EMBED_PROVIDER").unwrap_or_else(|_| "cohere".into()),
            model: std::env::var("MULTIMODAL_EMBED_MODEL")
                .unwrap_or_else(|_| "embed-multilingual-v3.0".into()),
            dim: parse_env_or("MULTIMODAL_EMBED_DIM", 1024),
            api_key: std::env::var("MULTIMODAL_EMBED_API_KEY").unwrap_or_default(),
            base_url: std::env::var("MULTIMODAL_EMBED_BASE_URL")
                .unwrap_or_else(|_| "https://api.cohere.com".into()),
            timeout: Duration::from_secs(parse_env_or("MULTIMODAL_EMBED_TIMEOUT_SECONDS", 15_u64)),
            image_throttle: Duration::from_millis(parse_env_or("COHERE_IMAGE_THROTTLE_MS", 13_000_u64)),
        };

        let text_embedder = TextEmbedderConfig {
            batch_size: parse_env_or("TEXT_EMBEDDER_BATCH_SIZE", 64),
            interval: Duration::from_secs(parse_env_or("TEXT_EMBEDDER_INTERVAL_SECONDS", 600_u64)),
            daily_cap: parse_env_or("TEXT_EMBEDDER_DAILY_CAP", 50_000_usize),
        };

        let image_embedder = ImageEmbedderConfig {
            aggregation: ImageEmbedAggregation::from_env(),
            daily_cap: parse_env_or("IMAGE_EMBED_DAILY_CAP", 50_000_usize),
        };

        let hybrid = HybridConfig {
            text_weight: parse_env_or("HYBRID_TEXT_WEIGHT", 0.5_f32),
            image_weight: parse_env_or("HYBRID_IMAGE_WEIGHT", 0.5_f32),
        };

        Self {
            mode: ServiceMode::from_env(),
            database_url: std::env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            port: parse_env_or("SERVICE_PORT", 8081),
            s3_images_bucket: std::env::var("S3_IMAGES_BUCKET")
                .unwrap_or_else(|_| "ava-product-images".into()),
            s3_preview_bucket: std::env::var("S3_PREVIEW_BUCKET")
                .unwrap_or_else(|_| "ava-product-previews".into()),
            s3_internal_url,
            s3_public_url,
            internal_api_token: std::env::var("INTERNAL_API_TOKEN").unwrap_or_default(),
            embed,
            text_embedder,
            image_embedder,
            hybrid,
        }
    }
}
