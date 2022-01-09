use async_trait::async_trait;
use qp::resource::Manage;
use qp::{Pool, Pooled};

const MAX_POOL_SIZE: usize = 4;
const WORKERS: usize = 8;
const ITERATIONS: usize = 16;

struct Counter(i32);

impl Counter {
    fn new() -> Self {
        Self(0)
    }

    fn get(&self) -> i32 {
        self.0
    }

    fn increase(&mut self) {
        self.0 += 1;
    }
}

struct CounterManager;

#[async_trait]
impl Manage for CounterManager {
    type Output = Counter;
    type Error = ();

    async fn try_create(&self) -> Result<Self::Output, Self::Error> {
        Ok(Self::Output::new())
    }
}

#[tokio::test]
async fn main() {
    let pool = Pool::new(CounterManager, MAX_POOL_SIZE);
    pool.reserve(MAX_POOL_SIZE).await.unwrap();

    let handles = (0..WORKERS)
        .map(|_| {
            let pool = pool.clone();
            tokio::spawn(async move {
                for _ in 0..ITERATIONS {
                    let mut counter = pool.acquire().await.unwrap();
                    counter.increase();
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
        drop(Pooled::take(counter));
    }
    assert_eq!(sum as usize, WORKERS * ITERATIONS);
}
