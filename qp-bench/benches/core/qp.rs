use criterion::Bencher;
use qp::async_trait;
use qp::pool::Pool;
use qp::resource::Factory;
use std::convert::Infallible;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use tokio::time::sleep;

pub struct IntFactory;

#[async_trait]
impl Factory for IntFactory {
    type Output = i32;
    type Error = Infallible;

    async fn try_create(&self) -> Result<Self::Output, Self::Error> {
        Ok(0)
    }

    async fn validate(&self, resource: &Self::Output) -> bool {
        resource >= &0
    }
}

pub fn bench_with_input(bencher: &mut Bencher, input: &(usize, usize)) {
    bencher
        .to_async(Runtime::new().unwrap())
        .iter_custom(|iters| async move {
            let pool = Pool::new(IntFactory, input.0);
            let start = Instant::now();
            for _ in 0..iters {
                let handles = (0..input.1)
                    .map(|_| {
                        let pool = pool.clone();
                        tokio::spawn(async move {
                            let int = pool.acquire().await.unwrap();
                            sleep(Duration::from_millis(1)).await;
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
