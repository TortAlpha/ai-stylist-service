//! Multimodal embedding client.
//!
//! Phase 0a uses only text embedding; image embedding lands in Phase 0b
//! against the same Cohere `embed-multilingual-v3.0` model so the
//! resulting vectors share a single space with `ai-service` query vectors.

mod cohere;
mod traits;

pub use cohere::CohereEmbedder;
pub use traits::{EmbedError, ImageInput, MultimodalEmbedder};
