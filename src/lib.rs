use parking_lot::Mutex;
use std::collections::VecDeque;
use std::error;
use std::fmt::{self, Debug, Formatter};
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use tokio::sync::Semaphore;

pub type Error = Box<dyn error::Error>;

pub trait Resource: Debug + Sized {
    fn try_new() -> Result<Self, Error>;
}

pub struct Pooled<'a, T: Resource> {
    pool: &'a PoolInner<T>,
    resource: Option<T>,
}

impl<T: Resource> Debug for Pooled<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.resource)
    }
}

impl<T: Resource> Deref for Pooled<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.resource.as_ref().unwrap()
    }
}

impl<T: Resource> DerefMut for Pooled<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
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
    pub fn new(capacity: usize) -> Result<Self, Error> {
        Ok(Pool {
            inner: Arc::new(PoolInner {
                resources: Mutex::new(
                    (0..capacity)
                        .map(|_| Resource::try_new())
                        .collect::<Result<VecDeque<T>, Error>>()?,
                ),
                semaphore: Semaphore::new(capacity),
            }),
        })
    }

    pub async fn acquire(&self) -> Result<Pooled<'_, T>, Error> {
        self.inner.acquire().await
    }
}

struct PoolInner<T: Resource> {
    resources: Mutex<VecDeque<T>>,
    semaphore: Semaphore,
}

impl<T: Resource> PoolInner<T> {
    pub async fn acquire(&self) -> Result<Pooled<'_, T>, Error> {
        let permit = self.semaphore.acquire().await?;
        permit.forget();
        Ok(Pooled {
            pool: self,
            resource: Some(self.resources.lock().pop_front().unwrap()),
        })
    }
}
