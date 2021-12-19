use crossbeam_utils::Backoff;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::task::{Context, Poll};

pub struct Semaphore {
    permits: AtomicUsize,
}

impl Semaphore {
    pub fn new(permits: usize) -> Self {
        Self {
            permits: AtomicUsize::new(permits),
        }
    }

    pub fn acquire(&self) -> Acquire {
        Acquire::new(self)
    }

    pub fn try_acquire(&self) -> Option<SemaphorePermit> {
        let mut permits = self.permits.load(Ordering::Acquire);
        loop {
            if permits == 0 {
                return None;
            }
            match self.permits.compare_exchange_weak(
                permits,
                permits - 1,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => return Some(SemaphorePermit::new(self)),
                Err(changed) => permits = changed,
            }
        }
    }
}

pub struct SemaphorePermit<'a> {
    semaphore: &'a Semaphore,
}

impl Drop for SemaphorePermit<'_> {
    fn drop(&mut self) {
        self.semaphore.permits.fetch_add(1, Ordering::Release);
    }
}

impl<'a> SemaphorePermit<'a> {
    fn new(semaphore: &'a Semaphore) -> Self {
        Self { semaphore }
    }
}

pub struct Acquire<'a> {
    semaphore: &'a Semaphore,
    backoff: Backoff,
}

impl<'a> Future for Acquire<'a> {
    type Output = SemaphorePermit<'a>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let permits = self.semaphore.permits.load(Ordering::Acquire);
        if permits > 0
            && self
                .semaphore
                .permits
                .compare_exchange_weak(permits, permits - 1, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
        {
            Poll::Ready(SemaphorePermit::new(self.semaphore))
        } else {
            self.backoff.spin();
            cx.waker().clone().wake();
            Poll::Pending
        }
    }
}

impl<'a> Acquire<'a> {
    pub fn new(semaphore: &'a Semaphore) -> Self {
        Self {
            semaphore,
            backoff: Backoff::new(),
        }
    }
}
