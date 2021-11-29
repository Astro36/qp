use bb8::{ManageConnection, Pool, PooledConnection};
use criterion::Bencher;
use qp::async_trait;
use std::convert::Infallible;
use std::time::Instant;
use tokio::runtime::Runtime;

pub struct IntManager;

#[async_trait]
impl ManageConnection for IntManager {
    type Connection = i32;
    type Error = Infallible;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        Ok(0)
    }

    async fn is_valid(&self, _: &mut PooledConnection<'_, Self>) -> Result<(), Self::Error> {
        Ok(())
    }

    fn has_broken(&self, _: &mut Self::Connection) -> bool {
        false
    }
}

pub fn bench_with_input(bencher: &mut Bencher, input: &(usize, usize)) {
    bencher
        .to_async(Runtime::new().unwrap())
        .iter_custom(|iters| async move {
            let pool = Pool::builder()
                .max_size(input.0 as u32)
                .build(IntManager)
                .await
                .unwrap();
            let start = Instant::now();
            for _ in 0..iters {
                let handles = (0..input.1)
                    .map(|_| {
                        let pool = pool.clone();
                        tokio::spawn(async move {
                            let int = pool.get().await.unwrap();
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
