use async_trait::async_trait;
use criterion::Bencher;
use deadpool::managed::{Manager, Pool, RecycleResult};
use futures::prelude::*;
use std::convert::Infallible;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use tokio::time::sleep;

pub struct IntManager;

#[async_trait]
impl Manager for IntManager {
    type Type = i32;
    type Error = Infallible;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        Ok(0)
    }

    async fn recycle(&self, _: &mut Self::Type) -> RecycleResult<Self::Error> {
        Ok(())
    }
}

pub fn bench_with_input(bencher: &mut Bencher, input: &(usize, usize)) {
    bencher
        .to_async(Runtime::new().unwrap())
        .iter_custom(|iters| async move {
            let pool: Pool<IntManager> =
                Pool::builder(IntManager).max_size(input.0).build().unwrap();
            drop(future::join_all((0..input.0).map(|_| pool.get())).await);
            let start = Instant::now();
            for _ in 0..iters {
                let handles = (0..input.1)
                    .map(|_| {
                        let pool = pool.clone();
                        tokio::spawn(async move {
                            let int = pool.get().await.unwrap();
                            sleep(Duration::from_millis(0)).await;
                            criterion::black_box(*int);
                        })
                    })
                    .collect::<Vec<_>>();
                for handle in handles {
                    handle.await.unwrap();
                }
            }
            start.elapsed()
        })
}
