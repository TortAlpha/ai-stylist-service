use serde::Serialize;

use super::super::category::Category;

/// Full category response (admin + public list endpoints)
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct CategoryFullResponse {
    pub id: i32,
    pub name: String,
    pub code: String,
    pub parent_id: Option<i32>,
    pub gender: String,
    pub product_type: String,
    pub size_group: String,
}

/// Short category info embedded in product responses
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct CategoryResponse {
    pub name: String,
    pub parent_category: Option<String>,
    pub gender: String,
    pub size_group: String,
}

impl From<Category> for CategoryFullResponse {
    fn from(c: Category) -> Self {
        Self {
            id: c.id,
            name: c.name,
            code: c.code,
            parent_id: c.parent_id,
            gender: c.gender.as_str().to_string(),
            product_type: c.product_type.as_str().to_string(),
            size_group: c.size_group.as_str().to_string(),
        }
    }
}
