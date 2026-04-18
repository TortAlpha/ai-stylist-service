use product_service::domain::error::ServiceError;
use product_service::domain::value_objects::PurchaseLocationName;

#[test]
fn purchase_location_name_parse_trims_value() {
    let name = PurchaseLocationName::parse("  Novi Sad  ").expect("name should be valid");
    assert_eq!(name.as_str(), "Novi Sad");
}

#[test]
fn purchase_location_name_parse_rejects_blank_value() {
    let err = PurchaseLocationName::parse("   ").expect_err("blank value should be rejected");
    assert!(matches!(err, ServiceError::BadRequest(_)));
}

#[test]
fn purchase_location_name_parse_rejects_too_long_value() {
    let value = "a".repeat(201);
    let err = PurchaseLocationName::parse(&value).expect_err("too long value should be rejected");
    assert!(matches!(err, ServiceError::BadRequest(_)));
}
