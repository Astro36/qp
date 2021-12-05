use super::DB_URI;
use criterion::Bencher;
use r2d2::Pool;
use r2d2_postgres::postgres::NoTls;
use r2d2_postgres::PostgresConnectionManager;
use std::thread;
use std::time::Instant;

pub fn bench_with_input(bencher: &mut Bencher, input: &(usize, usize)) {
    bencher.iter_custom(|iters| {
        let config = DB_URI.parse().unwrap();
        let manager = PostgresConnectionManager::new(config, NoTls);
        let pool = Pool::builder()
            .max_size(input.0 as u32)
            .build(manager)
            .unwrap();
        drop((0..input.0).map(|_| pool.get()));
        let start = Instant::now();
        for _ in 0..iters {
            let handles = (0..input.1)
                .map(|_| {
                    let pool = pool.clone();
                    thread::spawn(move || {
                        let mut client = pool.get().unwrap();
                        let row = client.query_one("SELECT 1", &[]).unwrap();
                        let int: i32 = row.get(0);
                        criterion::black_box(int);
                    })
                })
                .collect::<Vec<_>>();
            for handle in handles {
                handle.join().unwrap();
            }
        }
        start.elapsed()
    })
}
