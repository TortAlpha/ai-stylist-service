use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct CreatePurchaseLocationRequest {
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct UpdatePurchaseLocationRequest {
    pub name: String,
}
