use serde::Serialize;

/// Single tag response (style, vibe, or season)
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TagResponse {
    pub id: i32,
    pub name: String,
}

/// Aggregated tags embedded in product responses
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ProductTagsResponse {
    pub styles: Vec<String>,
    pub vibes: Vec<String>,
    pub seasons: Vec<String>,
}

impl From<crate::domain::tags::StyleTag> for TagResponse {
    fn from(t: crate::domain::tags::StyleTag) -> Self {
        Self {
            id: t.id,
            name: t.name,
        }
    }
}

impl From<crate::domain::tags::VibeTag> for TagResponse {
    fn from(t: crate::domain::tags::VibeTag) -> Self {
        Self {
            id: t.id,
            name: t.name,
        }
    }
}

impl From<crate::domain::tags::Season> for TagResponse {
    fn from(t: crate::domain::tags::Season) -> Self {
        Self {
            id: t.id,
            name: t.name,
        }
    }
}
