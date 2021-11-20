use deadpool::async_trait;
use deadpool::managed::{Manager, Pool, RecycleResult};
use std::convert::Infallible;

pub struct Int(i32);
pub struct IntManager;

#[async_trait]
impl Manager for IntManager {
    type Type = Int;
    type Error = Infallible;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        Ok(Int(0))
    }

    async fn recycle(&self, _: &mut Self::Type) -> RecycleResult<Self::Error> {
        Ok(())
    }
}

pub async fn run_with(pool_size: usize, workers: usize) {
    let pool: Pool<IntManager> = Pool::builder(IntManager)
        .max_size(pool_size)
        .build()
        .unwrap();
    let handles = (0..workers)
        .map(|_| {
            let pool = pool.clone();
            tokio::spawn(async move {
                let mut i = pool.get().await.unwrap();
                i.0 += 1;
            })
        })
        .collect::<Vec<_>>();
    for handle in handles {
        handle.await.unwrap();
    }
}
