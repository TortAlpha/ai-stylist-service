use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use uuid::Uuid;

/// Short product card — used in lists, search results, chat bot
#[derive(Debug, Serialize)]
pub struct ProductPreviewResponse {
    pub id: Uuid,
    pub sku: String,
    pub name: String,
    pub original_price: Decimal,
    pub discount: Decimal,
    pub final_price: Decimal,
    pub currency: String,
    pub preview_image_url: Option<String>,
    pub product_url: Option<String>,
    pub brand: BrandShortResponse,
    pub product_type: String,
    pub category: String,
    pub condition: String,
    pub color: Option<String>,
    /// Human-readable size label (e.g. "43 EU", "M", "30x20x10 cm")
    pub size_label: Option<String>,
}

/// Full product card — used on product page
#[derive(Debug, Serialize)]
pub struct ProductResponse {
    pub id: Uuid,
    pub sku: String,
    pub name: String,
    pub original_price: Decimal,
    pub discount: Decimal,
    pub final_price: Decimal,
    pub currency: String,
    pub in_stock: bool,
    pub preview_image_url: Option<String>,
    pub product_url: Option<String>,
    pub brand: BrandShortResponse,
    pub product_type: String,
    pub category: CategoryResponse,
    pub status: String,
    pub details: ProductAttributesResponse,
    pub type_details: TypeDetailsResponse,
    pub tags: ProductTagsResponse,
    pub created_at: DateTime<Utc>,
}

/// Brand — nested inside product responses
#[derive(Debug, Serialize)]
pub struct BrandShortResponse {
    pub id: i32,
    pub name: String,
    pub tier: String,
}

/// Category — nested inside product responses
#[derive(Debug, Serialize)]
pub struct CategoryResponse {
    pub name: String,
    pub parent_category: Option<String>,
    pub gender: String,
}

/// Common product details — nested, not a separate endpoint
#[derive(Debug, Serialize)]
pub struct ProductAttributesResponse {
    pub material: Option<String>,
    pub condition: String,
    pub color: Option<String>,
    pub year_of_release: Option<i32>,
    pub is_vintage: bool,
    pub is_collab: bool,
    pub collab_name: Option<String>,
    pub is_limited_edition: bool,
    pub special_notes: Option<String>,
}

/// Type-specific details — tagged enum, serialized as { "type": "footwear", ...fields }
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TypeDetailsResponse {
    Clothing {
        size: Option<String>,
        fit: Option<String>,
    },
    Footwear {
        shoe_size: Option<String>,
        size_system: String,
    },
    Bags {
        width_cm: Option<Decimal>,
        height_cm: Option<Decimal>,
        depth_cm: Option<Decimal>,
        handle_type: Option<String>,
    },
    Jewelry {
        metal: Option<String>,
        stone: Option<String>,
        clasp_type: Option<String>,
    },
    Accessories,
}

/// All tags grouped together
#[derive(Debug, Serialize)]
pub struct ProductTagsResponse {
    pub styles: Vec<String>,
    pub vibes: Vec<String>,
    pub seasons: Vec<String>,
}
