use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use product_service::domain::error::ServiceError;
use product_service::domain::product_details::{ProductCondition, TypeDetailsInput};
use product_service::domain::request_dto::product::{CreateProductRequest, UpdateProductRequest};
use product_service::domain::request_dto::product_details::{
    CreateProductDetailsRequest, SizeInput, UpdateProductDetailsRequest,
};
use product_service::service::product::validate_product::{
    validate_create_product, validate_update_product,
};

fn minimal_create_req() -> CreateProductRequest {
    CreateProductRequest {
        name: "Nike Air Max 90".to_string(),
        brand_id: 1,
        category_id: 1,
        status: None,
        purchase_price: None,
        purchase_location_id: None,
        currency: None,
        ai_notes: None,
        details: CreateProductDetailsRequest {
            condition: ProductCondition::Excellent,
            material: None,
            color: None,
            year_of_release: None,
            is_vintage: None,
            is_collab: None,
            collab_name: None,
            is_limited_edition: None,
            special_notes: None,
            size: SizeInput {
                size_value: Some("M".into()),
                size_value2: None,
                size_system: None,
                measurement_cm: None,
            },
            type_details: TypeDetailsInput::Clothing { fit: None },
        },
        style_tag_ids: vec![],
        vibe_tag_ids: vec![],
        season_ids: vec![],
    }
}

fn minimal_update_req() -> UpdateProductRequest {
    UpdateProductRequest {
        name: None,
        brand_id: None,
        category_id: None,
        status: None,
        purchase_price: None,
        purchase_location_id: None,
        currency: None,
        ai_notes: None,
        details: None,
        style_tag_ids: None,
        vibe_tag_ids: None,
        season_ids: None,
        expected_version: 1,
    }
}

// ── Create: name ────────────────────────────────────────

#[test]
fn create_accepts_valid_name() {
    let req = minimal_create_req();
    assert!(validate_create_product(&req).is_ok());
}

#[test]
fn create_rejects_blank_name() {
    let mut req = minimal_create_req();
    req.name = "   ".to_string();
    let err = validate_create_product(&req).unwrap_err();
    assert!(matches!(err, ServiceError::BadRequest(_)));
}

#[test]
fn create_rejects_too_long_name() {
    let mut req = minimal_create_req();
    req.name = "a".repeat(501);
    let err = validate_create_product(&req).unwrap_err();
    assert!(matches!(err, ServiceError::BadRequest(_)));
}

// ── Create: purchase_price ──────────────────────────────

#[test]
fn create_accepts_zero_price() {
    let mut req = minimal_create_req();
    req.purchase_price = Some(dec!(0));
    assert!(validate_create_product(&req).is_ok());
}

#[test]
fn create_accepts_positive_price() {
    let mut req = minimal_create_req();
    req.purchase_price = Some(dec!(199.99));
    assert!(validate_create_product(&req).is_ok());
}

#[test]
fn create_rejects_negative_price() {
    let mut req = minimal_create_req();
    req.purchase_price = Some(dec!(-1));
    let err = validate_create_product(&req).unwrap_err();
    assert!(matches!(err, ServiceError::BadRequest(_)));
}

#[test]
fn create_accepts_none_price() {
    let req = minimal_create_req();
    assert!(req.purchase_price.is_none());
    assert!(validate_create_product(&req).is_ok());
}

// ── Create: currency ────────────────────────────────────

#[test]
fn create_accepts_valid_currencies() {
    for code in &["RSD", "EUR", "USD", "GBP", "rsd", " eur "] {
        let mut req = minimal_create_req();
        req.currency = Some(code.to_string());
        assert!(
            validate_create_product(&req).is_ok(),
            "should accept {code}"
        );
    }
}

#[test]
fn create_rejects_unsupported_currency() {
    let mut req = minimal_create_req();
    req.currency = Some("JPY".to_string());
    let err = validate_create_product(&req).unwrap_err();
    assert!(matches!(err, ServiceError::BadRequest(_)));
}

// ── Update: partial validation ──────────────────────────

#[test]
fn update_empty_passes() {
    let req = minimal_update_req();
    assert!(validate_update_product(&req).is_ok());
}

#[test]
fn update_rejects_blank_name() {
    let mut req = minimal_update_req();
    req.name = Some("".to_string());
    let err = validate_update_product(&req).unwrap_err();
    assert!(matches!(err, ServiceError::BadRequest(_)));
}

#[test]
fn update_rejects_negative_price() {
    let mut req = minimal_update_req();
    req.purchase_price = Some(dec!(-5));
    let err = validate_update_product(&req).unwrap_err();
    assert!(matches!(err, ServiceError::BadRequest(_)));
}

#[test]
fn update_rejects_bad_currency() {
    let mut req = minimal_update_req();
    req.currency = Some("BTC".to_string());
    let err = validate_update_product(&req).unwrap_err();
    assert!(matches!(err, ServiceError::BadRequest(_)));
}

#[test]
fn update_accepts_valid_fields() {
    let mut req = minimal_update_req();
    req.name = Some("Updated Name".to_string());
    req.purchase_price = Some(dec!(100));
    req.currency = Some("EUR".to_string());
    assert!(validate_update_product(&req).is_ok());
}
