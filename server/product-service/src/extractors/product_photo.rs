use actix_multipart::Multipart;
use futures::StreamExt;
use tracing::{debug, warn};

use crate::domain::error::ServiceError;
use crate::domain::product_photo::UploadFile;

/// Extract binary photo files from multipart/form-data payload.
pub async fn extract_photo_files(payload: &mut Multipart) -> Result<Vec<UploadFile>, ServiceError> {
    let mut files = Vec::new();
    let mut field_index: usize = 0;

    while let Some(item) = payload.next().await {
        let mut field =
            item.map_err(|e| ServiceError::BadRequest(format!("multipart error: {e}")))?;
        let field_name = field.name().unwrap_or_default().to_string();

        let content_type = field
            .content_type()
            .map(|ct| ct.to_string())
            .unwrap_or_default();
        debug!(
            field_index,
            field_name = %field_name,
            content_type = %content_type,
            "extractor:reading photo multipart field"
        );

        let mut data = Vec::new();
        while let Some(chunk) = field.next().await {
            let chunk = chunk.map_err(|e| ServiceError::BadRequest(format!("read error: {e}")))?;
            data.extend_from_slice(&chunk);
        }

        if data.is_empty() {
            warn!(
                field_index,
                field_name = %field_name,
                "extractor:multipart photo field has empty payload"
            );
        }
        debug!(
            field_index,
            field_name = %field_name,
            size_bytes = data.len(),
            "extractor:photo field read complete"
        );
        files.push(UploadFile { data, content_type });
        field_index += 1;
    }

    debug!(
        files = files.len(),
        "extractor:photo multipart extraction complete"
    );
    Ok(files)
}
