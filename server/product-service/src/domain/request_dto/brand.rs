use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct CreateBrandRequest {
    pub name: String,
    pub code: String,
    pub tier: String,
    pub country: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateBrandRequest {
    pub name: Option<String>,
    pub code: Option<String>,
    pub tier: Option<String>,
    pub country: Option<String>,
}