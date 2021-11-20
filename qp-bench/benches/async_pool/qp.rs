use qp::async_trait;
use qp::pool::{Pool, Resource};
use std::convert::Infallible;

pub struct Int(i32);

#[async_trait]
impl Resource for Int {
    type Error = Infallible;

    async fn try_new() -> Result<Self, Self::Error> {
        Ok(Self(0))
    }
}

pub async fn run_with(pool_size: usize, workers: usize) {
    let pool: Pool<Int> = Pool::new(pool_size);
    let handles = (0..workers)
        .map(|_| {
            let pool = pool.clone();
            tokio::spawn(async move {
                let mut i = pool.acquire().await.unwrap();
                i.0 += 1;
            })
        })
        .collect::<Vec<_>>();
    for handle in handles {
        handle.await.unwrap();
    }
}
