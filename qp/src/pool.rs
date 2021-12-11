use crate::error::{Error, Result};
use crate::resource::Factory;
use crossbeam_queue::ArrayQueue;
use crossbeam_utils::Backoff;
use futures_lite::future;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

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
            let _ = self.pool.resources.push(resource);
            self.pool.idle.fetch_add(1, Ordering::SeqCst);
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
                idle: AtomicUsize::new(max_size),
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
    idle: AtomicUsize,
}

impl<F: Factory> Inner<F> {
    pub async fn acquire(&self) -> Result<Pooled<'_, F>> {
        let backoff = Backoff::new();
        loop {
            let idle = self.idle.load(Ordering::SeqCst);
            if idle > 0
                && self
                    .idle
                    .compare_exchange_weak(idle, idle - 1, Ordering::SeqCst, Ordering::SeqCst)
                    .is_ok()
            {
                break;
            }
            backoff.spin();
            future::yield_now().await;
        }
        Ok(Pooled {
            pool: self,
            resource: Some(self.pop_or_create_resource().await?),
        })
    }

    pub async fn acquire_unchecked(&self) -> Result<Pooled<'_, F>> {
        let backoff = Backoff::new();
        let mut idle = self.idle.load(Ordering::SeqCst);
        loop {
            match self.idle.compare_exchange_weak(
                idle,
                idle - 1,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => break,
                Err(changed) => idle = changed,
            }
            backoff.spin();
            future::yield_now().await;
        }
        Ok(Pooled {
            pool: self,
            resource: Some(self.pop_or_create_resource_unchecked().await?),
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
    guard.pool.idle.fetch_add(1, Ordering::SeqCst);
    guard.resource.take().unwrap()
}
