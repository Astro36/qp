use super::DB_URI;
use criterion::Bencher;
use sqlx::postgres::{PgPoolOptions, PgRow};
use sqlx::Row;
use std::time::Instant;
use tokio::runtime::Runtime;

pub fn bench_with_input(bencher: &mut Bencher, input: &(usize, usize)) {
    bencher
        .to_async(Runtime::new().unwrap())
        .iter_custom(|iters| async move {
            let pool = PgPoolOptions::new()
                .min_connections(input.0 as u32)
                .max_connections(input.0 as u32)
                .connect(DB_URI)
                .await
                .unwrap();
            let start = Instant::now();
            for _ in 0..iters {
                let handles = (0..input.1)
                    .map(|_| {
                        let pool = pool.clone();
                        tokio::spawn(async move {
                            let mut conn = pool.acquire().await.unwrap();
                            let int = sqlx::query("SELECT 1")
                                .try_map(|row: PgRow| row.try_get::<i32, _>(0))
                                .fetch_one(&mut conn)
                                .await
                                .unwrap();
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
