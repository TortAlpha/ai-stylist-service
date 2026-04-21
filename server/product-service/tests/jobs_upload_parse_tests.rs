//! Unit tests for `jobs::upload_product_images::parse` — pure function,
//! no DB / no S3 / no ProductPhotoService required.

use serde_json::json;
use uuid::Uuid;

use product_service::jobs::JobRow;
use product_service::jobs::upload_product_images::parse;

const PRODUCT_ID: &str = "11111111-2222-3333-4444-555555555555";

fn job(
    payload: serde_json::Value,
    blobs: Option<Vec<Vec<u8>>>,
    types: Option<Vec<String>>,
) -> JobRow {
    JobRow::for_test("upload_product_images", payload, blobs, types)
}

#[test]
fn parses_happy_path() {
    let j = job(
        json!({ "product_id": PRODUCT_ID }),
        Some(vec![vec![1, 2, 3], vec![4, 5]]),
        Some(vec!["image/jpeg".into(), "image/png".into()]),
    );

    let (id, files) = parse(&j).expect("should parse");

    assert_eq!(id, Uuid::parse_str(PRODUCT_ID).unwrap());
    assert_eq!(files.len(), 2);
    assert_eq!(files[0].data, vec![1, 2, 3]);
    assert_eq!(files[0].content_type, "image/jpeg");
    assert_eq!(files[1].data, vec![4, 5]);
    assert_eq!(files[1].content_type, "image/png");
}

#[test]
fn rejects_missing_product_id() {
    let j = job(
        json!({}),
        Some(vec![vec![1]]),
        Some(vec!["image/jpeg".into()]),
    );
    let err = parse(&j).unwrap_err();
    assert!(err.contains("product_id"), "got: {err}");
}

#[test]
fn rejects_non_string_product_id() {
    let j = job(
        json!({ "product_id": 42 }),
        Some(vec![vec![1]]),
        Some(vec!["image/jpeg".into()]),
    );
    let err = parse(&j).unwrap_err();
    assert!(err.contains("product_id"), "got: {err}");
}

#[test]
fn rejects_invalid_uuid() {
    let j = job(
        json!({ "product_id": "not-a-uuid" }),
        Some(vec![vec![1]]),
        Some(vec!["image/jpeg".into()]),
    );
    let err = parse(&j).unwrap_err();
    assert!(err.contains("invalid uuid"), "got: {err}");
}

#[test]
fn rejects_null_blobs() {
    let j = job(
        json!({ "product_id": PRODUCT_ID }),
        None,
        Some(vec!["image/jpeg".into()]),
    );
    let err = parse(&j).unwrap_err();
    assert!(err.contains("blobs"), "got: {err}");
}

#[test]
fn rejects_null_blob_types() {
    let j = job(
        json!({ "product_id": PRODUCT_ID }),
        Some(vec![vec![1]]),
        None,
    );
    let err = parse(&j).unwrap_err();
    assert!(err.contains("blob_types"), "got: {err}");
}

#[test]
fn rejects_length_mismatch() {
    let j = job(
        json!({ "product_id": PRODUCT_ID }),
        Some(vec![vec![1], vec![2]]),
        Some(vec!["image/jpeg".into()]),
    );
    let err = parse(&j).unwrap_err();
    assert!(err.contains("length mismatch"), "got: {err}");
}

#[test]
fn rejects_empty_files() {
    let j = job(
        json!({ "product_id": PRODUCT_ID }),
        Some(vec![]),
        Some(vec![]),
    );
    let err = parse(&j).unwrap_err();
    assert!(err.contains("no files"), "got: {err}");
}
