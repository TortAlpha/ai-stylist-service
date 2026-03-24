use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct CategoryResponse {
    pub name: String,
    pub parent_category: Option<String>,
    pub gender: String,
}