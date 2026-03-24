use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct BrandShortResponse {
    pub id: i32,
    pub name: String,
    pub tier: String,
}