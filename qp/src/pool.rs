use crate::error::{Error, Result};
use crate::resource::Factory;
use crate::sync::{Semaphore, SemaphorePermit};
use crossbeam_queue::ArrayQueue;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

pub struct Pooled<'a, F: Factory> {
    pool: &'a Inner<F>,
    resource: Option<F::Output>,
    _permit: SemaphorePermit<'a>,
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
            let _ = self.pool.resources.push(resource);
        }
    }
}

impl<F: Factory> Pooled<'_, F> {
    pub async fn is_valid(&self) -> bool {
        self.pool.get_factory().validate(self).await
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
                resources: ArrayQueue::new(max_size),
                semaphore: Semaphore::new(max_size),
            }),
        }
    }

    pub async fn acquire(&self) -> Result<Pooled<'_, F>> {
        self.inner.acquire().await
    }

    pub async fn acquire_unchecked(&self) -> Result<Pooled<'_, F>> {
        self.inner.acquire_unchecked().await
    }

    pub fn get_factory(&self) -> &F {
        self.inner.get_factory()
    }
}

struct Inner<F: Factory> {
    factory: F,
    resources: ArrayQueue<F::Output>,
    semaphore: Semaphore,
}

impl<F: Factory> Inner<F> {
    pub async fn acquire(&self) -> Result<Pooled<'_, F>> {
        let permit = self.semaphore.acquire().await;
        Ok(Pooled {
            pool: self,
            resource: Some(self.pop_or_create_resource().await?),
            _permit: permit,
        })
    }

    pub async fn acquire_unchecked(&self) -> Result<Pooled<'_, F>> {
        let permit = self.semaphore.acquire().await;
        Ok(Pooled {
            pool: self,
            resource: Some(self.pop_or_create_resource_unchecked().await?),
            _permit: permit,
        })
    }

    pub fn get_factory(&self) -> &F {
        &self.factory
    }

    async fn pop_or_create_resource(&self) -> Result<F::Output> {
        while let Some(resource) = self.resources.pop() {
            if self.factory.validate(&resource).await {
                return Ok(resource);
            }
        }
        self.factory
            .try_create()
            .await
            .map_err(|e| Error::Resource(Box::new(e)))
    }

    async fn pop_or_create_resource_unchecked(&self) -> Result<F::Output> {
        match self.resources.pop() {
            Some(resource) => Ok(resource),
            None => self
                .factory
                .try_create()
                .await
                .map_err(|e| Error::Resource(Box::new(e))),
        }
    }
}

pub fn take_resource<F: Factory>(mut guard: Pooled<'_, F>) -> F::Output {
    guard.resource.take().unwrap()
}
