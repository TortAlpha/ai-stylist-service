use sqlx::PgPool;
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::domain::category::Category;
use crate::domain::error::ServiceError;
use crate::domain::product::ProductFull;
use crate::domain::product_photo::UploadFile;
use crate::domain::request_dto::product::{CreateProductRequest, UpdateProductRequest};
use crate::domain::response_dto::product::{
    AdminProductDTO, AdminProductPreviewResponse, ProductFilterOptions, ProductPreviewResponse,
};
use crate::domain::response_dto::product_details::AvailableSizesResponse;
use crate::domain::utils::pagination::{PaginatedResponse, PaginationParams};
use crate::domain::utils::query::{AvailableSizesQuery, FilterOptionsQuery, ProductListQuery};
use crate::jobs;
use crate::repo::traits::category_repo::CategoryRepository;
use crate::repo::traits::product_repo::ProductRepository;
use crate::service::product_photo_service::ProductPhotoService;

use super::validate_details::{
    validate_common_details, validate_type_details, validate_type_matches_category,
};
use super::validate_product::{validate_create_product, validate_update_product};
use super::validate_size::validate_size_input;

type Result<T> = std::result::Result<T, ServiceError>;

pub struct ProductService {
    pool: PgPool,
    repo: Arc<dyn ProductRepository>,
    category_repo: Arc<dyn CategoryRepository>,
    photo_service: Arc<ProductPhotoService>,
}

impl ProductService {
    pub fn new(
        pool: PgPool,
        repo: Arc<dyn ProductRepository>,
        category_repo: Arc<dyn CategoryRepository>,
        photo_service: Arc<ProductPhotoService>,
    ) -> Self {
        Self {
            pool,
            repo,
            category_repo,
            photo_service,
        }
    }

    // ── CRUD ────────────────────────────────────────────────

    pub async fn get_product_by_id_admin(&self, id: Uuid) -> Result<AdminProductDTO> {
        debug!(product_id = %id, "service:get_product_by_id_admin");
        let product_full = self.find_product_full_by_id(id).await?;
        let urls = self.photo_service.resolve_image_urls(product_full.id).await;
        let admin_dto = product_full.into_response_for_admin(urls);
        info!(product_id = %id, "service:get_product_by_id_admin succeeded");
        Ok(admin_dto)
    }

    pub async fn get_product_previews_by_query_admin(
        &self,
        query: &ProductListQuery,
    ) -> Result<PaginatedResponse<AdminProductPreviewResponse>> {
        debug!(
            page = ?query.page,
            per_page = ?query.per_page,
            "service:get_product_previews_by_query_admin"
        );
        let (items, total) = self
            .repo
            .search_by_query(query)
            .await
            .map_err(ServiceError::from)?;

        let mut previews = Vec::with_capacity(items.len());
        for product in items {
            let urls = self.photo_service.resolve_image_urls(product.id).await;
            previews.push(product.into_admin_preview(urls));
        }

        let pagination = PaginationParams {
            page: query.page,
            per_page: query.per_page,
        };
        info!(
            items = previews.len(),
            total, "service:list products succeeded"
        );
        Ok(pagination.paginate(previews, total))
    }

    pub async fn create_product_admin(
        &self,
        req: &CreateProductRequest,
        preview: Option<UploadFile>,
        images: Vec<UploadFile>,
    ) -> Result<AdminProductDTO> {
        debug!(
            category_id = req.category_id,
            has_preview = preview.is_some(),
            images_count = images.len(),
            "service:create_product_admin"
        );
        validate_create_product(req)?;

        let category = self.get_leaf_category(req.category_id).await?;

        validate_common_details(&req.details)?;
        validate_type_matches_category(&req.details.type_details, &category.product_type)?;
        validate_type_details(&req.details.type_details)?;
        validate_size_input(&req.details.size, &category.size_group)?;

        // Product insert + upload jobs in one atomic transaction.
        let mut tx = self.pool.begin().await.map_err(ServiceError::from)?;

        let product_full = self
            .repo
            .create_in_tx(&mut tx, req)
            .await
            .map_err(ServiceError::from)?;

        if let Some(file) = preview {
            jobs::enqueue_upload_product_preview(&mut tx, product_full.id, file)
                .await
                .map_err(ServiceError::from)?;
        }

        if !images.is_empty() {
            jobs::enqueue_upload_product_images(&mut tx, product_full.id, images)
                .await
                .map_err(ServiceError::from)?;
        }

        tx.commit().await.map_err(ServiceError::from)?;

        let urls = self.photo_service.resolve_image_urls(product_full.id).await;
        info!(product_id = %product_full.id, "service:create_product_admin succeeded");
        Ok(product_full.into_response_for_admin(urls))
    }

    pub async fn update(
        &self,
        id: Uuid,
        req: &UpdateProductRequest,
        preview: Option<UploadFile>,
        images: Vec<UploadFile>,
    ) -> Result<AdminProductDTO> {
        debug!(
            product_id = %id,
            category_id = ?req.category_id,
            has_preview = preview.is_some(),
            images_count = images.len(),
            "service:update_product"
        );
        validate_update_product(req)?;

        if let Some(category_id) = req.category_id {
            self.get_leaf_category(category_id).await?;
        }

        if let Some(ref details) = req.details {
            if let Some(ref type_details) = details.type_details {
                let cat_id = if let Some(cid) = req.category_id {
                    cid
                } else {
                    let existing = self.find_product_full_by_id(id).await?;
                    existing.category_id
                };

                let category = self
                    .category_repo
                    .find_by_id(cat_id)
                    .await
                    .map_err(ServiceError::from)?
                    .ok_or_else(|| {
                        ServiceError::BadRequest(format!("category {cat_id} not found"))
                    })?;

                validate_type_matches_category(type_details, &category.product_type)?;
                validate_type_details(type_details)?;

                if let Some(ref size) = details.size {
                    validate_size_input(size, &category.size_group)?;
                }
            } else if let Some(ref size) = details.size {
                let cat_id = if let Some(cid) = req.category_id {
                    cid
                } else {
                    let existing = self.find_product_full_by_id(id).await?;
                    existing.category_id
                };

                let category = self
                    .category_repo
                    .find_by_id(cat_id)
                    .await
                    .map_err(ServiceError::from)?
                    .ok_or_else(|| {
                        ServiceError::BadRequest(format!("category {cat_id} not found"))
                    })?;

                validate_size_input(size, &category.size_group)?;
            }
        }

        let mut tx = self.pool.begin().await.map_err(ServiceError::from)?;

        let product = self
            .repo
            .update_in_tx(&mut tx, id, req)
            .await
            .map_err(ServiceError::from)?;

        if let Some(file) = preview {
            jobs::enqueue_upload_product_preview(&mut tx, product.id, file)
                .await
                .map_err(ServiceError::from)?;
        }

        if !images.is_empty() {
            jobs::enqueue_upload_product_images(&mut tx, product.id, images)
                .await
                .map_err(ServiceError::from)?;
        }

        tx.commit().await.map_err(ServiceError::from)?;

        let urls = self.photo_service.resolve_image_urls(product.id).await;
        info!(product_id = %product.id, version = product.version, "service:update_product succeeded");
        Ok(product.into_response_for_admin(urls))
    }

    pub async fn soft_delete(&self, id: Uuid, expected_version: i32) -> Result<()> {
        debug!(product_id = %id, expected_version, "service:soft_delete");
        let deleted = self
            .repo
            .soft_delete_with_photo_cleanup(id, expected_version)
            .await
            .map_err(ServiceError::from)?;

        if !deleted {
            warn!(product_id = %id, expected_version, "service:soft_delete stale version");
            return Err(ServiceError::StaleVersion);
        }

        info!(product_id = %id, "service:soft_delete succeeded");
        Ok(())
    }

    // ── Filters ─────────────────────────────────────────────

    pub async fn filter_options(&self, query: &FilterOptionsQuery) -> Result<ProductFilterOptions> {
        debug!("service:filter_options");
        self.repo
            .filter_options(query)
            .await
            .map_err(ServiceError::from)
    }

    pub async fn available_sizes(
        &self,
        query: &AvailableSizesQuery,
    ) -> Result<AvailableSizesResponse> {
        debug!(
            category_id = ?query.category_id,
            size_group = ?query.size_group,
            "service:available_sizes"
        );
        self.repo
            .available_sizes(query)
            .await
            .map_err(ServiceError::from)
    }

    // ── Internal ───────────────────────────────────────────

    async fn get_leaf_category(&self, category_id: i32) -> Result<Category> {
        debug!(category_id, "service:get_leaf_category");
        let category = self
            .category_repo
            .find_by_id(category_id)
            .await
            .map_err(ServiceError::from)?
            .ok_or_else(|| ServiceError::BadRequest(format!("category {category_id} not found")))?;

        let has_children = self
            .category_repo
            .has_children(category_id)
            .await
            .map_err(ServiceError::from)?;

        if has_children {
            warn!(
                category_id,
                "service:get_leaf_category category has children"
            );
            return Err(ServiceError::BadRequest(format!(
                "category {category_id} is not a leaf category; select a specific subcategory",
            )));
        }

        Ok(category)
    }

    async fn find_product_full_by_id(&self, id: Uuid) -> Result<ProductFull> {
        debug!(product_id = %id, "service:find_product_full_by_id");
        self.repo
            .find_by_id(id)
            .await
            .map_err(ServiceError::from)?
            .ok_or_else(|| ServiceError::NotFound(format!("product {id} not found")))
    }
}
