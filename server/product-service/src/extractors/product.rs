use actix_multipart::Multipart;
use futures::StreamExt;
use serde::de::DeserializeOwned;

use crate::domain::error::ServiceError;
use crate::domain::product_photo::UploadFile;

/// Parsed result from a multipart product request.
pub struct ProductMultipart<T> {
    pub metadata: T,
    pub preview: Option<UploadFile>,
    pub images: Vec<UploadFile>,
}

/// Extract a JSON `metadata` field, an optional `preview` image, and
/// zero or more `images` fields from a multipart/form-data payload.
///
/// Expected fields:
///   - `metadata` — a single JSON blob deserializable into `T`
///   - `preview`  — optional single image file (preview thumbnail)
///   - `images`   — zero or more binary image parts (gallery photos)
pub async fn extract_product_multipart<T: DeserializeOwned>(
    payload: &mut Multipart,
) -> Result<ProductMultipart<T>, ServiceError> {
    let mut metadata: Option<T> = None;
    let mut preview: Option<UploadFile> = None;
    let mut images: Vec<UploadFile> = Vec::new();

    while let Some(item) = payload.next().await {
        let mut field =
            item.map_err(|e| ServiceError::BadRequest(format!("multipart error: {e}")))?;

        let name = field.name().map(|s| s.to_string()).unwrap_or_default();

        match name.as_str() {
            "metadata" => {
                let mut buf = Vec::new();
                while let Some(chunk) = field.next().await {
                    let chunk =
                        chunk.map_err(|e| ServiceError::BadRequest(format!("read error: {e}")))?;
                    buf.extend_from_slice(&chunk);
                }
                metadata = Some(serde_json::from_slice(&buf).map_err(|e| {
                    ServiceError::BadRequest(format!("invalid metadata JSON: {e}"))
                })?);
            }
            "preview" => {
                let file = read_upload_file(&mut field).await?;
                preview = Some(file);
            }
            "images" => {
                let file = read_upload_file(&mut field).await?;
                images.push(file);
            }
            _ => {}
        }
    }

    let metadata =
        metadata.ok_or_else(|| ServiceError::BadRequest("missing 'metadata' field".into()))?;

    Ok(ProductMultipart {
        metadata,
        preview,
        images,
    })
}

async fn read_upload_file(field: &mut actix_multipart::Field) -> Result<UploadFile, ServiceError> {
    let content_type = field
        .content_type()
        .map(|ct| ct.to_string())
        .unwrap_or_default();
    let mut data = Vec::new();
    while let Some(chunk) = field.next().await {
        let chunk = chunk.map_err(|e| ServiceError::BadRequest(format!("read error: {e}")))?;
        data.extend_from_slice(&chunk);
    }
    Ok(UploadFile { data, content_type })
}
