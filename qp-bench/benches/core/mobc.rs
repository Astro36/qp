use criterion::Bencher;
use mobc::{async_trait, Manager, Pool};
use std::convert::Infallible;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use tokio::time::sleep;

pub struct IntManager;

#[async_trait]
impl Manager for IntManager {
    type Connection = i32;
    type Error = Infallible;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        Ok(0)
    }

    async fn check(&self, conn: Self::Connection) -> Result<Self::Connection, Self::Error> {
        Ok(conn)
    }
}

pub fn bench_with_input(bencher: &mut Bencher, input: &(usize, usize)) {
    bencher
        .to_async(Runtime::new().unwrap())
        .iter_custom(|iters| async move {
            let pool = Pool::builder().max_open(input.0 as u64).build(IntManager);
            let start = Instant::now();
            for _ in 0..iters {
                let handles = (0..input.1)
                    .map(|_| {
                        let pool = pool.clone();
                        tokio::spawn(async move {
                            let int = pool.get().await.unwrap();
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
