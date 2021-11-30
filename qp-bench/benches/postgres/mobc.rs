use super::DB_URI;
use criterion::Bencher;
use mobc::Pool;
use mobc_postgres::tokio_postgres::NoTls;
use mobc_postgres::PgConnectionManager;
use std::time::Instant;
use tokio::runtime::Runtime;

pub fn bench_with_input(bencher: &mut Bencher, input: &(usize, usize)) {
    bencher
        .to_async(Runtime::new().unwrap())
        .iter_custom(|iters| async move {
            let config = DB_URI.parse().unwrap();
            let manager = PgConnectionManager::new(config, NoTls);
            let pool = Pool::builder().max_open(input.0 as u64).build(manager);
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
