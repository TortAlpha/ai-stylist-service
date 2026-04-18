use rust_decimal_macros::dec;

use product_service::domain::category::SizeGroup;
use product_service::domain::error::ServiceError;
use product_service::domain::request_dto::product_details::SizeInput;
use product_service::service::product::validate_size::validate_size_input;

fn empty_size() -> SizeInput {
    SizeInput {
        size_value: None,
        size_value2: None,
        size_system: None,
        measurement_cm: None,
    }
}

// ── Letter ─────────────────────────────────────────────────

#[test]
fn letter_valid() {
    let s = SizeInput {
        size_value: Some("M".into()),
        ..empty_size()
    };
    assert!(validate_size_input(&s, &SizeGroup::Letter).is_ok());
}

#[test]
fn letter_requires_size_value() {
    let err = validate_size_input(&empty_size(), &SizeGroup::Letter).unwrap_err();
    assert!(matches!(err, ServiceError::BadRequest(_)));
}

#[test]
fn letter_forbids_size_value2() {
    let s = SizeInput {
        size_value: Some("M".into()),
        size_value2: Some("L".into()),
        ..empty_size()
    };
    let err = validate_size_input(&s, &SizeGroup::Letter).unwrap_err();
    assert!(matches!(err, ServiceError::BadRequest(_)));
}

#[test]
fn letter_forbids_size_system() {
    let s = SizeInput {
        size_value: Some("M".into()),
        size_system: Some(product_service::domain::size_info::SizeSystem::EU),
        ..empty_size()
    };
    let err = validate_size_input(&s, &SizeGroup::Letter).unwrap_err();
    assert!(matches!(err, ServiceError::BadRequest(_)));
}

#[test]
fn letter_forbids_measurement() {
    let s = SizeInput {
        size_value: Some("M".into()),
        measurement_cm: Some(dec!(50)),
        ..empty_size()
    };
    let err = validate_size_input(&s, &SizeGroup::Letter).unwrap_err();
    assert!(matches!(err, ServiceError::BadRequest(_)));
}

// ── LetterOrNumeric ────────────────────────────────────────

#[test]
fn letter_or_numeric_valid_letter() {
    let s = SizeInput {
        size_value: Some("XL".into()),
        ..empty_size()
    };
    assert!(validate_size_input(&s, &SizeGroup::LetterOrNumeric).is_ok());
}

#[test]
fn letter_or_numeric_valid_with_system() {
    let s = SizeInput {
        size_value: Some("42".into()),
        size_system: Some(product_service::domain::size_info::SizeSystem::EU),
        ..empty_size()
    };
    assert!(validate_size_input(&s, &SizeGroup::LetterOrNumeric).is_ok());
}

#[test]
fn letter_or_numeric_requires_size_value() {
    let err = validate_size_input(&empty_size(), &SizeGroup::LetterOrNumeric).unwrap_err();
    assert!(matches!(err, ServiceError::BadRequest(_)));
}

#[test]
fn letter_or_numeric_forbids_size_value2() {
    let s = SizeInput {
        size_value: Some("M".into()),
        size_value2: Some("32".into()),
        ..empty_size()
    };
    let err = validate_size_input(&s, &SizeGroup::LetterOrNumeric).unwrap_err();
    assert!(matches!(err, ServiceError::BadRequest(_)));
}

// ── WaistLength ────────────────────────────────────────────

#[test]
fn waist_length_valid() {
    let s = SizeInput {
        size_value: Some("32".into()),
        size_value2: Some("34".into()),
        ..empty_size()
    };
    assert!(validate_size_input(&s, &SizeGroup::WaistLength).is_ok());
}

#[test]
fn waist_length_requires_size_value() {
    let s = SizeInput {
        size_value2: Some("34".into()),
        ..empty_size()
    };
    let err = validate_size_input(&s, &SizeGroup::WaistLength).unwrap_err();
    assert!(matches!(err, ServiceError::BadRequest(_)));
}

#[test]
fn waist_length_forbids_size_system() {
    let s = SizeInput {
        size_value: Some("32".into()),
        size_system: Some(product_service::domain::size_info::SizeSystem::US),
        ..empty_size()
    };
    let err = validate_size_input(&s, &SizeGroup::WaistLength).unwrap_err();
    assert!(matches!(err, ServiceError::BadRequest(_)));
}

// ── Shoe ───────────────────────────────────────────────────

#[test]
fn shoe_valid() {
    let s = SizeInput {
        size_value: Some("42".into()),
        size_system: Some(product_service::domain::size_info::SizeSystem::EU),
        ..empty_size()
    };
    assert!(validate_size_input(&s, &SizeGroup::Shoe).is_ok());
}

#[test]
fn shoe_requires_size_value() {
    let err = validate_size_input(&empty_size(), &SizeGroup::Shoe).unwrap_err();
    assert!(matches!(err, ServiceError::BadRequest(_)));
}

#[test]
fn shoe_forbids_size_value2() {
    let s = SizeInput {
        size_value: Some("42".into()),
        size_value2: Some("43".into()),
        ..empty_size()
    };
    let err = validate_size_input(&s, &SizeGroup::Shoe).unwrap_err();
    assert!(matches!(err, ServiceError::BadRequest(_)));
}

// ── Ring ───────────────────────────────────────────────────

#[test]
fn ring_valid() {
    let s = SizeInput {
        size_value: Some("7".into()),
        ..empty_size()
    };
    assert!(validate_size_input(&s, &SizeGroup::Ring).is_ok());
}

#[test]
fn ring_requires_size_value() {
    let err = validate_size_input(&empty_size(), &SizeGroup::Ring).unwrap_err();
    assert!(matches!(err, ServiceError::BadRequest(_)));
}

// ── MeasurementCm ──────────────────────────────────────────

#[test]
fn measurement_cm_valid() {
    let s = SizeInput {
        measurement_cm: Some(dec!(58.5)),
        ..empty_size()
    };
    assert!(validate_size_input(&s, &SizeGroup::MeasurementCm).is_ok());
}

#[test]
fn measurement_cm_requires_measurement() {
    let err = validate_size_input(&empty_size(), &SizeGroup::MeasurementCm).unwrap_err();
    assert!(matches!(err, ServiceError::BadRequest(_)));
}

#[test]
fn measurement_cm_forbids_size_value() {
    let s = SizeInput {
        size_value: Some("M".into()),
        measurement_cm: Some(dec!(50)),
        ..empty_size()
    };
    let err = validate_size_input(&s, &SizeGroup::MeasurementCm).unwrap_err();
    assert!(matches!(err, ServiceError::BadRequest(_)));
}

#[test]
fn measurement_cm_rejects_zero() {
    let s = SizeInput {
        measurement_cm: Some(dec!(0)),
        ..empty_size()
    };
    let err = validate_size_input(&s, &SizeGroup::MeasurementCm).unwrap_err();
    assert!(matches!(err, ServiceError::BadRequest(_)));
}

#[test]
fn measurement_cm_rejects_negative() {
    let s = SizeInput {
        measurement_cm: Some(dec!(-10)),
        ..empty_size()
    };
    let err = validate_size_input(&s, &SizeGroup::MeasurementCm).unwrap_err();
    assert!(matches!(err, ServiceError::BadRequest(_)));
}

// ── Hat ────────────────────────────────────────────────────

#[test]
fn hat_valid_with_value() {
    let s = SizeInput {
        size_value: Some("L".into()),
        ..empty_size()
    };
    assert!(validate_size_input(&s, &SizeGroup::Hat).is_ok());
}

#[test]
fn hat_valid_with_measurement() {
    let s = SizeInput {
        measurement_cm: Some(dec!(58)),
        ..empty_size()
    };
    assert!(validate_size_input(&s, &SizeGroup::Hat).is_ok());
}

#[test]
fn hat_valid_empty() {
    assert!(validate_size_input(&empty_size(), &SizeGroup::Hat).is_ok());
}

#[test]
fn hat_forbids_size_value2() {
    let s = SizeInput {
        size_value2: Some("x".into()),
        ..empty_size()
    };
    let err = validate_size_input(&s, &SizeGroup::Hat).unwrap_err();
    assert!(matches!(err, ServiceError::BadRequest(_)));
}

// ── Dimensions ─────────────────────────────────────────────

#[test]
fn dimensions_valid_empty() {
    assert!(validate_size_input(&empty_size(), &SizeGroup::Dimensions).is_ok());
}

#[test]
fn dimensions_forbids_all_fields() {
    let s = SizeInput {
        size_value: Some("M".into()),
        ..empty_size()
    };
    let err = validate_size_input(&s, &SizeGroup::Dimensions).unwrap_err();
    assert!(matches!(err, ServiceError::BadRequest(_)));
}

// ── OneSize ────────────────────────────────────────────────

#[test]
fn one_size_valid_empty() {
    assert!(validate_size_input(&empty_size(), &SizeGroup::OneSize).is_ok());
}

#[test]
fn one_size_forbids_size_value() {
    let s = SizeInput {
        size_value: Some("OS".into()),
        ..empty_size()
    };
    let err = validate_size_input(&s, &SizeGroup::OneSize).unwrap_err();
    assert!(matches!(err, ServiceError::BadRequest(_)));
}

#[test]
fn one_size_forbids_measurement() {
    let s = SizeInput {
        measurement_cm: Some(dec!(10)),
        ..empty_size()
    };
    let err = validate_size_input(&s, &SizeGroup::OneSize).unwrap_err();
    assert!(matches!(err, ServiceError::BadRequest(_)));
}
