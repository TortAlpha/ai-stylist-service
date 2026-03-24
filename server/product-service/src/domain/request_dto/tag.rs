use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct CreateTagRequest {
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SetTagIdsRequest {
    pub ids: Vec<i32>,
}