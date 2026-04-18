use rust_decimal_macros::dec;

use product_service::domain::category::ProductType;
use product_service::domain::error::ServiceError;
use product_service::domain::product_details::{ProductCondition, TypeDetailsInput};
use product_service::domain::request_dto::product_details::{
    CreateProductDetailsRequest, SizeInput,
};
use product_service::service::product::validate_details::{
    validate_common_details, validate_type_details, validate_type_matches_category,
};

// ── validate_type_matches_category ─────────────────────────

#[test]
fn type_matches_category_clothing() {
    let input = TypeDetailsInput::Clothing { fit: None };
    assert!(validate_type_matches_category(&input, &ProductType::Clothing).is_ok());
}

#[test]
fn type_matches_category_footwear() {
    let input = TypeDetailsInput::Footwear {
        shoe_width: None,
        insole_length_cm: None,
    };
    assert!(validate_type_matches_category(&input, &ProductType::Footwear).is_ok());
}

#[test]
fn type_matches_category_bags() {
    let input = TypeDetailsInput::Bags {
        width_cm: None,
        height_cm: None,
        depth_cm: None,
        handle_type: None,
        bag_size_label: None,
    };
    assert!(validate_type_matches_category(&input, &ProductType::Bags).is_ok());
}

#[test]
fn type_matches_category_jewelry() {
    let input = TypeDetailsInput::Jewelry {
        metal: None,
        stone: None,
        clasp_type: None,
    };
    assert!(validate_type_matches_category(&input, &ProductType::Jewelry).is_ok());
}

#[test]
fn type_matches_category_accessories() {
    let input = TypeDetailsInput::Accessories;
    assert!(validate_type_matches_category(&input, &ProductType::Accessories).is_ok());
}

#[test]
fn type_mismatch_clothing_vs_footwear() {
    let input = TypeDetailsInput::Clothing { fit: None };
    let err = validate_type_matches_category(&input, &ProductType::Footwear).unwrap_err();
    assert!(matches!(err, ServiceError::BadRequest(_)));
}

#[test]
fn type_mismatch_bags_vs_jewelry() {
    let input = TypeDetailsInput::Bags {
        width_cm: None,
        height_cm: None,
        depth_cm: None,
        handle_type: None,
        bag_size_label: None,
    };
    let err = validate_type_matches_category(&input, &ProductType::Jewelry).unwrap_err();
    assert!(matches!(err, ServiceError::BadRequest(_)));
}

// ── validate_type_details ──────────────────────────────────

#[test]
fn clothing_details_always_valid() {
    let input = TypeDetailsInput::Clothing { fit: None };
    assert!(validate_type_details(&input).is_ok());
}

#[test]
fn footwear_valid_insole() {
    let input = TypeDetailsInput::Footwear {
        shoe_width: None,
        insole_length_cm: Some(dec!(27.5)),
    };
    assert!(validate_type_details(&input).is_ok());
}

#[test]
fn footwear_rejects_zero_insole() {
    let input = TypeDetailsInput::Footwear {
        shoe_width: None,
        insole_length_cm: Some(dec!(0)),
    };
    let err = validate_type_details(&input).unwrap_err();
    assert!(matches!(err, ServiceError::BadRequest(_)));
}

#[test]
fn footwear_rejects_negative_insole() {
    let input = TypeDetailsInput::Footwear {
        shoe_width: None,
        insole_length_cm: Some(dec!(-1)),
    };
    let err = validate_type_details(&input).unwrap_err();
    assert!(matches!(err, ServiceError::BadRequest(_)));
}

#[test]
fn footwear_none_insole_ok() {
    let input = TypeDetailsInput::Footwear {
        shoe_width: None,
        insole_length_cm: None,
    };
    assert!(validate_type_details(&input).is_ok());
}

#[test]
fn bags_valid_dimensions() {
    let input = TypeDetailsInput::Bags {
        width_cm: Some(dec!(30)),
        height_cm: Some(dec!(40)),
        depth_cm: Some(dec!(15)),
        handle_type: None,
        bag_size_label: None,
    };
    assert!(validate_type_details(&input).is_ok());
}

#[test]
fn bags_rejects_zero_width() {
    let input = TypeDetailsInput::Bags {
        width_cm: Some(dec!(0)),
        height_cm: None,
        depth_cm: None,
        handle_type: None,
        bag_size_label: None,
    };
    let err = validate_type_details(&input).unwrap_err();
    assert!(matches!(err, ServiceError::BadRequest(_)));
}

#[test]
fn bags_rejects_negative_height() {
    let input = TypeDetailsInput::Bags {
        width_cm: None,
        height_cm: Some(dec!(-5)),
        depth_cm: None,
        handle_type: None,
        bag_size_label: None,
    };
    let err = validate_type_details(&input).unwrap_err();
    assert!(matches!(err, ServiceError::BadRequest(_)));
}

#[test]
fn bags_rejects_negative_depth() {
    let input = TypeDetailsInput::Bags {
        width_cm: None,
        height_cm: None,
        depth_cm: Some(dec!(-0.5)),
        handle_type: None,
        bag_size_label: None,
    };
    let err = validate_type_details(&input).unwrap_err();
    assert!(matches!(err, ServiceError::BadRequest(_)));
}

#[test]
fn jewelry_always_valid() {
    let input = TypeDetailsInput::Jewelry {
        metal: Some("gold".into()),
        stone: Some("diamond".into()),
        clasp_type: None,
    };
    assert!(validate_type_details(&input).is_ok());
}

#[test]
fn accessories_always_valid() {
    let input = TypeDetailsInput::Accessories;
    assert!(validate_type_details(&input).is_ok());
}

// ── validate_common_details ────────────────────────────────

fn base_details() -> CreateProductDetailsRequest {
    CreateProductDetailsRequest {
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
    }
}

#[test]
fn common_details_minimal_valid() {
    assert!(validate_common_details(&base_details()).is_ok());
}

#[test]
fn common_details_valid_year() {
    let mut d = base_details();
    d.year_of_release = Some(2024);
    assert!(validate_common_details(&d).is_ok());
}

#[test]
fn common_details_rejects_year_too_old() {
    let mut d = base_details();
    d.year_of_release = Some(1899);
    let err = validate_common_details(&d).unwrap_err();
    assert!(matches!(err, ServiceError::BadRequest(_)));
}

#[test]
fn common_details_rejects_future_year() {
    let mut d = base_details();
    d.year_of_release = Some(2099);
    let err = validate_common_details(&d).unwrap_err();
    assert!(matches!(err, ServiceError::BadRequest(_)));
}

#[test]
fn common_details_collab_requires_name() {
    let mut d = base_details();
    d.is_collab = Some(true);
    d.collab_name = None;
    let err = validate_common_details(&d).unwrap_err();
    assert!(matches!(err, ServiceError::BadRequest(_)));
}

#[test]
fn common_details_collab_rejects_blank_name() {
    let mut d = base_details();
    d.is_collab = Some(true);
    d.collab_name = Some("   ".into());
    let err = validate_common_details(&d).unwrap_err();
    assert!(matches!(err, ServiceError::BadRequest(_)));
}

#[test]
fn common_details_collab_with_name_ok() {
    let mut d = base_details();
    d.is_collab = Some(true);
    d.collab_name = Some("Off-White".into());
    assert!(validate_common_details(&d).is_ok());
}

#[test]
fn common_details_not_collab_ignores_name() {
    let mut d = base_details();
    d.is_collab = Some(false);
    d.collab_name = None;
    assert!(validate_common_details(&d).is_ok());
}
