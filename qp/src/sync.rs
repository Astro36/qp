//! Synchronization primitives for use in asynchronous contexts.
use crossbeam_queue::SegQueue;
use crossbeam_utils::Backoff;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::task::{Context, Poll, Waker};

/// Counting semaphore performing asynchronous permit acquisition.
pub struct Semaphore {
    permits: AtomicUsize,
    waiters: SegQueue<Waker>,
}

impl Semaphore {
    /// Creates a new semaphore with the initial number of permits.
    ///
    /// # Examples
    ///
    /// ```
    /// # use qp::sync::Semaphore;
    /// let binary_semaphore = Semaphore::new(1);
    /// ```
    pub const fn new(permits: usize) -> Self {
        debug_assert!(permits >= 1);
        Self {
            permits: AtomicUsize::new(permits),
            waiters: SegQueue::new(),
        }
    }

    /// Acquires a permit from the semaphore.
    ///
    /// # Examples
    ///
    /// ```
    /// # use qp::sync::Semaphore;
    /// # #[tokio::main]
    /// # async fn main() {
    /// let binary_semaphore = Semaphore::new(1);
    /// assert_eq!(binary_semaphore.available_permits(), 1);
    /// let permit = binary_semaphore.acquire().await;
    /// assert_eq!(binary_semaphore.available_permits(), 0);
    /// drop(permit);
    /// assert_eq!(binary_semaphore.available_permits(), 1);
    /// # }
    /// ```
    pub async fn acquire(&self) -> SemaphorePermit<'_> {
        Acquire::new(self).await
    }

    /// Returns the current number of available permits.
    ///
    /// # Examples
    ///
    /// ```
    /// # use qp::sync::Semaphore;
    /// # #[tokio::main]
    /// # async fn main() {
    /// let binary_semaphore = Semaphore::new(1);
    /// assert_eq!(binary_semaphore.available_permits(), 1);
    /// let permit = binary_semaphore.acquire().await;
    /// assert_eq!(binary_semaphore.available_permits(), 0);
    /// # }
    /// ```
    pub fn available_permits(&self) -> usize {
        self.permits.load(Ordering::Acquire)
    }

    /// Tries to acquire a permit from the semaphore if there is one available.
    ///
    /// Returns `None` immediately if there are no idle resources available in the pool.
    ///
    /// # Examples
    ///
    /// ```
    /// # use qp::sync::Semaphore;
    /// # #[tokio::main]
    /// # async fn main() {
    /// let binary_semaphore = Semaphore::new(1);
    /// let permit1 = binary_semaphore.try_acquire();
    /// assert!(permit1.is_some());
    /// let permit2 = binary_semaphore.try_acquire();
    /// assert!(permit2.is_none());
    /// drop(permit1);
    /// let permit3 = binary_semaphore.try_acquire();
    /// assert!(permit3.is_some());
    /// # }
    pub fn try_acquire(&self) -> Option<SemaphorePermit> {
        let backoff = Backoff::new();
        let mut permits = self.permits.load(Ordering::Relaxed);
        loop {
            if permits == 0 {
                return None;
            }
            match self.permits.compare_exchange_weak(
                permits,
                permits - 1,
                Ordering::Acquire,
                Ordering::Relaxed,
            ) {
                Ok(_) => return Some(SemaphorePermit::new(self)),
                Err(changed) => permits = changed,
            }
            backoff.spin();
        }
    }
}

/// A permit from the semaphore.
///
/// This type is created by the [`Semaphore::acquire`] method and related methods.
pub struct SemaphorePermit<'a> {
    semaphore: &'a Semaphore,
}

impl Drop for SemaphorePermit<'_> {
    fn drop(&mut self) {
        self.semaphore.permits.fetch_add(1, Ordering::Release);
        if let Some(waker) = self.semaphore.waiters.pop() {
            waker.wake();
        }
    }
}

impl<'a> SemaphorePermit<'a> {
    const fn new(semaphore: &'a Semaphore) -> Self {
        Self { semaphore }
    }
}

struct Acquire<'a> {
    semaphore: &'a Semaphore,
    waiting: AtomicBool,
}

impl<'a> Future for Acquire<'a> {
    type Output = SemaphorePermit<'a>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.semaphore.try_acquire() {
            Some(permit) => Poll::Ready(permit),
            None => {
                if !self.waiting.load(Ordering::SeqCst) {
                    self.waiting.store(true, Ordering::SeqCst);
                    self.semaphore.waiters.push(cx.waker().clone());
                }
                Poll::Pending
            }
        }
    }
}

impl<'a> Acquire<'a> {
    const fn new(semaphore: &'a Semaphore) -> Self {
        Self {
            semaphore,
            waiting: AtomicBool::new(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{sync::Arc, time::Duration};
    #[tokio::test]
    async fn test_abort_acquire() {
        let sem = Arc::new(Semaphore::new(1));
        //assert_eq!(sem.waiting_count(), 0);
        // Grab the only permit for the semaphore
        let permit = sem.try_acquire().unwrap();
        // Spawn two tokio tasks waiting for the semaphore to become
        // available. The first one times out after 1ms and the second
        // after 3ms.
        let a = {
            let sem = sem.clone();
            tokio::spawn(tokio::time::timeout(Duration::from_millis(1), async move {
                sem.acquire().await;
            }))
        };
        tokio::time::sleep(Duration::from_millis(1)).await;
        let b = {
            let sem = sem.clone();
            tokio::spawn(tokio::time::timeout(Duration::from_millis(2), async move {
                let _ = sem.acquire().await;
            }))
        };
        tokio::time::sleep(Duration::from_millis(1)).await;
        // The first task should now be timed out.
        drop(permit);
        assert!(a.await.unwrap().is_err());
        assert!(b.await.unwrap().is_ok());
        assert_eq!(sem.waiters.len(), 0);
    }
}
