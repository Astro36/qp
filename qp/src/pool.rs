use crate::resource::Manage;
use crate::sync::{Semaphore, SemaphorePermit};
use crossbeam_queue::ArrayQueue;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

/// An async resource pool.
pub struct Pool<M: Manage> {
    inner: Arc<Inner<M>>,
}

impl<M: Manage> Clone for Pool<M> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<M: Manage> Pool<M> {
    /// Creates a new `Pool` with the given size.
    pub fn new(manager: M, max_size: usize) -> Self {
        debug_assert!(max_size >= 1);
        Self {
            inner: Arc::new(Inner {
                manager,
                resources: ArrayQueue::new(max_size),
                semaphore: Semaphore::new(max_size),
            }),
        }
    }

    /// Acquires a resource from the pool.
    pub async fn acquire(&self) -> Result<Pooled<'_, M>, M::Error> {
        self.inner.acquire().await
    }

    /// Acquires a resource from the pool without checking whether the resource is valid.
    pub async fn acquire_unchecked(&self) -> Result<Pooled<'_, M>, M::Error> {
        self.inner.acquire_unchecked().await
    }

    /// Returns the resource manager of the pool.
    pub fn get_manager(&self) -> &M {
        &self.inner.manager
    }

    /// Reserves the resources for at least `size` more resources to be acquired from the pool.
    pub async fn reserve(&self, size: usize) -> Result<(), M::Error> {
        debug_assert!(size >= 1);
        self.inner.reserve(size).await
    }
}

struct Inner<M: Manage> {
    manager: M,
    resources: ArrayQueue<M::Output>,
    semaphore: Semaphore,
}

impl<M: Manage> Inner<M> {
    async fn acquire(&self) -> Result<Pooled<'_, M>, M::Error> {
        let permit = self.semaphore.acquire().await;
        while let Some(resource) = self.resources.pop() {
            if self.manager.validate(&resource).await {
                return Ok(self.make_pooled(resource, permit));
            }
        }
        Ok(self.make_pooled(self.manager.try_create().await?, permit))
    }

    async fn acquire_unchecked(&self) -> Result<Pooled<'_, M>, M::Error> {
        let permit = self.semaphore.acquire().await;
        Ok(self.make_pooled(
            match self.resources.pop() {
                Some(resource) => resource,
                None => self.manager.try_create().await?,
            },
            permit,
        ))
    }

    fn make_pooled<'a>(
        &'a self,
        resource: M::Output,
        permit: SemaphorePermit<'a>,
    ) -> Pooled<'a, M> {
        Pooled {
            pool: self,
            resource: Some(resource),
            _permit: permit,
        }
    }

    async fn reserve(&self, size: usize) -> Result<(), M::Error> {
        debug_assert!(size <= self.resources.capacity());
        let mut resources = Vec::with_capacity(size);
        for _ in 0..size {
            resources.push(self.acquire_unchecked().await?);
        }
        Ok(())
    }
}

/// An acquired resource from the pool.
///
/// This type is created by the [`Pool::acquire`] method and related methods.
pub struct Pooled<'a, M: Manage> {
    pool: &'a Inner<M>,
    resource: Option<M::Output>,
    _permit: SemaphorePermit<'a>,
}

impl<M: Manage> Deref for Pooled<'_, M> {
    type Target = M::Output;

    fn deref(&self) -> &Self::Target {
        self.resource.as_ref().unwrap()
    }
}

impl<M: Manage> DerefMut for Pooled<'_, M> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.resource.as_mut().unwrap()
    }
}

impl<M: Manage> Drop for Pooled<'_, M> {
    fn drop(&mut self) {
        if let Some(resource) = self.resource.take() {
            let _ = self.pool.resources.push(resource);
        }
    }
}

impl<M: Manage> Pooled<'_, M> {
    /// Returns `true` if the given resource is valid.
    pub async fn is_valid(pooled: &Self) -> bool {
        pooled.pool.manager.validate(pooled).await
    }

    /// Takes the raw resource out of the [`Pooled`], leaving a `None` in its place.
    ///
    /// This function consumes `pooled` to prevent double `take`.
    pub fn take(mut pooled: Self) -> M::Output {
        pooled.resource.take().unwrap()
    }
}
