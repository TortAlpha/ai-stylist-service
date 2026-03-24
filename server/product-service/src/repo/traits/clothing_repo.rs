use async_trait::async_trait;

#[async_trait]
pub trait ClothingRepository: Send + Sync {
}
