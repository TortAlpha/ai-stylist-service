use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ProductTagsResponse {
    pub styles: Vec<String>,
    pub vibes: Vec<String>,
    pub seasons: Vec<String>,
}