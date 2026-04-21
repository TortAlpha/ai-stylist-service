use crate::domain::error::ServiceError;
use crate::domain::product_photo::UploadFile;

const MAX_IMAGE_SIZE: usize = 10 * 1024 * 1024; // 10 MB

#[derive(Debug)]
pub struct ValidatedUploadFile {
    pub data: Vec<u8>,
    pub content_type: String,
    pub extension: &'static str,
}

pub fn validate_files_not_empty(files_len: usize) -> Result<(), ServiceError> {
    if files_len == 0 {
        return Err(ServiceError::BadRequest("no files provided".into()));
    }

    Ok(())
}

pub fn validate_upload_file(file: UploadFile) -> Result<ValidatedUploadFile, ServiceError> {
    if file.data.len() > MAX_IMAGE_SIZE {
        return Err(ServiceError::BadRequest(format!(
            "file exceeds max size of {} bytes",
            MAX_IMAGE_SIZE
        )));
    }

    let extension = resolve_extension(&file.content_type)?;
    Ok(ValidatedUploadFile {
        data: file.data,
        content_type: file.content_type,
        extension,
    })
}

pub fn validate_upload_files(
    files: Vec<UploadFile>,
) -> Result<Vec<ValidatedUploadFile>, ServiceError> {
    validate_files_not_empty(files.len())?;
    files.into_iter().map(validate_upload_file).collect()
}

fn resolve_extension(content_type: &str) -> Result<&'static str, ServiceError> {
    match content_type {
        "image/jpeg" => Ok("jpg"),
        "image/png" => Ok("png"),
        "image/webp" => Ok("webp"),
        other => Err(ServiceError::BadRequest(format!(
            "unsupported content type: {other}. Allowed: image/jpeg, image/png, image/webp"
        ))),
    }
}
