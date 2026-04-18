use crate::domain::error::ServiceError;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CurrencyCode(String);

const SUPPORTED_CURRENCIES: &[&str] = &["RSD", "EUR", "USD", "GBP"];

impl CurrencyCode {
    pub fn parse(raw: &str) -> Result<Self, ServiceError> {
        let normalized = raw.trim().to_uppercase();

        if !SUPPORTED_CURRENCIES.contains(&normalized.as_str()) {
            return Err(ServiceError::BadRequest(format!(
                "unsupported currency '{}', expected one of: {}",
                raw.trim(),
                SUPPORTED_CURRENCIES.join(", "),
            )));
        }

        Ok(Self(normalized))
    }

    pub fn rsd() -> Self {
        Self("RSD".to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl From<CurrencyCode> for String {
    fn from(value: CurrencyCode) -> Self {
        value.into_inner()
    }
}

impl AsRef<str> for CurrencyCode {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
