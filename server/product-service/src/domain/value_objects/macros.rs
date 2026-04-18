/// Generates a validated name value object: trim + not empty + max chars.
macro_rules! define_name_value_object {
    ($name:ident, $max_chars:expr, $label:expr) => {
        use crate::domain::error::ServiceError;

        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $name(String);

        impl $name {
            pub const MAX_CHARS: usize = $max_chars;

            pub fn parse(raw: &str) -> Result<Self, ServiceError> {
                let normalized = raw.trim();

                if normalized.is_empty() {
                    return Err(ServiceError::BadRequest(format!(
                        "{} must not be empty",
                        $label
                    )));
                }

                if normalized.chars().count() > Self::MAX_CHARS {
                    return Err(ServiceError::BadRequest(format!(
                        "{} must be at most {} characters",
                        $label,
                        Self::MAX_CHARS,
                    )));
                }

                Ok(Self(normalized.to_string()))
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }

            pub fn into_inner(self) -> String {
                self.0
            }
        }

        impl TryFrom<&str> for $name {
            type Error = ServiceError;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                Self::parse(value)
            }
        }

        impl From<$name> for String {
            fn from(value: $name) -> Self {
                value.into_inner()
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }
    };
}

pub(crate) use define_name_value_object;
