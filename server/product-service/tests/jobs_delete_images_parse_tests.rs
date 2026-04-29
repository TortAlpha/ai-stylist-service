//! Unit tests for `jobs::delete_product_images::parse`.

use serde_json::json;
use uuid::Uuid;

use product_service::jobs::JobRow;
use product_service::jobs::delete_product_images::parse;

const PRODUCT_ID: &str = "11111111-2222-3333-4444-555555555555";

fn job(payload: serde_json::Value) -> JobRow {
    JobRow::for_test("delete_product_images", payload, None, None)
}

#[test]
fn parses_happy_path() {
    let j = job(json!({ "product_id": PRODUCT_ID }));

    let id = parse(&j).expect("should parse");

    assert_eq!(id, Uuid::parse_str(PRODUCT_ID).unwrap());
}

#[test]
fn rejects_missing_product_id() {
    let j = job(json!({}));

    let err = parse(&j).unwrap_err();

    assert!(err.contains("product_id"), "got: {err}");
}

#[test]
fn rejects_invalid_uuid() {
    let j = job(json!({ "product_id": "not-a-uuid" }));

    let err = parse(&j).unwrap_err();

    assert!(err.contains("invalid uuid"), "got: {err}");
}
