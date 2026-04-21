use rust_decimal::Decimal;

use crate::domain::error::ServiceError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PurchasePrice(Decimal);

impl PurchasePrice {
    pub fn parse(value: Decimal) -> Result<Self, ServiceError> {
        if value < Decimal::ZERO {
            return Err(ServiceError::BadRequest(
                "purchase price must not be negative".into(),
            ));
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

impl From<PurchasePrice> for Decimal {
    fn from(value: PurchasePrice) -> Self {
        value.into_inner()
    }
}
