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
    pub fn manager(&self) -> &M {
        &self.inner.manager
    }

    /// Returns the number of resources the pool can manage.
    pub fn max_size(&self) -> usize {
        self.inner.resources.capacity()
    }

    /// Reserves the resources for at least `size` more resources to be acquired from the pool.
    pub async fn reserve(&self, size: usize) -> Result<(), M::Error> {
        debug_assert!(size >= 1);
        self.inner.reserve(size).await
    }

    /// Returns the current number of available resources.
    pub fn size(&self) -> usize {
        self.inner.semaphore.available_permits()
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
    /// This function consumes `pooled` to prevent double [`take`](Pooled::take).
    pub fn take(mut pooled: Self) -> M::Output {
        pooled.resource.take().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::atomic::{AtomicUsize, Ordering}, time::Duration};

    use super::*;

    #[derive(Default)]
    struct Manager {
        creation_counter: AtomicUsize,
    }

    #[async_trait::async_trait]
    impl Manage for Manager {
        type Output = ();
        type Error = ();
        async fn try_create(&self) -> Result<Self::Output, Self::Error> {
            assert_eq!(self.creation_counter.fetch_add(1, Ordering::Relaxed), 0);
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_abort_acquire() {
        let pool = Pool::new(Manager::default(), 1);
        // Grab the only object from the pool
        let obj = pool.acquire().await;
        // Spawn two tokio tasks waiting for an object.
        // The first one times out after 1ms and the second
        // after 3ms.
        let a = {
            let pool = pool.clone();
            tokio::spawn(tokio::time::timeout(Duration::from_millis(1), async move {
                let _ = pool.acquire().await;
            }))
        };
        tokio::time::sleep(Duration::from_millis(1)).await;
        let b = {
            let pool = pool.clone();
            tokio::spawn(tokio::time::timeout(Duration::from_millis(2), async move {
                let _ = pool.acquire().await;
            }))
        };
        tokio::time::sleep(Duration::from_millis(1)).await;
        // The first task should now be timed out.
        drop(obj);
        // The first task should have timed out and returned an error
        // while the second task should've gotten hold of the object.
        assert!(a.await.unwrap().is_err());
        assert!(b.await.unwrap().is_ok());
    }
}
