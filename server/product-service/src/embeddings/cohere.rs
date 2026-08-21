use async_trait::async_trait;
use base64::Engine;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::time::sleep;
use tracing::{debug, info, warn};

use super::traits::{EmbedError, ImageInput, MultimodalEmbedder};

const IMAGE_RETRY_WAIT: Duration = Duration::from_secs(65);
const IMAGE_MAX_RETRIES: usize = 3;

/// Cohere v2 embed client.
///
/// API reference: https://docs.cohere.com/v2/reference/embed
/// Multimodal note: `embed-multilingual-v3.0` accepts both `texts` and
/// `images` inputs (separately) and returns vectors in the same space,
/// which is what the hybrid retrieval relies on.
pub struct CohereEmbedder {
    client: Client,
    api_key: String,
    base_url: String,
    model: String,
    dim: usize,
    image_throttle: Duration,
    /// Last image-embed call time, shared so concurrent jobs respect the
    /// global per-minute rate cap (Cohere counts requests, not callers).
    last_image_call: Arc<Mutex<Option<Instant>>>,
}

impl CohereEmbedder {
    pub fn new(
        api_key: String,
        base_url: String,
        model: String,
        dim: usize,
        timeout: Duration,
        image_throttle: Duration,
    ) -> Result<Self, EmbedError> {
        if api_key.is_empty() {
            return Err(EmbedError::InvalidConfig(
                "MULTIMODAL_EMBED_API_KEY is empty".into(),
            ));
        }
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| EmbedError::Transport(e.to_string()))?;
        let base_url = base_url.trim_end_matches('/').to_string();
        Ok(Self {
            client,
            api_key,
            base_url,
            model,
            dim,
            image_throttle,
            last_image_call: Arc::new(Mutex::new(None)),
        })
    }

    async fn call(&self, body: serde_json::Value) -> Result<EmbedResponse, EmbedError> {
        let url = format!("{}/v2/embed", self.base_url);
        let resp = self
            .client
            .post(&url)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| EmbedError::Transport(e.to_string()))?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            warn!(%status, %body, "cohere.embed_failed");
            return Err(EmbedError::Status {
                status: status.as_u16(),
                body,
            });
        }
        let parsed: EmbedResponse = resp
            .json()
            .await
            .map_err(|e| EmbedError::UnexpectedShape(e.to_string()))?;
        Ok(parsed)
    }

    fn validate_dim(&self, vectors: &[Vec<f32>]) -> Result<(), EmbedError> {
        for (i, v) in vectors.iter().enumerate() {
            if v.len() != self.dim {
                return Err(EmbedError::UnexpectedShape(format!(
                    "vector {i}: got dim {}, expected {}",
                    v.len(),
                    self.dim
                )));
            }
        }
        Ok(())
    }

    /// Block until `image_throttle` has elapsed since the previous image call.
    async fn wait_for_image_slot(&self) {
        if self.image_throttle.is_zero() {
            return;
        }
        let to_wait = {
            let mut last = self.last_image_call.lock().await;
            let now = Instant::now();
            let wait = match *last {
                Some(prev) => self
                    .image_throttle
                    .checked_sub(now.duration_since(prev))
                    .unwrap_or_default(),
                None => Duration::ZERO,
            };
            // Reserve the slot now so concurrent callers don't all wake at once.
            *last = Some(now + wait);
            wait
        };
        if !to_wait.is_zero() {
            sleep(to_wait).await;
        }
    }

    /// Call `/v2/embed` for image input. On 429 wait ~1 minute and retry
    /// up to `IMAGE_MAX_RETRIES` times. Other errors surface immediately
    /// so the job-queue can fail fast on permanent problems.
    async fn call_image_with_backoff(
        &self,
        body: serde_json::Value,
    ) -> Result<EmbedResponse, EmbedError> {
        let mut attempt = 0usize;
        loop {
            match self.call(body.clone()).await {
                Ok(resp) => return Ok(resp),
                Err(EmbedError::Status { status: 429, body: msg }) => {
                    attempt += 1;
                    if attempt > IMAGE_MAX_RETRIES {
                        return Err(EmbedError::Status { status: 429, body: msg });
                    }
                    info!(
                        attempt,
                        wait_ms = IMAGE_RETRY_WAIT.as_millis() as u64,
                        "cohere.image_429_backoff"
                    );
                    sleep(IMAGE_RETRY_WAIT).await;
                    // Reset the throttle anchor — we've effectively slept past it.
                    *self.last_image_call.lock().await = Some(Instant::now());
                }
                Err(e) => return Err(e),
            }
        }
    }
}

#[async_trait]
impl MultimodalEmbedder for CohereEmbedder {
    async fn embed_text(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbedError> {
        if texts.is_empty() {
            return Ok(vec![]);
        }
        debug!(count = texts.len(), "cohere.embed_text");
        let body = json!({
            "model": self.model,
            "input_type": "search_document",
            "embedding_types": ["float"],
            "texts": texts,
        });
        let parsed = self.call(body).await?;
        let vectors = parsed.into_vectors()?;
        self.validate_dim(&vectors)?;
        Ok(vectors)
    }

    async fn embed_image(&self, images: &[ImageInput]) -> Result<Vec<Vec<f32>>, EmbedError> {
        if images.is_empty() {
            return Ok(vec![]);
        }
        debug!(count = images.len(), "cohere.embed_image");
        // Cohere v2 caps `images` at 1 per request, and the image-mode
        // rate limit is much lower than text (e.g. ~5/min on trial). We
        // serialise calls, sleep `image_throttle` between them, and
        // absorb 429s with a single ~minute-long wait + retry instead of
        // letting the surrounding job-queue burn its retry budget.
        let mut out: Vec<Vec<f32>> = Vec::with_capacity(images.len());
        for im in images {
            self.wait_for_image_slot().await;
            let b64 = base64::engine::general_purpose::STANDARD.encode(&im.bytes);
            let data_url = format!("data:{};base64,{}", im.content_type, b64);
            let body = json!({
                "model": self.model,
                "input_type": "image",
                "embedding_types": ["float"],
                "images": [data_url],
            });

            let parsed = self.call_image_with_backoff(body).await?;
            let mut vectors = parsed.into_vectors()?;
            self.validate_dim(&vectors)?;
            if let Some(v) = vectors.pop() {
                out.push(v);
            } else {
                return Err(EmbedError::UnexpectedShape(
                    "cohere returned no vector for image".into(),
                ));
            }
        }
        Ok(out)
    }

    fn dim(&self) -> usize {
        self.dim
    }
}

#[derive(Debug, Deserialize)]
struct EmbedResponse {
    embeddings: Embeddings,
}

#[derive(Debug, Deserialize)]
struct Embeddings {
    float: Option<Vec<Vec<f32>>>,
}

impl EmbedResponse {
    fn into_vectors(self) -> Result<Vec<Vec<f32>>, EmbedError> {
        self.embeddings.float.ok_or_else(|| {
            EmbedError::UnexpectedShape(
                "cohere response missing `embeddings.float`; was `embedding_types` set?".into(),
            )
        })
    }
}
