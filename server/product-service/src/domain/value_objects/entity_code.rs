use crate::domain::error::ServiceError;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EntityCode(String);

impl EntityCode {
    pub const MAX_CHARS: usize = 50;

    pub fn parse(raw: &str, label: &str) -> Result<Self, ServiceError> {
        let normalized = raw.trim().to_lowercase();

        if normalized.is_empty() {
            return Err(ServiceError::BadRequest(format!(
                "{label} must not be empty"
            )));
        }

        if normalized.chars().count() > Self::MAX_CHARS {
            return Err(ServiceError::BadRequest(format!(
                "{label} must be at most {} characters",
                Self::MAX_CHARS,
            )));
        }

        Ok(Self(normalized))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl From<EntityCode> for String {
    fn from(value: EntityCode) -> Self {
        value.into_inner()
    }
}

impl AsRef<str> for EntityCode {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
