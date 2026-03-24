use serde::Serialize;
use rust_decimal::Decimal;

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
        size_system: Option<String>,
        insole_length_cm: Option<Decimal>,
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

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AvailableSizesResponse {
    Clothing {
        sizes: Vec<String>,
        fits: Vec<String>,
    },
    Footwear {
        shoe_sizes: Vec<String>,
        size_systems: Vec<String>,
    },
    Bags,
    Jewelry,
    Accessories,
}