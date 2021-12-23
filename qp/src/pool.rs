use crate::resource::Manage;
use crate::sync::{Semaphore, SemaphorePermit};
use crossbeam_queue::ArrayQueue;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

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
    pub fn new(manager: M, max_size: usize) -> Self {
        Self {
            inner: Arc::new(Inner {
                manager,
                resources: ArrayQueue::new(max_size),
                semaphore: Semaphore::new(max_size),
            }),
        }
    }

    pub async fn acquire(&self) -> Result<Pooled<'_, M>, M::Error> {
        self.inner.acquire().await
    }

    pub async fn acquire_unchecked(&self) -> Result<Pooled<'_, M>, M::Error> {
        self.inner.acquire_unchecked().await
    }

    pub fn get_manager(&self) -> &M {
        &self.inner.manager
    }

    pub async fn reserve(&self, size: usize) {
        self.inner.reserve(size).await;
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
                return Ok(Pooled {
                    pool: self,
                    resource: Some(resource),
                    _permit: permit,
                });
            }
        }
        Ok(Pooled {
            pool: self,
            resource: Some(self.manager.try_create().await?),
            _permit: permit,
        })
    }

    async fn acquire_unchecked(&self) -> Result<Pooled<'_, M>, M::Error> {
        let permit = self.semaphore.acquire().await;
        Ok(Pooled {
            pool: self,
            resource: Some(match self.resources.pop() {
                Some(resource) => resource,
                None => self.manager.try_create().await?,
            }),
            _permit: permit,
        })
    }

    async fn reserve(&self, size: usize) {
        let mut resources = Vec::with_capacity(size);
        for _ in 0..size {
            resources.push(self.acquire_unchecked().await);
        }
    }
}

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
    pub async fn is_valid(pooled: &Self) -> bool {
        pooled.pool.manager.validate(pooled).await
    }

    pub fn take(mut pooled: Self) -> M::Output {
        pooled.resource.take().unwrap()
    }
}
