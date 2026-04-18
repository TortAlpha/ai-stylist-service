use crate::domain::error::ServiceError;

pub fn validate_positive_i32(value: i32, field_name: &str) -> Result<i32, ServiceError> {
    if value <= 0 {
        return Err(ServiceError::BadRequest(format!(
            "{field_name} must be positive"
        )));
    }

    Ok(value)
}
