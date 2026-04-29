use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use chrono::Utc;
use rust_decimal_macros::dec;
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

use product_service::domain::category::{Category, Gender, ProductType, SizeGroup};
use product_service::domain::error::ServiceError;
use product_service::domain::product::ProductFull;
use product_service::domain::product_details::{ProductCondition, TypeDetailsInput};
use product_service::domain::product_photo::UploadFile;
use product_service::domain::request_dto::product::{CreateProductRequest, UpdateProductRequest};
use product_service::domain::request_dto::product_details::{
    CreateProductDetailsRequest, SizeInput, UpdateProductDetailsRequest,
};
use product_service::domain::response_dto::product::ProductFilterOptions;
use product_service::domain::response_dto::product_details::AvailableSizesResponse;
use product_service::domain::utils::query::{
    AvailableSizesQuery, FilterOptionsQuery, ProductListQuery,
};
use product_service::repo::traits::category_repo::CategoryRepository;
use product_service::repo::traits::product_photo_repo::ProductPhotoRepository;
use product_service::repo::traits::product_repo::ProductRepository;
use product_service::service::product::ProductService;
use product_service::service::product_photo_service::ProductPhotoService;
use product_service::storage::error::StorageError;
use product_service::storage::traits::ImageStorage;

fn lazy_pool() -> sqlx::PgPool {
    PgPoolOptions::new()
        .max_connections(1)
        .connect_lazy("postgres://postgres:postgres@127.0.0.1:5432/postgres")
        .expect("lazy pool")
}

fn tiny_png_bytes() -> Vec<u8> {
    use image::{ImageBuffer, Rgba};
    let buf: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::from_fn(8, 8, |_, _| Rgba([255, 0, 0, 255]));
    let mut out = Vec::new();
    image::DynamicImage::ImageRgba8(buf)
        .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
        .expect("encode tiny png");
    out
}

fn test_product_full(id: Uuid) -> ProductFull {
    let now = Utc::now();
    ProductFull {
        id,
        sku: "SKU-001".into(),
        name: "Test Product".into(),
        purchase_price: Some(dec!(100)),
        purchase_location: Some("Belgrade".into()),
        currency: "RSD".into(),
        ai_notes: None,
        category_id: 1,
        image_count: 0,
        version: 1,
        status: "intake".into(),
        product_type: "clothing".into(),
        brand_id: 1,
        brand_name: "Nike".into(),
        brand_tier: "premium".into(),
        category_name: "T-Shirts".into(),
        parent_category: Some("Tops".into()),
        gender: "male".into(),
        material: Some("Cotton".into()),
        condition: Some("excellent".into()),
        color: Some("Black".into()),
        year_of_release: Some(2024),
        is_vintage: Some(false),
        is_collab: Some(false),
        collab_name: None,
        is_limited_edition: Some(false),
        special_notes: None,
        size_group: "letter".into(),
        size_value: Some("M".into()),
        size_value2: None,
        size_system: None,
        measurement_cm: None,
        clothing_fit: Some("regular".into()),
        shoe_width: None,
        insole_length_cm: None,
        bag_width_cm: None,
        bag_height_cm: None,
        bag_depth_cm: None,
        bag_handle_type: None,
        bag_size_label: None,
        jewelry_metal: None,
        jewelry_stone: None,
        jewelry_clasp_type: None,
        style_tags: vec!["streetwear".into()],
        vibe_tags: vec!["casual".into()],
        season_tags: vec!["summer".into()],
        created_at: now,
        updated_at: now,
    }
}

fn test_category_clothing() -> Category {
    Category {
        id: 1,
        name: "T-Shirts".into(),
        code: "tshirts".into(),
        parent_id: Some(10),
        gender: Gender::Male,
        product_type: ProductType::Clothing,
        size_group: SizeGroup::Letter,
    }
}

fn minimal_create_req() -> CreateProductRequest {
    CreateProductRequest {
        name: "Nike Air Max 90".into(),
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

struct ProductRepoStub {
    find_by_id_result: Option<Option<ProductFull>>,
    search_result: Option<(Vec<ProductFull>, i64)>,
    soft_delete_result: Option<bool>,
}

impl ProductRepoStub {
    fn new() -> Self {
        Self {
            find_by_id_result: None,
            search_result: None,
            soft_delete_result: None,
        }
    }

    fn with_find_by_id(result: Option<ProductFull>) -> Self {
        Self {
            find_by_id_result: Some(result),
            search_result: None,
            soft_delete_result: None,
        }
    }

    fn with_search(result: (Vec<ProductFull>, i64)) -> Self {
        Self {
            find_by_id_result: None,
            search_result: Some(result),
            soft_delete_result: None,
        }
    }

    fn with_soft_delete(result: bool) -> Self {
        Self {
            find_by_id_result: None,
            search_result: None,
            soft_delete_result: Some(result),
        }
    }
}

#[async_trait]
impl ProductRepository for ProductRepoStub {
    async fn find_by_id(&self, _id: Uuid) -> Result<Option<ProductFull>, sqlx::Error> {
        match &self.find_by_id_result {
            Some(v) => Ok(v.clone()),
            None => panic!("unexpected call to find_by_id"),
        }
    }

    async fn search_by_query(
        &self,
        _query: &ProductListQuery,
    ) -> Result<(Vec<ProductFull>, i64), sqlx::Error> {
        match &self.search_result {
            Some(v) => Ok(v.clone()),
            None => panic!("unexpected call to search_by_query"),
        }
    }

    async fn create(&self, _req: &CreateProductRequest) -> Result<ProductFull, sqlx::Error> {
        panic!("unexpected call to create")
    }

    async fn create_in_tx(
        &self,
        _tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        _req: &CreateProductRequest,
    ) -> Result<ProductFull, sqlx::Error> {
        panic!("unexpected call to create_in_tx")
    }

    async fn update(
        &self,
        _id: Uuid,
        _req: &UpdateProductRequest,
    ) -> Result<ProductFull, sqlx::Error> {
        panic!("unexpected call to update")
    }

    async fn update_in_tx(
        &self,
        _tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        _id: Uuid,
        _req: &UpdateProductRequest,
    ) -> Result<ProductFull, sqlx::Error> {
        panic!("unexpected call to update_in_tx")
    }

    async fn soft_delete(&self, _id: Uuid, _expected_version: i32) -> Result<bool, sqlx::Error> {
        match self.soft_delete_result {
            Some(v) => Ok(v),
            None => panic!("unexpected call to soft_delete"),
        }
    }

    async fn filter_options(
        &self,
        _query: &FilterOptionsQuery,
    ) -> Result<ProductFilterOptions, sqlx::Error> {
        panic!("unexpected call to filter_options")
    }

    async fn available_sizes(
        &self,
        _query: &AvailableSizesQuery,
    ) -> Result<AvailableSizesResponse, sqlx::Error> {
        panic!("unexpected call to available_sizes")
    }
}

struct CategoryRepoStub {
    find_by_id_result: Option<Option<Category>>,
    has_children_result: Option<bool>,
}

impl CategoryRepoStub {
    fn new() -> Self {
        Self {
            find_by_id_result: None,
            has_children_result: None,
        }
    }

    fn with_category(category: Option<Category>, has_children: Option<bool>) -> Self {
        Self {
            find_by_id_result: Some(category),
            has_children_result: has_children,
        }
    }
}

#[async_trait]
impl CategoryRepository for CategoryRepoStub {
    async fn find_by_id(&self, _id: i32) -> Result<Option<Category>, sqlx::Error> {
        match &self.find_by_id_result {
            Some(v) => Ok(v.clone()),
            None => panic!("unexpected call to category.find_by_id"),
        }
    }

    async fn list_all(&self) -> Result<Vec<Category>, sqlx::Error> {
        panic!("unexpected call to list_all")
    }

    async fn list_by_parent(&self, _parent_id: Option<i32>) -> Result<Vec<Category>, sqlx::Error> {
        panic!("unexpected call to list_by_parent")
    }

    async fn create(
        &self,
        _req: &product_service::domain::request_dto::category::CreateCategoryRequest,
    ) -> Result<Category, sqlx::Error> {
        panic!("unexpected call to category.create")
    }

    async fn update_by_id(
        &self,
        _id: i32,
        _req: &product_service::domain::request_dto::category::UpdateCategoryRequest,
    ) -> Result<Category, sqlx::Error> {
        panic!("unexpected call to category.update_by_id")
    }

    async fn delete_by_id(&self, _id: i32) -> Result<bool, sqlx::Error> {
        panic!("unexpected call to category.delete_by_id")
    }

    async fn has_children(&self, _id: i32) -> Result<bool, sqlx::Error> {
        match self.has_children_result {
            Some(v) => Ok(v),
            None => panic!("unexpected call to category.has_children"),
        }
    }
}

struct ProductPhotoRepoStub {
    product_exists_result: Option<bool>,
    enqueue_images_job_id: Option<i64>,
    enqueue_preview_job_id: Option<i64>,
}

impl ProductPhotoRepoStub {
    fn never_called() -> Self {
        Self {
            product_exists_result: None,
            enqueue_images_job_id: None,
            enqueue_preview_job_id: None,
        }
    }

    fn with_exists(exists: bool) -> Self {
        Self {
            product_exists_result: Some(exists),
            enqueue_images_job_id: None,
            enqueue_preview_job_id: None,
        }
    }

    fn with_queue(exists: bool, images_job_id: i64, preview_job_id: i64) -> Self {
        Self {
            product_exists_result: Some(exists),
            enqueue_images_job_id: Some(images_job_id),
            enqueue_preview_job_id: Some(preview_job_id),
        }
    }
}

#[async_trait]
impl ProductPhotoRepository for ProductPhotoRepoStub {
    async fn product_exists(&self, _id: Uuid) -> Result<bool, sqlx::Error> {
        match self.product_exists_result {
            Some(v) => Ok(v),
            None => panic!("unexpected call to product_exists"),
        }
    }

    async fn enqueue_images_upload(
        &self,
        _product_id: Uuid,
        _files: Vec<UploadFile>,
    ) -> Result<i64, sqlx::Error> {
        match self.enqueue_images_job_id {
            Some(v) => Ok(v),
            None => panic!("unexpected call to enqueue_images_upload"),
        }
    }

    async fn enqueue_preview_upload(
        &self,
        _product_id: Uuid,
        _file: UploadFile,
    ) -> Result<i64, sqlx::Error> {
        match self.enqueue_preview_job_id {
            Some(v) => Ok(v),
            None => panic!("unexpected call to enqueue_preview_upload"),
        }
    }
}

#[derive(Default)]
struct StorageStub {
    image_keys: Vec<String>,
    preview_keys: Vec<String>,
    put_calls: Mutex<Vec<String>>,
    delete_calls: Mutex<Vec<String>>,
}

impl StorageStub {
    fn with_existing(image_keys: Vec<String>, preview_keys: Vec<String>) -> Self {
        Self {
            image_keys,
            preview_keys,
            put_calls: Mutex::new(vec![]),
            delete_calls: Mutex::new(vec![]),
        }
    }

    fn put_calls(&self) -> Vec<String> {
        self.put_calls
            .lock()
            .expect("put_calls mutex poisoned")
            .clone()
    }

    fn delete_calls(&self) -> Vec<String> {
        self.delete_calls
            .lock()
            .expect("delete_calls mutex poisoned")
            .clone()
    }
}

#[async_trait]
impl ImageStorage for StorageStub {
    async fn put_object(
        &self,
        bucket: &str,
        key: &str,
        _data: Vec<u8>,
        content_type: &str,
    ) -> Result<(), StorageError> {
        self.put_calls
            .lock()
            .expect("put_calls mutex poisoned")
            .push(format!("{bucket}:{key}:{content_type}"));
        Ok(())
    }

    async fn delete_object(&self, _bucket: &str, _key: &str) -> Result<(), StorageError> {
        Ok(())
    }

    async fn delete_prefix(&self, bucket: &str, prefix: &str) -> Result<(), StorageError> {
        self.delete_calls
            .lock()
            .expect("delete_calls mutex poisoned")
            .push(format!("{bucket}:{prefix}"));
        Ok(())
    }

    async fn list_keys(&self, _bucket: &str, prefix: &str) -> Result<Vec<String>, StorageError> {
        if prefix.starts_with("products/") {
            return Ok(self.image_keys.clone());
        }
        if prefix.starts_with("previews/") {
            return Ok(self.preview_keys.clone());
        }
        Ok(vec![])
    }

    async fn presigned_get_url(
        &self,
        bucket: &str,
        key: &str,
        _ttl: Duration,
    ) -> Result<String, StorageError> {
        Ok(format!("https://example.com/{bucket}/{key}"))
    }

    fn public_url(&self, bucket: &str, key: &str) -> String {
        format!("https://example.com/public/{bucket}/{key}")
    }
}

fn build_photo_service(
    photo_repo: Arc<dyn ProductPhotoRepository>,
    storage: Arc<dyn ImageStorage>,
) -> Arc<ProductPhotoService> {
    Arc::new(ProductPhotoService::new(
        photo_repo,
        storage,
        "images-bucket".into(),
        "preview-bucket".into(),
    ))
}

fn build_product_service(
    product_repo: Arc<dyn ProductRepository>,
    category_repo: Arc<dyn CategoryRepository>,
) -> ProductService {
    build_product_service_with_storage(
        product_repo,
        category_repo,
        Arc::new(StorageStub::default()),
    )
}

fn build_product_service_with_storage(
    product_repo: Arc<dyn ProductRepository>,
    category_repo: Arc<dyn CategoryRepository>,
    storage: Arc<StorageStub>,
) -> ProductService {
    let photo_repo: Arc<dyn ProductPhotoRepository> =
        Arc::new(ProductPhotoRepoStub::never_called());
    let storage_trait: Arc<dyn ImageStorage> = storage;
    let photo_service = build_photo_service(photo_repo, storage_trait);

    ProductService::new(lazy_pool(), product_repo, category_repo, photo_service)
}

#[tokio::test]
async fn create_product_admin_rejects_blank_name() {
    let svc = build_product_service(
        Arc::new(ProductRepoStub::new()),
        Arc::new(CategoryRepoStub::new()),
    );

    let mut req = minimal_create_req();
    req.name = "   ".into();

    let err = svc
        .create_product_admin(&req, None, vec![])
        .await
        .expect_err("blank name should be rejected");

    assert!(matches!(err, ServiceError::BadRequest(_)));
}

#[tokio::test]
async fn create_product_admin_rejects_missing_category() {
    let svc = build_product_service(
        Arc::new(ProductRepoStub::new()),
        Arc::new(CategoryRepoStub::with_category(None, None)),
    );

    let req = minimal_create_req();
    let err = svc
        .create_product_admin(&req, None, vec![])
        .await
        .expect_err("missing category should be rejected");

    assert!(matches!(err, ServiceError::BadRequest(_)));
}

#[tokio::test]
async fn create_product_admin_rejects_non_leaf_category() {
    let svc = build_product_service(
        Arc::new(ProductRepoStub::new()),
        Arc::new(CategoryRepoStub::with_category(
            Some(test_category_clothing()),
            Some(true),
        )),
    );

    let req = minimal_create_req();
    let err = svc
        .create_product_admin(&req, None, vec![])
        .await
        .expect_err("non-leaf category should be rejected");

    assert!(matches!(err, ServiceError::BadRequest(_)));
}

#[tokio::test]
async fn create_product_admin_rejects_type_mismatch() {
    let svc = build_product_service(
        Arc::new(ProductRepoStub::new()),
        Arc::new(CategoryRepoStub::with_category(
            Some(test_category_clothing()),
            Some(false),
        )),
    );

    let mut req = minimal_create_req();
    req.details.type_details = TypeDetailsInput::Footwear {
        shoe_width: None,
        insole_length_cm: None,
    };

    let err = svc
        .create_product_admin(&req, None, vec![])
        .await
        .expect_err("type mismatch should be rejected");

    assert!(matches!(err, ServiceError::BadRequest(_)));
}

#[tokio::test]
async fn update_rejects_blank_name() {
    let svc = build_product_service(
        Arc::new(ProductRepoStub::new()),
        Arc::new(CategoryRepoStub::new()),
    );

    let mut req = minimal_update_req();
    req.name = Some("".into());

    let err = svc
        .update(Uuid::new_v4(), &req, None, vec![])
        .await
        .expect_err("blank name should be rejected");

    assert!(matches!(err, ServiceError::BadRequest(_)));
}

#[tokio::test]
async fn update_rejects_type_mismatch_against_existing_category() {
    let id = Uuid::new_v4();
    let svc = build_product_service(
        Arc::new(ProductRepoStub::with_find_by_id(Some(test_product_full(
            id,
        )))),
        Arc::new(CategoryRepoStub::with_category(
            Some(test_category_clothing()),
            None,
        )),
    );

    let mut req = minimal_update_req();
    req.details = Some(UpdateProductDetailsRequest {
        condition: None,
        material: None,
        color: None,
        year_of_release: None,
        is_vintage: None,
        is_collab: None,
        collab_name: None,
        is_limited_edition: None,
        special_notes: None,
        size: None,
        type_details: Some(TypeDetailsInput::Footwear {
            shoe_width: None,
            insole_length_cm: None,
        }),
    });

    let err = svc
        .update(id, &req, None, vec![])
        .await
        .expect_err("type mismatch should be rejected");

    assert!(matches!(err, ServiceError::BadRequest(_)));
}

#[tokio::test]
async fn get_product_by_id_admin_returns_dto() {
    let id = Uuid::new_v4();
    let svc = build_product_service(
        Arc::new(ProductRepoStub::with_find_by_id(Some(test_product_full(
            id,
        )))),
        Arc::new(CategoryRepoStub::new()),
    );

    let dto = svc
        .get_product_by_id_admin(id)
        .await
        .expect("product should be returned");

    assert_eq!(dto.id, id);
    assert_eq!(dto.brand.name, "Nike");
    assert!(dto.preview_url.is_none());
    assert!(dto.image_urls.is_empty());
}

#[tokio::test]
async fn get_product_by_id_admin_returns_not_found() {
    let svc = build_product_service(
        Arc::new(ProductRepoStub::with_find_by_id(None)),
        Arc::new(CategoryRepoStub::new()),
    );

    let err = svc
        .get_product_by_id_admin(Uuid::new_v4())
        .await
        .expect_err("missing product should return not found");

    assert!(matches!(err, ServiceError::NotFound(_)));
}

#[tokio::test]
async fn get_product_previews_by_query_admin_returns_paginated() {
    let id = Uuid::new_v4();
    let svc = build_product_service(
        Arc::new(ProductRepoStub::with_search((
            vec![test_product_full(id)],
            2,
        ))),
        Arc::new(CategoryRepoStub::new()),
    );

    let query = ProductListQuery {
        page: Some(2),
        per_page: Some(1),
        brand_id: None,
        category_id: None,
        product_type: None,
        status: None,
        gender: None,
        color: None,
        price_min: None,
        price_max: None,
        condition: None,
        size_value: None,
        size_value2: None,
        size_system: None,
        size_group: None,
        size_values: None,
        size_values2: None,
        size_systems: None,
        shoe_widths: None,
        sort_by: None,
        sort_order: None,
        search: None,
    };

    let result = svc
        .get_product_previews_by_query_admin(&query)
        .await
        .expect("query should succeed");

    assert_eq!(result.items.len(), 1);
    assert_eq!(result.items[0].id, id);
    assert_eq!(result.total, 2);
    assert_eq!(result.page, 2);
    assert_eq!(result.per_page, 1);
    assert_eq!(result.total_pages, 2);
}

#[tokio::test]
async fn soft_delete_ok() {
    let product_id = Uuid::new_v4();
    let svc = build_product_service(
        Arc::new(ProductRepoStub::with_soft_delete(true)),
        Arc::new(CategoryRepoStub::new()),
    );

    svc.soft_delete(product_id, 1)
        .await
        .expect("soft_delete should succeed");
}

#[tokio::test]
async fn soft_delete_does_not_delete_storage_inline() {
    let product_id = Uuid::new_v4();
    let storage = Arc::new(StorageStub::default());
    let svc = build_product_service_with_storage(
        Arc::new(ProductRepoStub::with_soft_delete(true)),
        Arc::new(CategoryRepoStub::new()),
        storage.clone(),
    );

    svc.soft_delete(product_id, 1)
        .await
        .expect("soft_delete should succeed");

    assert!(storage.delete_calls().is_empty());
}

#[tokio::test]
async fn delete_product_images_removes_images_only() {
    let product_id = Uuid::new_v4();
    let storage = Arc::new(StorageStub::default());
    let photo_repo: Arc<dyn ProductPhotoRepository> =
        Arc::new(ProductPhotoRepoStub::never_called());
    let storage_trait: Arc<dyn ImageStorage> = storage.clone();
    let photo_service = build_photo_service(photo_repo, storage_trait);

    photo_service
        .delete_product_images(product_id)
        .await
        .expect("image cleanup should succeed");

    assert_eq!(
        storage.delete_calls(),
        vec![format!("images-bucket:products/{product_id}/")]
    );
}

#[tokio::test]
async fn soft_delete_stale_version() {
    let product_id = Uuid::new_v4();
    let storage = Arc::new(StorageStub::default());
    let svc = build_product_service_with_storage(
        Arc::new(ProductRepoStub::with_soft_delete(false)),
        Arc::new(CategoryRepoStub::new()),
        storage.clone(),
    );

    let err = svc
        .soft_delete(product_id, 1)
        .await
        .expect_err("stale delete should return conflict error");

    assert!(matches!(err, ServiceError::StaleVersion));
    assert!(storage.delete_calls().is_empty());
}

#[tokio::test]
async fn enqueue_images_upload_rejects_empty_files() {
    let photo_repo: Arc<dyn ProductPhotoRepository> =
        Arc::new(ProductPhotoRepoStub::never_called());
    let storage: Arc<dyn ImageStorage> = Arc::new(StorageStub::default());
    let svc = build_photo_service(photo_repo, storage);

    let err = svc
        .enqueue_images_upload(Uuid::new_v4(), vec![])
        .await
        .expect_err("empty files should be rejected");

    assert!(matches!(err, ServiceError::BadRequest(_)));
}

#[tokio::test]
async fn enqueue_images_upload_returns_not_found_when_product_missing() {
    let photo_repo: Arc<dyn ProductPhotoRepository> =
        Arc::new(ProductPhotoRepoStub::with_exists(false));
    let storage: Arc<dyn ImageStorage> = Arc::new(StorageStub::default());
    let svc = build_photo_service(photo_repo, storage);

    let file = UploadFile {
        data: vec![1, 2, 3],
        content_type: "image/jpeg".into(),
    };

    let err = svc
        .enqueue_images_upload(Uuid::new_v4(), vec![file])
        .await
        .expect_err("missing product should fail");

    assert!(matches!(err, ServiceError::NotFound(_)));
}

#[tokio::test]
async fn enqueue_images_upload_returns_job_id() {
    let photo_repo: Arc<dyn ProductPhotoRepository> =
        Arc::new(ProductPhotoRepoStub::with_queue(true, 42, 24));
    let storage: Arc<dyn ImageStorage> = Arc::new(StorageStub::default());
    let svc = build_photo_service(photo_repo, storage);

    let file = UploadFile {
        data: vec![1, 2, 3],
        content_type: "image/jpeg".into(),
    };

    let job_id = svc
        .enqueue_images_upload(Uuid::new_v4(), vec![file])
        .await
        .expect("enqueue should succeed");

    assert_eq!(job_id, 42);
}

#[tokio::test]
async fn enqueue_preview_upload_returns_job_id() {
    let photo_repo: Arc<dyn ProductPhotoRepository> =
        Arc::new(ProductPhotoRepoStub::with_queue(true, 42, 24));
    let storage: Arc<dyn ImageStorage> = Arc::new(StorageStub::default());
    let svc = build_photo_service(photo_repo, storage);

    let file = UploadFile {
        data: vec![1, 2, 3],
        content_type: "image/png".into(),
    };

    let job_id = svc
        .enqueue_preview_upload(Uuid::new_v4(), file)
        .await
        .expect("enqueue preview should succeed");

    assert_eq!(job_id, 24);
}

#[tokio::test]
async fn upload_images_rejects_oversized_file() {
    let photo_repo: Arc<dyn ProductPhotoRepository> =
        Arc::new(ProductPhotoRepoStub::never_called());
    let storage: Arc<dyn ImageStorage> = Arc::new(StorageStub::default());
    let svc = build_photo_service(photo_repo, storage);

    let file = UploadFile {
        data: vec![0u8; 11 * 1024 * 1024],
        content_type: "image/jpeg".into(),
    };

    let err = svc
        .upload_images(Uuid::new_v4(), vec![file])
        .await
        .expect_err("oversized file should be rejected");

    assert!(matches!(err, ServiceError::BadRequest(_)));
}

#[tokio::test]
async fn upload_images_rejects_invalid_content_type() {
    let photo_repo: Arc<dyn ProductPhotoRepository> =
        Arc::new(ProductPhotoRepoStub::never_called());
    let storage: Arc<dyn ImageStorage> = Arc::new(StorageStub::default());
    let svc = build_photo_service(photo_repo, storage);

    let file = UploadFile {
        data: vec![1, 2, 3],
        content_type: "application/pdf".into(),
    };

    let err = svc
        .upload_images(Uuid::new_v4(), vec![file])
        .await
        .expect_err("invalid content type should be rejected");

    assert!(matches!(err, ServiceError::BadRequest(_)));
}

#[tokio::test]
async fn upload_images_ok() {
    let id = Uuid::new_v4();
    let photo_repo: Arc<dyn ProductPhotoRepository> =
        Arc::new(ProductPhotoRepoStub::with_exists(true));
    let storage = Arc::new(StorageStub::with_existing(
        vec![
            format!("products/{id}/0.jpg"),
            format!("products/{id}/1.jpg"),
        ],
        vec![],
    ));
    let storage_trait: Arc<dyn ImageStorage> = storage.clone();
    let svc = build_photo_service(photo_repo, storage_trait);

    let file = UploadFile {
        data: tiny_png_bytes(),
        content_type: "image/png".into(),
    };

    let count = svc
        .upload_images(id, vec![file])
        .await
        .expect("upload should succeed");

    assert_eq!(count, 1);

    let puts = storage.put_calls();
    assert_eq!(puts.len(), 3);
}

#[tokio::test]
async fn upload_preview_ok() {
    let id = Uuid::new_v4();
    let photo_repo: Arc<dyn ProductPhotoRepository> =
        Arc::new(ProductPhotoRepoStub::with_exists(true));
    let storage = Arc::new(StorageStub::default());
    let storage_trait: Arc<dyn ImageStorage> = storage.clone();
    let svc = build_photo_service(photo_repo, storage_trait);

    let file = UploadFile {
        data: tiny_png_bytes(),
        content_type: "image/png".into(),
    };

    let _key = svc
        .upload_preview(id, file)
        .await
        .expect("preview upload should succeed");

    let puts = storage.put_calls();
    assert!(!puts.is_empty());
}

#[tokio::test]
async fn upload_preview_rejects_invalid_content_type() {
    let photo_repo: Arc<dyn ProductPhotoRepository> =
        Arc::new(ProductPhotoRepoStub::never_called());
    let storage: Arc<dyn ImageStorage> = Arc::new(StorageStub::default());
    let svc = build_photo_service(photo_repo, storage);

    let file = UploadFile {
        data: vec![1, 2, 3],
        content_type: "image/gif".into(),
    };

    let err = svc
        .upload_preview(Uuid::new_v4(), file)
        .await
        .expect_err("invalid content type should be rejected");

    assert!(matches!(err, ServiceError::BadRequest(_)));
}

#[tokio::test]
async fn upload_preview_returns_not_found_when_product_missing() {
    let photo_repo: Arc<dyn ProductPhotoRepository> =
        Arc::new(ProductPhotoRepoStub::with_exists(false));
    let storage: Arc<dyn ImageStorage> = Arc::new(StorageStub::default());
    let svc = build_photo_service(photo_repo, storage);

    let file = UploadFile {
        data: vec![1, 2, 3],
        content_type: "image/png".into(),
    };

    let err = svc
        .upload_preview(Uuid::new_v4(), file)
        .await
        .expect_err("missing product should return not found");

    assert!(matches!(err, ServiceError::NotFound(_)));
}
