use crate::domain::category::SizeGroup;
use crate::domain::error::ServiceError;
use crate::domain::request_dto::product_details::SizeInput;
use crate::domain::value_objects::PositiveMeasurement;

type Result<T> = std::result::Result<T, ServiceError>;

pub fn validate_size_input(input: &SizeInput, size_group: &SizeGroup) -> Result<()> {
    match size_group {
        SizeGroup::Letter => {
            require_field(&input.size_value, "size_value", size_group)?;
            forbid_field(&input.size_value2, "size_value2", size_group)?;
            forbid_field(&input.size_system, "size_system", size_group)?;
            forbid_measurement(&input.measurement_cm, size_group)?;
        }
        SizeGroup::LetterOrNumeric => {
            require_field(&input.size_value, "size_value", size_group)?;
            forbid_field(&input.size_value2, "size_value2", size_group)?;
            forbid_measurement(&input.measurement_cm, size_group)?;
        }
        SizeGroup::WaistLength => {
            require_field(&input.size_value, "size_value", size_group)?;
            forbid_field(&input.size_system, "size_system", size_group)?;
            forbid_measurement(&input.measurement_cm, size_group)?;
        }
        SizeGroup::Shoe | SizeGroup::Ring => {
            require_field(&input.size_value, "size_value", size_group)?;
            forbid_field(&input.size_value2, "size_value2", size_group)?;
            forbid_measurement(&input.measurement_cm, size_group)?;
        }
        SizeGroup::MeasurementCm => {
            forbid_field(&input.size_value, "size_value", size_group)?;
            forbid_field(&input.size_value2, "size_value2", size_group)?;
            forbid_field(&input.size_system, "size_system", size_group)?;
            require_measurement(&input.measurement_cm, size_group)?;
        }
        SizeGroup::Hat => {
            forbid_field(&input.size_value2, "size_value2", size_group)?;
            forbid_field(&input.size_system, "size_system", size_group)?;
        }
        SizeGroup::Dimensions | SizeGroup::OneSize => {
            forbid_field(&input.size_value, "size_value", size_group)?;
            forbid_field(&input.size_value2, "size_value2", size_group)?;
            forbid_field(&input.size_system, "size_system", size_group)?;
            forbid_measurement(&input.measurement_cm, size_group)?;
        }
    }

    if let Some(cm) = input.measurement_cm {
        PositiveMeasurement::parse(cm, "measurement_cm")?;
    }

    Ok(())
}

fn require_field<T>(field: &Option<T>, name: &str, group: &SizeGroup) -> Result<()> {
    if field.is_none() {
        return Err(ServiceError::BadRequest(format!(
            "{name} is required for size group '{group:?}'"
        )));
    }
    Ok(())
}

fn forbid_field<T>(field: &Option<T>, name: &str, group: &SizeGroup) -> Result<()> {
    if field.is_some() {
        return Err(ServiceError::BadRequest(format!(
            "{name} is not applicable for size group '{group:?}'"
        )));
    }
    Ok(())
}

fn require_measurement(field: &Option<rust_decimal::Decimal>, group: &SizeGroup) -> Result<()> {
    require_field(field, "measurement_cm", group)
}

fn forbid_measurement(field: &Option<rust_decimal::Decimal>, group: &SizeGroup) -> Result<()> {
    forbid_field(field, "measurement_cm", group)
}
