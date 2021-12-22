use async_trait::async_trait;

#[async_trait]
pub trait Manage: Sync {
    type Output: Send + Sync;
    type Error;

    async fn try_create(&self) -> Result<Self::Output, Self::Error>;

    async fn validate(&self, _resource: &Self::Output) -> bool {
        true
    }
}
