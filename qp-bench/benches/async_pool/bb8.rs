use qp::async_trait;
use bb8::{ManageConnection, Pool, PooledConnection};
use std::convert::Infallible;

pub struct Int(i32);
pub struct IntManager;

#[async_trait]
impl ManageConnection for IntManager {
    type Connection = Int;
    type Error = Infallible;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        Ok(Int(0))
    }

    async fn is_valid(&self, _: &mut PooledConnection<'_, Self>) -> Result<(), Self::Error> {
        Ok(())
    }

    fn has_broken(&self, _: &mut Self::Connection) -> bool {
        false
    }
}

pub async fn run_with(pool_size: usize, workers: usize) {
    let pool: Pool<IntManager> = Pool::builder().max_size(pool_size as u32).build(IntManager).await.unwrap();
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
