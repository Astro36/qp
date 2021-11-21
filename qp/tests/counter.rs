use qp::async_trait;
use qp::pool::{Pool, Resource, take};
use std::convert::Infallible;
use std::time::Duration;
use tokio::time::sleep;

const MAX_POOL_SIZE: usize = 4;
const WORKERS: usize = 8;
const ITERATIONS: usize = 16;

struct Counter {
    value: i32,
}

#[async_trait]
impl Resource for Counter {
    type Error = Infallible;

    async fn try_new() -> Result<Self, Self::Error> {
        Ok(Self::new())
    }
}

impl Counter {
    fn new() -> Self {
        Self { value: 0 }
    }

    fn get(&self) -> i32 {
        self.value
    }

    fn increase(&mut self) {
        self.value += 1;
    }
}

#[tokio::test]
async fn main() {
    let pool = Pool::<Counter>::new(MAX_POOL_SIZE);

    let handles = (0..WORKERS)
        .map(|_| {
            let pool = pool.clone();
            tokio::spawn(async move {
                for _ in 0..ITERATIONS {
                    let mut counter = pool.acquire().await.unwrap();
                    counter.increase();
                    sleep(Duration::from_millis(1)).await;
                }
            })
        })
        .collect::<Vec<_>>();
    for handle in handles {
        handle.await.unwrap();
    }

    let mut sum = 0;
    for _ in 0..MAX_POOL_SIZE {
        let counter = pool.acquire().await.unwrap();
        sum += dbg!(counter.get());
        drop(take(counter));
    }
    assert_eq!(sum as usize, WORKERS * ITERATIONS);
}
