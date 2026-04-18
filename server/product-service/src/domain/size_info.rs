use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use super::category::SizeGroup;

/// Unified sizing data — interpretation depends on the category's `size_group`.
///
/// | size_group       | size_value         | size_value2   | size_system   | measurement_cm         |
/// |------------------|--------------------|---------------|---------------|------------------------|
/// | letter           | XS/S/M/L/XL/XXL   |               |               |                        |
/// | letter_or_numeric| S/M/L OR 36/38/40  |               | EU/US/UK/IT/FR|                        |
/// | waist_length     | 28/30/32 (waist)   | 30/32 (length)|               |                        |
/// | shoe             | 42/9/8 (size)      |               | EU/US/UK      |                        |
/// | ring             | 7/54/N (size)      |               | US/EU/UK      |                        |
/// | measurement_cm   |                    |               |               | cm (belt, bracelet...) |
/// | hat              | S/M/L (optional)   |               |               | head circumference cm  |
/// | dimensions       | (use bag w/h/d)    |               |               |                        |
/// | one_size         | (no sizing)        |               |               |                        |
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SizeInfo {
    pub product_id: Uuid,
    pub size_value: Option<String>,
    pub size_value2: Option<String>,
    pub size_system: Option<SizeSystem>,
    pub measurement_cm: Option<Decimal>,
}

#[derive(
    Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, utoipa::ToSchema,
)]
#[sqlx(type_name = "VARCHAR", rename_all = "UPPERCASE")]
#[serde(rename_all = "UPPERCASE")]
pub enum SizeSystem {
    #[default]
    EU,
    US,
    UK,
    IT,
    FR,
}

impl SizeSystem {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::EU => "EU",
            Self::US => "US",
            Self::UK => "UK",
            Self::IT => "IT",
            Self::FR => "FR",
        }
    }
}

/// Produces a human-readable size label for preview cards based on size_group.
pub fn make_size_label(size_group: &SizeGroup, info: &SizeInfo) -> Option<String> {
    match size_group {
        SizeGroup::Letter => info.size_value.clone(),

        SizeGroup::LetterOrNumeric => match (&info.size_value, &info.size_system) {
            (Some(v), Some(sys)) => Some(format!("{} {}", v, sys.as_str())),
            (Some(v), None) => Some(v.clone()),
            _ => None,
        },

        SizeGroup::WaistLength => match (&info.size_value, &info.size_value2) {
            (Some(w), Some(l)) => Some(format!("W{}/L{}", w, l)),
            (Some(w), None) => Some(format!("W{}", w)),
            _ => None,
        },

        SizeGroup::Shoe => match (&info.size_value, &info.size_system) {
            (Some(v), Some(sys)) => Some(format!("{} {}", v, sys.as_str())),
            (Some(v), None) => Some(v.clone()),
            _ => None,
        },

        SizeGroup::Ring => match (&info.size_value, &info.size_system) {
            (Some(v), Some(sys)) => Some(format!("{} {}", v, sys.as_str())),
            (Some(v), None) => Some(v.clone()),
            _ => None,
        },

        SizeGroup::MeasurementCm => info.measurement_cm.map(|cm| format!("{} cm", cm)),

        SizeGroup::Hat => match (&info.size_value, &info.measurement_cm) {
            (Some(v), _) => Some(v.clone()),
            (None, Some(cm)) => Some(format!("{} cm", cm)),
            _ => None,
        },

        SizeGroup::Dimensions | SizeGroup::OneSize => None,
    }
}
