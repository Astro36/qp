use crossbeam_queue::SegQueue;
use crossbeam_utils::Backoff;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::task::{Context, Poll, Waker};

pub struct Semaphore {
    permits: AtomicUsize,
    waiters: SegQueue<Waker>,
}

impl Semaphore {
    pub fn new(permits: usize) -> Self {
        Self {
            permits: AtomicUsize::new(permits),
            waiters: SegQueue::new(),
        }
    }

    pub fn acquire(&self) -> Acquire {
        Acquire::new(self)
    }

    pub fn try_acquire(&self) -> Option<SemaphorePermit> {
        let backoff = Backoff::new();
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
            backoff.spin();
        }
    }
}

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
    fn new(semaphore: &'a Semaphore) -> Self {
        Self { semaphore }
    }
}

pub struct Acquire<'a> {
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
    pub fn new(semaphore: &'a Semaphore) -> Self {
        Self { semaphore }
    }
}
