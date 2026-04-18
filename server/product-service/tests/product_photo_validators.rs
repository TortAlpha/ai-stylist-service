use product_service::domain::error::ServiceError;
use product_service::domain::product_photo::UploadFile;
use product_service::service::utils::validators::{
    validate_files_not_empty, validate_upload_file, validate_upload_files,
};

#[test]
fn validate_files_not_empty_accepts_non_empty() {
    validate_files_not_empty(1).expect("non-empty should pass");
}

#[test]
fn validate_files_not_empty_rejects_empty() {
    let err = validate_files_not_empty(0).expect_err("empty list should fail");
    assert!(matches!(err, ServiceError::BadRequest(_)));
}

#[test]
fn validate_upload_file_accepts_jpeg_and_sets_extension() {
    let file = UploadFile {
        data: vec![1, 2, 3],
        content_type: "image/jpeg".into(),
    };

    let validated = validate_upload_file(file).expect("jpeg should pass");

    assert_eq!(validated.extension, "jpg");
    assert_eq!(validated.content_type, "image/jpeg");
    assert_eq!(validated.data, vec![1, 2, 3]);
}

#[test]
fn validate_upload_file_rejects_unsupported_content_type() {
    let file = UploadFile {
        data: vec![1, 2, 3],
        content_type: "image/gif".into(),
    };

    let err = validate_upload_file(file).expect_err("gif should fail");
    assert!(matches!(err, ServiceError::BadRequest(_)));
}

#[test]
fn validate_upload_file_rejects_oversized() {
    let file = UploadFile {
        data: vec![0u8; 11 * 1024 * 1024],
        content_type: "image/png".into(),
    };

    let err = validate_upload_file(file).expect_err("oversized file should fail");
    assert!(matches!(err, ServiceError::BadRequest(_)));
}

#[test]
fn validate_upload_files_validates_whole_list() {
    let files = vec![
        UploadFile {
            data: vec![1, 2, 3],
            content_type: "image/png".into(),
        },
        UploadFile {
            data: vec![4, 5, 6],
            content_type: "image/webp".into(),
        },
    ];

    let validated = validate_upload_files(files).expect("list should pass");

    assert_eq!(validated.len(), 2);
    assert_eq!(validated[0].extension, "png");
    assert_eq!(validated[1].extension, "webp");
}

#[test]
fn validate_upload_files_fails_when_any_file_invalid() {
    let files = vec![
        UploadFile {
            data: vec![1, 2, 3],
            content_type: "image/png".into(),
        },
        UploadFile {
            data: vec![7, 8],
            content_type: "application/pdf".into(),
        },
    ];

    let err = validate_upload_files(files).expect_err("invalid file should fail whole list");
    assert!(matches!(err, ServiceError::BadRequest(_)));
}
