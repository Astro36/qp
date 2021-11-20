use qp::async_trait;
use qp::pool::{Pool, Resource};
use std::convert::Infallible;
use std::time::Duration;

struct Int(i32);

#[async_trait]
impl Resource for Int {
    type Error = Infallible;

    async fn try_new() -> Result<Self, Self::Error> {
        Ok(Self(0))
    }
}

pub async fn run(pool_size: usize, workers: usize) {
    let pool: Pool<Int> = Pool::new(pool_size, Duration::from_millis(100));
    let handles = (0..workers)
        .map(|_| {
            let pool = pool.clone();
            tokio::spawn(async move {
                let i = pool.acquire().await.unwrap();
                criterion::black_box(i);
            })
        })
        .collect::<Vec<_>>();
    for handle in handles {
        handle.await.unwrap();
    }
}
