use product_service::domain::error::ServiceError;
use product_service::service::utils::validators::validate_positive_i32;

#[test]
fn validate_positive_i32_accepts_positive_value() {
    let value = validate_positive_i32(7, "id").expect("should accept positive");
    assert_eq!(value, 7);
}

#[test]
fn validate_positive_i32_rejects_non_positive_value() {
    let err = validate_positive_i32(0, "id").expect_err("should reject zero");
    assert!(matches!(err, ServiceError::BadRequest(_)));
}
