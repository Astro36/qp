//! A module for managed resources.
use async_trait::async_trait;

/// An interface for managing resources used by [`Pool`](crate::Pool).
#[async_trait]
pub trait Manage: Sync {
    /// The type of resource managed by [`Pool`](crate::Pool).
    type Output: Send + Sync;

    /// The type of error from a resource.
    type Error;

    /// Tries to create a resource using the manager.
    async fn try_create(&self) -> Result<Self::Output, Self::Error>;

    /// Returns `true` if the given resource is valid.
    async fn validate(&self, _resource: &Self::Output) -> bool {
        true
    }
}
