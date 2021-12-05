use super::DB_URI;
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use criterion::Bencher;
use futures::prelude::*;
use std::time::Instant;
use tokio::runtime::Runtime;
use tokio_postgres::NoTls;

pub fn bench_with_input(bencher: &mut Bencher, input: &(usize, usize)) {
    bencher
        .to_async(Runtime::new().unwrap())
        .iter_custom(|iters| async move {
            let manager = PostgresConnectionManager::new_from_stringlike(DB_URI, NoTls).unwrap();
            let pool = Pool::builder()
                .max_size(input.0 as u32)
                .build(manager)
                .await
                .unwrap();
            drop(future::join_all((0..input.0).map(|_| pool.get())).await);
            let start = Instant::now();
            for _ in 0..iters {
                let handles = (0..input.1)
                    .map(|_| {
                        let pool = pool.clone();
                        tokio::spawn(async move {
                            let client = pool.get().await.unwrap();
                            let row = client.query_one("SELECT 1", &[]).await.unwrap();
                            let int: i32 = row.get(0);
                            criterion::black_box(int);
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
