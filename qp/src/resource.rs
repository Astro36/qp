use async_trait::async_trait;
use std::error::Error;

#[async_trait]
pub trait Factory: Sync {
    type Output: Send + Sync;
    type Error: Error + Send + Sync + 'static;

    async fn try_create_resource(&self) -> Result<Self::Output, Self::Error>;

    async fn validate(&self, _resource: &Self::Output) -> bool {
        true
    }
}
