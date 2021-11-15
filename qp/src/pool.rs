use crate::error::{Error, Result};
use parking_lot::Mutex;
use std::collections::VecDeque;
use std::ops::{Deref, DerefMut};
use std::result::Result as StdResult;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio::time::timeout;

pub trait Resource: Sized {
    type Error;

    fn try_new() -> StdResult<Self, Self::Error>;
}

pub struct Pooled<'a, T: Resource> {
    pool: &'a Inner<T>,
    resource: Option<T>,
}

impl<T: Resource> Deref for Pooled<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.resource.as_ref().unwrap()
    }
}

impl<T: Resource> DerefMut for Pooled<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.resource.as_mut().unwrap()
    }
}

impl<T: Resource> Drop for Pooled<'_, T> {
    fn drop(&mut self) {
        self.pool
            .resources
            .lock()
            .push_back(self.resource.take().unwrap());
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
    pub fn try_new(capacity: usize, timeout: Duration) -> Result<Self> {
        Ok(Pool {
            inner: Arc::new(Inner {
                capacity,
                resources: Mutex::new(
                    (0..capacity)
                        .map(|_| Resource::try_new())
                        .filter_map(|resource| resource.ok())
                        .collect::<VecDeque<T>>(),
                ),
                semaphore: Semaphore::new(capacity),
                timeout,
            }),
        })
    }

    pub async fn acquire(&self) -> Result<Pooled<'_, T>> {
        self.inner.acquire().await
    }
}

struct Inner<T: Resource> {
    capacity: usize,
    resources: Mutex<VecDeque<T>>,
    semaphore: Semaphore,
    timeout: Duration,
}

impl<T: Resource> Inner<T> {
    pub async fn acquire(&self) -> Result<Pooled<'_, T>> {
        // A `Semaphore::acquire` can only fail if the semaphore has been closed.
        let permit = timeout(self.timeout, self.semaphore.acquire())
            .await
            .map_err(|_| Error::PoolTimedOut)?
            .map_err(|_| Error::PoolClosed)?;
        permit.forget();
        let mut resources = self.resources.lock();
        /* match resources.pop_front() {
            Some(resource) => {}
            None => {}
        } */
        Ok(Pooled {
            pool: self,
            resource: Some(resources.pop_front().unwrap()),
        })
    }
}
