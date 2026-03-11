use super::product::ProductFull;
use super::product_details::{
    BagDetails, ClothingDetails, FootwearDetails, JewelryDetails,
};
use super::response_dto::{
    BrandShortResponse, CategoryResponse, ProductAttributesResponse, ProductPreviewResponse,
    ProductResponse, ProductTagsResponse, TypeDetailsResponse,
};

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

// ============================================================
// ProductFull + TypeDetails → ProductResponse (full card)
// ============================================================

impl From<ProductWithTypeDetails> for ProductResponse {
    fn from(data: ProductWithTypeDetails) -> Self {
        let p = data.full;

        Self {
            id: p.id,
            sku: p.sku,
            name: p.name,
            original_price: p.original_price,
            discount: p.discount,
            final_price: p.final_price,
            currency: p.currency,
            in_stock: p.in_stock,
            preview_image_url: p.preview_image_url,
            product_url: p.product_url,
            brand: BrandShortResponse {
                id: p.brand_id,
                name: p.brand_name,
                tier: p.brand_tier,
            },
            product_type: p.type_name,
            category: CategoryResponse {
                name: p.category_name,
                parent_category: p.parent_category,
                gender: p.gender,
            },
            status: p.status_name,
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
            type_details: map_type_details(data.type_details),
            tags: ProductTagsResponse {
                styles: p.style_tags,
                vibes: p.vibe_tags,
                seasons: p.season_tags,
            },
            created_at: p.created_at,
        }
    }
}

// ============================================================
// ProductFull + TypeDetails → ProductPreviewResponse (list card)
// ============================================================

impl From<ProductWithTypeDetails> for ProductPreviewResponse {
    fn from(data: ProductWithTypeDetails) -> Self {
        let p = data.full;
        let size_label = make_size_label(&data.type_details);

        Self {
            id: p.id,
            sku: p.sku,
            name: p.name,
            original_price: p.original_price,
            discount: p.discount,
            final_price: p.final_price,
            currency: p.currency,
            preview_image_url: p.preview_image_url,
            product_url: p.product_url,
            brand: BrandShortResponse {
                id: p.brand_id,
                name: p.brand_name,
                tier: p.brand_tier,
            },
            product_type: p.type_name,
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
            size_system: format!("{:?}", d.size_system),
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
