use chrono::Utc;

use crate::domain::error::ServiceError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct YearOfRelease(i32);

impl YearOfRelease {
    pub const MIN_YEAR: i32 = 1900;

    pub fn parse(value: i32) -> Result<Self, ServiceError> {
        let max_year = Utc::now().format("%Y").to_string().parse::<i32>().unwrap() + 1;

        if value < Self::MIN_YEAR || value > max_year {
            return Err(ServiceError::BadRequest(format!(
                "year of release must be between {} and {}",
                Self::MIN_YEAR,
                max_year,
            )));
        }

        Ok(Self(value))
    }

    pub fn value(&self) -> i32 {
        self.0
    }

    pub fn into_inner(self) -> i32 {
        self.0
    }
}

impl From<YearOfRelease> for i32 {
    fn from(value: YearOfRelease) -> Self {
        value.into_inner()
    }
}
