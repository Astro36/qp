use crate::error::{Error, Result};
use crate::resource::Factory;
use futures_util::future::TryFutureExt;
use parking_lot::Mutex;
use std::collections::VecDeque;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use tokio::sync::Semaphore;

pub struct Pooled<'a, F: Factory> {
    pool: &'a Inner<F>,
    resource: Option<F::Output>,
}

impl<F: Factory> Deref for Pooled<'_, F> {
    type Target = F::Output;

    fn deref(&self) -> &Self::Target {
        self.resource.as_ref().unwrap()
    }
}

impl<F: Factory> DerefMut for Pooled<'_, F> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.resource.as_mut().unwrap()
    }
}

impl<F: Factory> Drop for Pooled<'_, F> {
    fn drop(&mut self) {
        if let Some(resource) = self.resource.take() {
            self.pool.push_resource(resource);
            self.pool.semaphore.add_permits(1);
        }
    }
}

pub struct Pool<F: Factory> {
    inner: Arc<Inner<F>>,
}

impl<F: Factory> Clone for Pool<F> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<F: Factory> Pool<F> {
    pub fn new(factory: F, max_size: usize) -> Self {
        Self {
            inner: Arc::new(Inner {
                factory,
                resources: Mutex::new(VecDeque::with_capacity(max_size)),
                semaphore: Semaphore::new(max_size),
            }),
        }
    }

    pub async fn acquire(&self) -> Result<Pooled<'_, F>> {
        self.inner.acquire().await
    }
}

struct Inner<F: Factory> {
    factory: F,
    resources: Mutex<VecDeque<F::Output>>,
    semaphore: Semaphore,
}

impl<F: Factory> Inner<F> {
    pub async fn acquire(&self) -> Result<Pooled<'_, F>> {
        // A `Semaphore::acquire` can only fail if the semaphore has been closed.
        self.semaphore
            .acquire()
            .map_err(|_| Error::PoolClosed)
            .await?
            .forget();
        Ok(Pooled {
            pool: self,
            resource: Some(self.pop_or_create_resource().await?),
        })
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
        self.factory
            .try_create()
            .map_err(|e| Error::Resource(Box::new(e)))
            .await
    }

    fn push_resource(&self, resource: F::Output) {
        self.resources.lock().push_back(resource);
    }
}

pub fn take_resource<F: Factory>(mut guard: Pooled<'_, F>) -> F::Output {
    guard.pool.semaphore.add_permits(1);
    guard.resource.take().unwrap()
}
