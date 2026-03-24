use super::super::response_dto::brand::BrandShortResponse;
use super::super::response_dto::category::CategoryResponse;
use super::super::response_dto::product_details::ProductAttributesResponse;
use super::super::response_dto::product::ProductPreviewResponse;
use super::super::response_dto::product::ProductResponse;
use super::super::response_dto::tag::ProductTagsResponse;
use super::super::response_dto::product_details::TypeDetailsResponse;

use super::super::product::ProductFull;
use super::super::bags_details::BagDetails;
use super::super::clothing_details::ClothingDetails;
use super::super::footwear_details::FootwearDetails;
use super::super::jewelry_details::JewelryDetails;

/// Wrapper that carries view data + type-specific details for full mapping
pub struct ProductWithTypeDetails {
    pub full: ProductFull,
    pub type_details: TypeDetails,
}

/// Type-specific detail variants fetched from separate tables
pub enum TypeDetails {
    Clothing(ClothingDetails),
    Footwear(FootwearDetails),
    Bags(BagDetails),
    Jewelry(JewelryDetails),
    Accessories,
}

/// Image URLs resolved by the service layer (presigned)
#[derive(Debug, serde::Serialize)]
pub struct ResolvedImageUrls {
    pub preview_url: Option<String>,
    pub image_urls: Vec<String>,
}

// ============================================================
// ProductFull + TypeDetails + URLs → ProductResponse (full card)
// ============================================================

impl ProductWithTypeDetails {
    pub fn into_response(self, urls: ResolvedImageUrls) -> ProductResponse {
        let p = self.full;

        ProductResponse {
            id: p.id,
            sku: p.sku,
            name: p.name,
            purchase_price: p.purchase_price,
            purchase_location: p.purchase_location,
            currency: p.currency,
            ai_notes: p.ai_notes,
            preview_url: urls.preview_url,
            image_urls: urls.image_urls,
            product_url: p.product_url,
            brand: BrandShortResponse {
                id: p.brand_id,
                name: p.brand_name,
                tier: p.brand_tier,
            },
            brand_id: p.brand_id,
            product_type: p.product_type,
            category: CategoryResponse {
                name: p.category_name,
                parent_category: p.parent_category,
                gender: p.gender,
            },
            category_id: p.category_id,
            status: p.status,
            version: p.version,
            details: ProductAttributesResponse {
                material: p.material,
                condition: p.condition.unwrap_or_default(),
                color: p.color,
                year_of_release: p.year_of_release,
                is_vintage: p.is_vintage.unwrap_or(false),
                is_collab: p.is_collab.unwrap_or(false),
                collab_name: p.collab_name,
                is_limited_edition: p.is_limited_edition.unwrap_or(false),
                special_notes: p.special_notes,
            },
            type_details: map_type_details(self.type_details),
            tags: ProductTagsResponse {
                styles: p.style_tags,
                vibes: p.vibe_tags,
                seasons: p.season_tags,
            },
            created_at: p.created_at,
        }
    }

    pub fn into_preview(self, urls: ResolvedImageUrls) -> ProductPreviewResponse {
        let p = self.full;
        let size_label = make_size_label(&self.type_details);

        ProductPreviewResponse {
            id: p.id,
            sku: p.sku,
            name: p.name,
            purchase_price: p.purchase_price,
            currency: p.currency,
            preview_url: urls.preview_url,
            image_urls: urls.image_urls,
            product_url: p.product_url,
            brand: BrandShortResponse {
                id: p.brand_id,
                name: p.brand_name,
                tier: p.brand_tier,
            },
            product_type: p.product_type,
            category: p.category_name,
            condition: p.condition.unwrap_or_default(),
            color: p.color,
            size_label,
        }
    }
}

// ============================================================
// Helpers
// ============================================================

fn map_type_details(details: TypeDetails) -> TypeDetailsResponse {
    match details {
        TypeDetails::Clothing(d) => TypeDetailsResponse::Clothing {
            size: d.size,
            fit: d.fit.map(|f| format!("{:?}", f).to_lowercase()),
        },
        TypeDetails::Footwear(d) => TypeDetailsResponse::Footwear {
            shoe_size: d.shoe_size,
            size_system: Some(format!("{:?}", d.size_system)),
            insole_length_cm: d.insole_length_cm,
        },
        TypeDetails::Bags(d) => TypeDetailsResponse::Bags {
            width_cm: d.width_cm,
            height_cm: d.height_cm,
            depth_cm: d.depth_cm,
            handle_type: d.handle_type.map(|h| format!("{:?}", h).to_lowercase()),
        },
        TypeDetails::Jewelry(d) => TypeDetailsResponse::Jewelry {
            metal: d.metal,
            stone: d.stone,
            clasp_type: d.clasp_type,
        },
        TypeDetails::Accessories => TypeDetailsResponse::Accessories,
    }
}

/// Generates a human-readable size label for preview cards
fn make_size_label(details: &TypeDetails) -> Option<String> {
    match details {
        TypeDetails::Clothing(d) => d.size.clone(),
        TypeDetails::Footwear(d) => d
            .shoe_size
            .as_ref()
            .map(|s| format!("{} {:?}", s, d.size_system)),
        TypeDetails::Bags(d) => {
            match (d.width_cm, d.height_cm, d.depth_cm) {
                (Some(w), Some(h), Some(dep)) => Some(format!("{}x{}x{} cm", w, h, dep)),
                (Some(w), Some(h), None) => Some(format!("{}x{} cm", w, h)),
                _ => None,
            }
        }
        TypeDetails::Jewelry(_) | TypeDetails::Accessories => None,
    }
}
