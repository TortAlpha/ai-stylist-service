use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use super::bags_details::HandleType;
use super::category::ProductType;
use super::clothing_details::ClothingFit;
use super::footwear_details::ShoeWidth;
use super::size_info::{SizeInfo, SizeSystem};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProductDetails {
    pub product_id: Uuid,
    pub material: Option<String>,
    pub condition: ProductCondition,
    pub color: Option<String>,
    pub year_of_release: Option<i32>,
    pub is_vintage: bool,
    pub is_collab: bool,
    pub collab_name: Option<String>,
    pub is_limited_edition: bool,
    pub special_notes: Option<String>,
    pub size_value: Option<String>,
    pub size_value2: Option<String>,
    pub size_system: Option<SizeSystem>,
    pub measurement_cm: Option<Decimal>,
    pub fit: Option<ClothingFit>,
    pub shoe_width: Option<ShoeWidth>,
    pub insole_length_cm: Option<Decimal>,
    pub width_cm: Option<Decimal>,
    pub height_cm: Option<Decimal>,
    pub depth_cm: Option<Decimal>,
    pub handle_type: Option<HandleType>,
    pub bag_size_label: Option<String>,
    pub metal: Option<String>,
    pub stone: Option<String>,
    pub clasp_type: Option<String>,
}

impl ProductDetails {
    pub fn to_size_info(&self) -> SizeInfo {
        SizeInfo {
            product_id: self.product_id,
            size_value: self.size_value.clone(),
            size_value2: self.size_value2.clone(),
            size_system: self.size_system.clone(),
            measurement_cm: self.measurement_cm,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, utoipa::ToSchema)]
#[sqlx(type_name = "VARCHAR", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ProductCondition {
    NewWithTags,
    Excellent,
    Good,
    Fair,
}

impl ProductCondition {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::NewWithTags => "new_with_tags",
            Self::Excellent => "excellent",
            Self::Good => "good",
            Self::Fair => "fair",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, utoipa::ToSchema)]
#[sqlx(type_name = "VARCHAR", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ProductStatus {
    Intake,
    Inspection,
    Rejected,
    Preparation,
    PhotoQueue,
    PhotoDone,
    Ready,
    Reserved,
    Sold,
    Returned,
}

impl ProductStatus {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Intake => "intake",
            Self::Inspection => "inspection",
            Self::Rejected => "rejected",
            Self::Preparation => "preparation",
            Self::PhotoQueue => "photo_queue",
            Self::PhotoDone => "photo_done",
            Self::Ready => "ready",
            Self::Reserved => "reserved",
            Self::Sold => "sold",
            Self::Returned => "returned",
        }
    }
}

#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
#[serde(tag = "product_type", rename_all = "snake_case")]
pub enum TypeDetailsInput {
    Clothing {
        fit: Option<ClothingFit>,
    },
    Footwear {
        shoe_width: Option<ShoeWidth>,
        insole_length_cm: Option<Decimal>,
    },
    Bags {
        width_cm: Option<Decimal>,
        height_cm: Option<Decimal>,
        depth_cm: Option<Decimal>,
        handle_type: Option<HandleType>,
        bag_size_label: Option<String>,
    },
    Jewelry {
        metal: Option<String>,
        stone: Option<String>,
        clasp_type: Option<String>,
    },
    Accessories,
}

impl TypeDetailsInput {
    pub fn product_type(&self) -> ProductType {
        match self {
            Self::Clothing { .. } => ProductType::Clothing,
            Self::Footwear { .. } => ProductType::Footwear,
            Self::Bags { .. } => ProductType::Bags,
            Self::Jewelry { .. } => ProductType::Jewelry,
            Self::Accessories => ProductType::Accessories,
        }
    }
}
