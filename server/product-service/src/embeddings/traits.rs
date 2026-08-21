use async_trait::async_trait;
use std::fmt;

/// Image input for `embed_image`. `bytes` is the raw `medium.webp` payload;
/// `content_type` defaults to `image/webp`.
#[derive(Clone)]
pub struct ImageInput {
    pub bytes: Vec<u8>,
    pub content_type: String,
}

impl ImageInput {
    pub fn webp(bytes: Vec<u8>) -> Self {
        Self {
            bytes,
            content_type: "image/webp".into(),
        }
    }
}

#[derive(Debug)]
pub enum EmbedError {
    Transport(String),
    Status { status: u16, body: String },
    UnexpectedShape(String),
    InvalidConfig(String),
}

impl fmt::Display for EmbedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Transport(e) => write!(f, "embed transport error: {e}"),
            Self::Status { status, body } => write!(f, "embed status {status}: {body}"),
            Self::UnexpectedShape(e) => write!(f, "embed unexpected response: {e}"),
            Self::InvalidConfig(e) => write!(f, "embed misconfigured: {e}"),
        }
    }
}

impl std::error::Error for EmbedError {}

#[async_trait]
pub trait MultimodalEmbedder: Send + Sync {
    /// Embed texts in the model's `search_document` mode (for index-side
    /// vectors). Query vectors should use `search_query` mode but ai-service
    /// owns query embedding, so this trait only exposes the index path.
    async fn embed_text(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, EmbedError>;

    /// Embed images. Phase 0b consumer. Default implementation rejects so
    /// Phase 0a code can stay agnostic while leaving the contract intact.
    async fn embed_image(&self, _images: &[ImageInput]) -> Result<Vec<Vec<f32>>, EmbedError> {
        Err(EmbedError::InvalidConfig(
            "embed_image not implemented for this provider yet".into(),
        ))
    }

    fn dim(&self) -> usize;
}
