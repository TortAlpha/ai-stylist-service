use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Category {
    pub id: i32,
    pub name: String,
    pub code: String,
    pub parent_id: Option<i32>,
    pub gender: Gender,
    pub product_type: ProductType,
    pub size_group: SizeGroup,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, utoipa::ToSchema)]
#[sqlx(type_name = "VARCHAR", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum Gender {
    Male,
    Female,
    Unisex,
    Kids,
}

impl Gender {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Male => "male",
            Self::Female => "female",
            Self::Unisex => "unisex",
            Self::Kids => "kids",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, utoipa::ToSchema)]
#[sqlx(type_name = "VARCHAR", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ProductType {
    Clothing,
    Footwear,
    Bags,
    Jewelry,
    Accessories,
}

impl ProductType {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Clothing => "clothing",
            Self::Footwear => "footwear",
            Self::Bags => "bags",
            Self::Jewelry => "jewelry",
            Self::Accessories => "accessories",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, utoipa::ToSchema)]
#[sqlx(type_name = "VARCHAR", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum SizeGroup {
    Letter,
    LetterOrNumeric,
    WaistLength,
    Shoe,
    Ring,
    MeasurementCm,
    Dimensions,
    Hat,
    OneSize,
}

impl SizeGroup {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Letter => "letter",
            Self::LetterOrNumeric => "letter_or_numeric",
            Self::WaistLength => "waist_length",
            Self::Shoe => "shoe",
            Self::Ring => "ring",
            Self::MeasurementCm => "measurement_cm",
            Self::Dimensions => "dimensions",
            Self::Hat => "hat",
            Self::OneSize => "one_size",
        }
    }
}
