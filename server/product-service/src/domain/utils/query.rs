use rust_decimal::Decimal;
use serde::Deserialize;

use super::pagination::PaginationParams;

/// Query parameters for admin brand search with pagination
#[derive(Debug, Clone, Deserialize, utoipa::IntoParams)]
pub struct BrandSearchQuery {
    #[serde(default)]
    pub search: String,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

impl BrandSearchQuery {
    pub fn pagination(&self) -> PaginationParams {
        PaginationParams {
            page: self.page,
            per_page: self.per_page,
        }
    }
}

/// Query parameters for category listing
#[derive(Debug, Clone, Deserialize, utoipa::IntoParams)]
pub struct CategoryListQuery {
    pub parent_id: Option<i32>,
}

/// Query parameters for listing products
#[derive(Debug, Clone, Deserialize, utoipa::IntoParams)]
pub struct ProductListQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub brand_id: Option<i32>,
    pub category_id: Option<i32>,
    pub product_type: Option<String>,
    pub status: Option<String>,
    pub gender: Option<String>,
    pub price_min: Option<Decimal>,
    pub price_max: Option<Decimal>,
    pub condition: Option<String>,

    // Size filters — meaningful only within a category or size_group context
    pub size_value: Option<String>,
    pub size_value2: Option<String>,
    pub size_system: Option<String>,
    pub size_group: Option<String>,

    pub sort_by: Option<ProductSortField>,
    pub sort_order: Option<SortOrder>,

    pub search: Option<String>,
}

/// Query parameters for available sizes endpoint.
/// Returns distinct size values for a given category or size_group,
/// so the UI knows which filter options to render.
///
/// Usage:
///   GET /api/products/sizes?category_id=47        → ["XS","S","M","L","XL"]
///   GET /api/products/sizes?size_group=shoe        → { values: ["38","39",...], systems: ["EU","US","UK"] }
///   GET /api/products/sizes?category_id=118        → { values: ["28","30","32"], values2: ["30","32","34"] }
#[derive(Debug, Clone, Deserialize, utoipa::IntoParams)]
pub struct AvailableSizesQuery {
    pub category_id: Option<i32>,
    pub size_group: Option<String>,
    pub brand_id: Option<i32>,
    pub gender: Option<String>,
}

#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProductSortField {
    Price,
    CreatedAt,
    UpdatedAt,
    Name,
}

#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    Asc,
    Desc,
}

/// Query params for GET /api/products/filter-options.
/// Accepts the same filter scope as the product list —
/// returns available values for the remaining filter fields.
#[derive(Debug, Clone, Deserialize, utoipa::IntoParams)]
pub struct FilterOptionsQuery {
    pub brand_id: Option<i32>,
    pub category_id: Option<i32>,
    pub product_type: Option<String>,
    pub gender: Option<String>,
    pub condition: Option<String>,
    pub price_min: Option<Decimal>,
    pub price_max: Option<Decimal>,
}

/// Query parameters for soft-deleting a product (optimistic locking)
#[derive(Debug, Clone, Deserialize, utoipa::IntoParams)]
pub struct DeleteProductQuery {
    pub expected_version: i32,
}
