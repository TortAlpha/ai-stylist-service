use crate::domain::error::ServiceError;
use crate::domain::request_dto::product::{CreateProductRequest, UpdateProductRequest};
use crate::domain::value_objects::{CurrencyCode, ProductName, PurchasePrice};

type Result<T> = std::result::Result<T, ServiceError>;

pub fn validate_create_product(req: &CreateProductRequest) -> Result<()> {
    ProductName::parse(&req.name)?;

    if let Some(ref price) = req.purchase_price {
        PurchasePrice::parse(*price)?;
    }

    if let Some(ref currency) = req.currency {
        CurrencyCode::parse(currency)?;
    }

    Ok(())
}

pub fn validate_update_product(req: &UpdateProductRequest) -> Result<()> {
    if let Some(ref name) = req.name {
        ProductName::parse(name)?;
    }

    if let Some(ref price) = req.purchase_price {
        PurchasePrice::parse(*price)?;
    }

    if let Some(ref currency) = req.currency {
        CurrencyCode::parse(currency)?;
    }

    Ok(())
}
