use std::time::Duration;

use async_trait::async_trait;

#[cfg_attr(any(test, feature = "test-mocks"), mockall::automock)]
#[async_trait]
pub trait ImageStorage: Send + Sync {
}


