use mobc::{async_trait, Manager, Pool};
use std::convert::Infallible;

pub struct Int(i32);
pub struct IntManager;

#[async_trait]
impl Manager for IntManager {
    type Connection = Int;
    type Error = Infallible;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        Ok(Int(0))
    }

    async fn check(&self, conn: Self::Connection) -> Result<Self::Connection, Self::Error> {
        Ok(conn)
    }
}

pub async fn run_with(pool_size: usize, workers: usize) {
    let pool: Pool<IntManager> = Pool::builder().max_open(pool_size as u64).build(IntManager);
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
