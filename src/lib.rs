use parking_lot::Mutex;
use std::collections::VecDeque;
use std::error;
use std::fmt::{self, Debug, Formatter};
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::Semaphore;
use tokio::time::timeout;

pub trait Resource: Debug + Sized {
    fn try_new() -> Result<Self, Box<dyn error::Error>>;
}

pub struct Pooled<'a, T: Resource> {
    pool: &'a PoolInner<T>,
    resource: Option<T>,
}

impl<T: Resource> Debug for Pooled<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.resource.as_ref().unwrap())
    }
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
    inner: Arc<PoolInner<T>>,
}

impl<T: Resource> Clone for Pool<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T: Resource> Pool<T> {
    pub fn try_new(capacity: usize, timeout: Duration) -> Result<Self, Box<dyn error::Error>> {
        Ok(Pool {
            inner: Arc::new(PoolInner {
                capacity,
                resources: Mutex::new(
                    (0..capacity)
                        .map(|_| Resource::try_new())
                        .collect::<Result<VecDeque<T>, Box<dyn error::Error>>>()?,
                ),
                semaphore: Semaphore::new(capacity),
                timeout,
            }),
        })
    }

    pub async fn acquire(&self) -> Result<Pooled<'_, T>, AcquireError> {
        self.inner.acquire().await
    }
}

struct PoolInner<T: Resource> {
    capacity: usize,
    resources: Mutex<VecDeque<T>>,
    semaphore: Semaphore,
    timeout: Duration,
}

impl<T: Resource> PoolInner<T> {
    pub async fn acquire(&self) -> Result<Pooled<'_, T>, AcquireError> {
        // A `Semaphore::acquire` can only fail if the semaphore has been closed.
        let permit = timeout(self.timeout, self.semaphore.acquire())
            .await
            .map_err(|_| AcquireError::Timeout)?
            .map_err(|_| AcquireError::Close)?;
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

#[derive(Debug, Error)]
pub enum AcquireError {
    #[error("close")]
    Close,
    #[error("timeout")]
    Timeout,
}
