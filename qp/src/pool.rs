use crate::error::{Error, Result};
use async_trait::async_trait;
use parking_lot::Mutex;
use std::collections::VecDeque;
use std::error::Error as StdError;
use std::ops::{Deref, DerefMut};
use std::result::Result as StdResult;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio::time::timeout;

#[async_trait]
pub trait Resource: Send + Sized + Sync {
    type Error: StdError + Send + Sync + 'static;

    async fn try_new() -> StdResult<Self, Self::Error>;
}

pub struct ResourceGuard<'a, T: Resource> {
    pool: &'a Inner<T>,
    resource: Option<T>,
}

impl<T: Resource> Deref for ResourceGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.resource.as_ref().unwrap()
    }
}

impl<T: Resource> DerefMut for ResourceGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.resource.as_mut().unwrap()
    }
}

impl<T: Resource> Drop for ResourceGuard<'_, T> {
    fn drop(&mut self) {
        if let Some(resource) = self.resource.take() {
            self.pool.push_resource(resource);
            self.pool.semaphore.add_permits(1);
        }
    }
}

impl<T: Resource> ResourceGuard<'_, T> {
    pub fn release(&mut self) {
        self.resource.take();
        self.pool.semaphore.add_permits(1);
    }
}

pub struct Pool<T: Resource> {
    inner: Arc<Inner<T>>,
}

impl<T: Resource> Clone for Pool<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T: Resource> Pool<T> {
    pub fn new(capacity: usize, timeout: Duration) -> Self {
        Self {
            inner: Arc::new(Inner {
                resources: Mutex::new(VecDeque::with_capacity(capacity)),
                semaphore: Semaphore::new(capacity),
                timeout,
            }),
        }
    }

    pub async fn acquire(&self) -> Result<ResourceGuard<'_, T>> {
        self.inner.acquire().await
    }
}

struct Inner<T: Resource> {
    resources: Mutex<VecDeque<T>>,
    semaphore: Semaphore,
    timeout: Duration,
}

impl<T: Resource> Inner<T> {
    pub async fn acquire(&self) -> Result<ResourceGuard<'_, T>> {
        // A `Semaphore::acquire` can only fail if the semaphore has been closed.
        timeout(self.timeout, self.semaphore.acquire())
            .await
            .map_err(|_| Error::PoolTimedOut)?
            .map_err(|_| Error::PoolClosed)?
            .forget();
        Ok(ResourceGuard {
            pool: self,
            resource: Some(self.pop_or_create_resource().await?),
        })
    }

    async fn create_resource(&self) -> Result<T> {
        T::try_new().await.map_err(|e| Error::Resource(Box::new(e)))
    }

    fn pop_resource(&self) -> Option<T> {
        self.resources.lock().pop_front()
    }

    async fn pop_or_create_resource(&self) -> Result<T> {
        match self.pop_resource() {
            Some(resource) => Ok(resource),
            None => self.create_resource().await,
        }
    }

    fn push_resource(&self, resource: T) {
        self.resources.lock().push_back(resource);
    }
}
