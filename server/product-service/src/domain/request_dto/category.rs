use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct CreateCategoryRequest {
    pub name: String,
    pub code: String,
    pub parent_id: Option<i32>,
    pub gender: String,
}