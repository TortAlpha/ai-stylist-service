use rust_decimal::Decimal;
use serde::Serialize;

#[derive(Debug, Serialize, utoipa::ToSchema)]
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

/// Unified size info in responses
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SizeResponse {
    pub size_group: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_value2: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub measurement_cm: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_label: Option<String>,
}

/// Type-specific details — tagged enum, serialized as { "type": "footwear", ...fields }
#[derive(Debug, Serialize, utoipa::ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TypeDetailsResponse {
    Clothing {
        fit: Option<String>,
    },
    Footwear {
        shoe_width: Option<String>,
        insole_length_cm: Option<Decimal>,
    },
    Bags {
        width_cm: Option<Decimal>,
        height_cm: Option<Decimal>,
        depth_cm: Option<Decimal>,
        handle_type: Option<String>,
        bag_size_label: Option<String>,
    },
    Jewelry {
        metal: Option<String>,
        stone: Option<String>,
        clasp_type: Option<String>,
    },
    Accessories,
}

/// Response for GET /api/products/sizes — available size filter options.
/// Fields are populated based on the size_group of the queried category.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct AvailableSizesResponse {
    pub size_group: String,

    /// Distinct primary size values (e.g. ["XS","S","M","L","XL"] or ["38","39","40","41","42"])
    pub values: Vec<String>,

    /// Distinct secondary size values — only for waist_length (e.g. inseam: ["30","32","34"])
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub values2: Vec<String>,

    /// Distinct size systems — only for letter_or_numeric, shoe, ring (e.g. ["EU","US","UK"])
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub systems: Vec<String>,

    /// Distinct shoe widths — only for shoe (e.g. ["narrow","regular","wide"])
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub widths: Vec<String>,
}
