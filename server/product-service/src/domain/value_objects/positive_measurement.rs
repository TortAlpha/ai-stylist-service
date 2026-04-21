use rust_decimal::Decimal;

use crate::domain::error::ServiceError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PositiveMeasurement(Decimal);

impl PositiveMeasurement {
    pub fn parse(value: Decimal, label: &str) -> Result<Self, ServiceError> {
        if value <= Decimal::ZERO {
            return Err(ServiceError::BadRequest(format!(
                "{label} must be positive"
            )));
        }

        Ok(Self(value))
    }

    pub fn value(&self) -> Decimal {
        self.0
    }

    pub fn into_inner(self) -> Decimal {
        self.0
    }
}

impl From<PositiveMeasurement> for Decimal {
    fn from(value: PositiveMeasurement) -> Self {
        value.into_inner()
    }
}
