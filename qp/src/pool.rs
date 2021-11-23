use crate::error::{Error, Result};
use async_trait::async_trait;
use futures::future::TryFutureExt;
use parking_lot::Mutex;
use std::collections::VecDeque;
use std::error::Error as StdError;
use std::ops::{Deref, DerefMut};
use std::result::Result as StdResult;
use std::sync::Arc;
use tokio::sync::Semaphore;

#[async_trait]
pub trait ResourceFactory: Sync {
    type Output: Send + Sync;
    type Error: StdError + Send + Sync + 'static;

    async fn try_create_resource(&self) -> StdResult<Self::Output, Self::Error>;

    async fn validate(&self, _resource: &Self::Output) -> bool {
        true
    }
}

pub struct ResourceGuard<'a, F: ResourceFactory> {
    pool: &'a Inner<F>,
    resource: Option<F::Output>,
}

impl<F: ResourceFactory> Deref for ResourceGuard<'_, F> {
    type Target = F::Output;

    fn deref(&self) -> &Self::Target {
        self.resource.as_ref().unwrap()
    }
}

impl<F: ResourceFactory> DerefMut for ResourceGuard<'_, F> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.resource.as_mut().unwrap()
    }
}

impl<F: ResourceFactory> Drop for ResourceGuard<'_, F> {
    fn drop(&mut self) {
        if let Some(resource) = self.resource.take() {
            self.pool.push_resource(resource);
            self.pool.semaphore.add_permits(1);
        }
    }
}

pub struct Pool<F: ResourceFactory> {
    inner: Arc<Inner<F>>,
}

impl<F: ResourceFactory> Clone for Pool<F> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<F: ResourceFactory> Pool<F> {
    pub fn new(factory: F, max_size: usize) -> Self {
        Self {
            inner: Arc::new(Inner {
                factory,
                resources: Mutex::new(VecDeque::with_capacity(max_size)),
                semaphore: Semaphore::new(max_size),
            }),
        }
    }

    pub async fn acquire(&self) -> Result<ResourceGuard<'_, F>> {
        self.inner.acquire().await
    }
}

struct Inner<F: ResourceFactory> {
    factory: F,
    resources: Mutex<VecDeque<F::Output>>,
    semaphore: Semaphore,
}

impl<F: ResourceFactory> Inner<F> {
    pub async fn acquire(&self) -> Result<ResourceGuard<'_, F>> {
        // A `Semaphore::acquire` can only fail if the semaphore has been closed.
        self.semaphore
            .acquire()
            .map_err(|_| Error::PoolClosed)
            .await?
            .forget();
        Ok(ResourceGuard {
            pool: self,
            resource: Some(self.pop_or_create_resource().await?),
        })
    }

    async fn create_resource(&self) -> Result<F::Output> {
        self.factory
            .try_create_resource()
            .map_err(|e| Error::Resource(Box::new(e)))
            .await
    }

    fn pop_resource(&self) -> Option<F::Output> {
        self.resources.lock().pop_front()
    }

    async fn pop_or_create_resource(&self) -> Result<F::Output> {
        while let Some(resource) = self.pop_resource() {
            if self.factory.validate(&resource).await {
                return Ok(resource);
            }
        }
        self.create_resource().await
    }

    fn push_resource(&self, resource: F::Output) {
        self.resources.lock().push_back(resource);
    }
}

pub fn take_resource<F: ResourceFactory>(mut guard: ResourceGuard<'_, F>) -> F::Output {
    guard.pool.semaphore.add_permits(1);
    guard.resource.take().unwrap()
}
