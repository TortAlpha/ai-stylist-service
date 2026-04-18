#[derive(Debug)]
pub enum StorageError {
    NotFound(String),
    PermissionDenied(String),
    Internal(String),
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(msg) => write!(f, "Storage not found: {msg}"),
            Self::PermissionDenied(msg) => write!(f, "Storage permission denied: {msg}"),
            Self::Internal(msg) => write!(f, "Storage error: {msg}"),
        }
    }
}
