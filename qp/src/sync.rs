//! Synchronization primitives for use in asynchronous contexts.
use crossbeam_queue::SegQueue;
use crossbeam_utils::Backoff;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
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
    pub fn new(permits: usize) -> Self {
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
    /// use qp::sync::Semaphore;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let binary_semaphore = Semaphore::new(1);
    ///     let permit1 = binary_semaphore.acquire().await;
    /// }
    /// ```
    pub async fn acquire(&self) -> SemaphorePermit<'_> {
        Acquire::new(self).await
    }

    /// Tries to acquire a permit from the semaphore if there is one available.
    ///
    /// Returns `None` immediately if there are no idle resources available in the pool.
    ///
    /// # Examples
    ///
    /// ```
    /// use qp::sync::Semaphore;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let binary_semaphore = Semaphore::new(1);
    ///     let permit1 = binary_semaphore.try_acquire();
    ///     assert!(permit1.is_some());
    ///     let permit2 = binary_semaphore.try_acquire();
    ///     assert!(permit2.is_none());
    ///     let permit3 = binary_semaphore.try_acquire();
    ///     assert!(permit3.is_none());
    ///     drop(permit1);
    ///     let permit4 = binary_semaphore.try_acquire();
    ///     assert!(permit4.is_some());
    /// }
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
        self.semaphore.permits.fetch_add(1, Ordering::Relaxed);
        if let Some(waker) = self.semaphore.waiters.pop() {
            waker.wake();
        }
    }
}

impl<'a> SemaphorePermit<'a> {
    fn new(semaphore: &'a Semaphore) -> Self {
        Self { semaphore }
    }
}

struct Acquire<'a> {
    semaphore: &'a Semaphore,
}

impl<'a> Future for Acquire<'a> {
    type Output = SemaphorePermit<'a>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.semaphore.try_acquire() {
            Some(permit) => Poll::Ready(permit),
            None => {
                self.semaphore.waiters.push(cx.waker().clone());
                Poll::Pending
            }
        }
    }
}

impl<'a> Acquire<'a> {
    fn new(semaphore: &'a Semaphore) -> Self {
        Self { semaphore }
    }
}
