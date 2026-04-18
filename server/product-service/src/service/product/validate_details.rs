use crate::domain::category::ProductType;
use crate::domain::error::ServiceError;
use crate::domain::product_details::TypeDetailsInput;
use crate::domain::request_dto::product_details::CreateProductDetailsRequest;
use crate::domain::value_objects::{PositiveMeasurement, YearOfRelease};

type Result<T> = std::result::Result<T, ServiceError>;

pub fn validate_type_matches_category(
    input: &TypeDetailsInput,
    expected: &ProductType,
) -> Result<()> {
    if &input.product_type() != expected {
        return Err(ServiceError::BadRequest(format!(
            "type_details product_type '{:?}' does not match category product_type '{:?}'",
            input.product_type(),
            expected,
        )));
    }
    Ok(())
}

pub fn validate_type_details(input: &TypeDetailsInput) -> Result<()> {
    match input {
        TypeDetailsInput::Clothing { .. } => {}
        TypeDetailsInput::Footwear {
            insole_length_cm, ..
        } => {
            if let Some(v) = insole_length_cm {
                PositiveMeasurement::parse(*v, "insole_length_cm")?;
            }
        }
        TypeDetailsInput::Bags {
            width_cm,
            height_cm,
            depth_cm,
            ..
        } => {
            if let Some(v) = width_cm {
                PositiveMeasurement::parse(*v, "bag width_cm")?;
            }
            if let Some(v) = height_cm {
                PositiveMeasurement::parse(*v, "bag height_cm")?;
            }
            if let Some(v) = depth_cm {
                PositiveMeasurement::parse(*v, "bag depth_cm")?;
            }
        }
        TypeDetailsInput::Jewelry { .. } | TypeDetailsInput::Accessories => {}
    }
    Ok(())
}

pub fn validate_common_details(input: &CreateProductDetailsRequest) -> Result<()> {
    if let Some(year) = input.year_of_release {
        YearOfRelease::parse(year)?;
    }

    if input.is_collab.unwrap_or(false) {
        match &input.collab_name {
            Some(name) if !name.trim().is_empty() => {}
            _ => {
                return Err(ServiceError::BadRequest(
                    "collab_name is required when is_collab is true".into(),
                ));
            }
        }
    }

    Ok(())
}
