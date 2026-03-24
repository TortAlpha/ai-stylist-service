use async_trait::async_trait;

#[cfg_attr(any(test, feature = "test-mocks"), mockall::automock)]
#[async_trait]
pub trait ProductRepository: Send + Sync {

}
