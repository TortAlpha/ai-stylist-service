use super::super::product::ProductFull;
use super::super::response_dto::brand::BrandShortResponse;
use super::super::response_dto::category::CategoryResponse;
use super::super::response_dto::product::ProductPreviewResponse;
use super::super::response_dto::product_details::ProductAttributesResponse;
use super::super::response_dto::product_details::SizeResponse;
use super::super::response_dto::product_details::TypeDetailsResponse;
use super::super::response_dto::tag::ProductTagsResponse;
use crate::domain::response_dto::product::{
    AdminProductDTO, AdminProductPreviewResponse, MarketplaceProductDTO,
};

/// Three presigned URLs for the same image at different resolutions.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ImageVariantUrls {
    pub thumb: String,
    pub medium: String,
    pub full: String,
}

/// Indexed product image: storage `id` plus its three variant URLs.
/// The `id` is the directory index under `products/{product_id}/{id}/` and
/// is the value clients pass back to `DELETE /products/{id}/images/{image_id}`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct ProductImageUrls {
    pub id: usize,
    pub thumb: String,
    pub medium: String,
    pub full: String,
}

/// Image URLs resolved by the service layer (presigned)
#[derive(Debug, serde::Serialize)]
pub struct ResolvedImageUrls {
    pub preview_url: Option<ImageVariantUrls>,
    pub image_urls: Vec<ProductImageUrls>,
}

impl ProductFull {
    pub fn into_response_for_admin(self, urls: ResolvedImageUrls) -> AdminProductDTO {
        let p = self;
        let size = map_size(&p);
        let type_details = map_type_details(&p);
        let details = map_attributes(&p);

        AdminProductDTO {
            id: p.id,
            sku: p.sku,
            name: p.name,
            purchase_price: p.purchase_price,
            purchase_location: p.purchase_location,
            currency: p.currency,
            ai_notes: p.ai_notes,
            preview_url: urls.preview_url,
            image_urls: urls.image_urls,
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
                size_group: p.size_group,
            },
            category_id: p.category_id,
            status: p.status,
            version: p.version,
            details,
            size,
            type_details,
            tags: ProductTagsResponse {
                styles: p.style_tags,
                vibes: p.vibe_tags,
                seasons: p.season_tags,
            },
            created_at: p.created_at,
            updated_at: p.updated_at,
        }
    }

    pub fn into_response_for_marketplace(self, urls: ResolvedImageUrls) -> MarketplaceProductDTO {
        todo!()
    }

    pub fn into_admin_preview(self, urls: ResolvedImageUrls) -> AdminProductPreviewResponse {
        let p = self;
        let size = map_size(&p);

        AdminProductPreviewResponse {
            id: p.id,
            sku: p.sku,
            name: p.name,
            purchase_price: p.purchase_price,
            currency: p.currency,
            preview_url: urls.preview_url,
            brand_name: p.brand_name,
            product_type: p.product_type,
            category: p.category_name,
            status: p.status,
            condition: p.condition,
            color: p.color,
            size,
        }
    }

    pub fn into_preview(self, urls: ResolvedImageUrls) -> ProductPreviewResponse {
        let p = self;
        let size = map_size(&p);

        ProductPreviewResponse {
            id: p.id,
            sku: p.sku,
            name: p.name,
            purchase_price: p.purchase_price,
            currency: p.currency,
            preview_url: urls.preview_url,
            brand_name: p.brand_name,
            product_type: p.product_type,
            category: p.category_name,
            color: p.color,
            size,
        }
    }
}

fn map_attributes(p: &ProductFull) -> ProductAttributesResponse {
    ProductAttributesResponse {
        material: p.material.clone(),
        condition: default_condition(p.condition.clone()),
        color: p.color.clone(),
        year_of_release: p.year_of_release,
        is_vintage: p.is_vintage.unwrap_or(false),
        is_collab: p.is_collab.unwrap_or(false),
        collab_name: p.collab_name.clone(),
        is_limited_edition: p.is_limited_edition.unwrap_or(false),
        special_notes: p.special_notes.clone(),
    }
}

fn map_type_details(p: &ProductFull) -> TypeDetailsResponse {
    match p.product_type.as_str() {
        "clothing" => TypeDetailsResponse::Clothing {
            fit: p.clothing_fit.clone(),
        },
        "footwear" => TypeDetailsResponse::Footwear {
            shoe_width: p.shoe_width.clone(),
            insole_length_cm: p.insole_length_cm,
        },
        "bags" => TypeDetailsResponse::Bags {
            width_cm: p.bag_width_cm,
            height_cm: p.bag_height_cm,
            depth_cm: p.bag_depth_cm,
            handle_type: p.bag_handle_type.clone(),
            bag_size_label: p.bag_size_label.clone(),
        },
        "jewelry" => TypeDetailsResponse::Jewelry {
            metal: p.jewelry_metal.clone(),
            stone: p.jewelry_stone.clone(),
            clasp_type: p.jewelry_clasp_type.clone(),
        },
        _ => TypeDetailsResponse::Accessories,
    }
}

fn map_size(p: &ProductFull) -> SizeResponse {
    SizeResponse {
        size_group: p.size_group.clone(),
        size_value: p.size_value.clone(),
        size_value2: p.size_value2.clone(),
        size_system: p.size_system.clone(),
        measurement_cm: p.measurement_cm,
        size_label: make_size_label(p),
    }
}

fn make_size_label(p: &ProductFull) -> Option<String> {
    match p.size_group.as_str() {
        "letter" => p.size_value.clone(),
        "letter_or_numeric" | "shoe" | "ring" => match (&p.size_value, &p.size_system) {
            (Some(v), Some(system)) => Some(format!("{} {}", v, system)),
            (Some(v), None) => Some(v.clone()),
            _ => None,
        },
        "waist_length" => match (&p.size_value, &p.size_value2) {
            (Some(w), Some(l)) => Some(format!("W{}/L{}", w, l)),
            (Some(w), None) => Some(format!("W{}", w)),
            _ => None,
        },
        "measurement_cm" => p.measurement_cm.map(|cm| format!("{} cm", cm)),
        "hat" => match (&p.size_value, p.measurement_cm) {
            (Some(v), _) => Some(v.clone()),
            (None, Some(cm)) => Some(format!("{} cm", cm)),
            _ => None,
        },
        "dimensions" => match (p.bag_width_cm, p.bag_height_cm, p.bag_depth_cm) {
            (Some(w), Some(h), Some(d)) => Some(format!("{}x{}x{} cm", w, h, d)),
            (Some(w), Some(h), None) => Some(format!("{}x{} cm", w, h)),
            _ => None,
        },
        _ => None,
    }
}

fn default_condition(condition: Option<String>) -> String {
    condition.unwrap_or_else(|| "unknown".to_string())
}
