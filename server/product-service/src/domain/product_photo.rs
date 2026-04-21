/// Raw file data extracted from multipart/form-data.
#[derive(Debug, Clone)]
pub struct UploadFile {
    pub data: Vec<u8>,
    pub content_type: String,
}
