use super::loop_factorial20;
use criterion::Bencher;
use r2d2::{ManageConnection, Pool};
use std::convert::Infallible;
use std::thread;
use std::time::Instant;

pub struct IntManager;

impl ManageConnection for IntManager {
    type Connection = i32;
    type Error = Infallible;

    fn connect(&self) -> Result<Self::Connection, Self::Error> {
        Ok(0)
    }

    fn is_valid(&self, _: &mut Self::Connection) -> Result<(), Self::Error> {
        Ok(())
    }

    fn has_broken(&self, _: &mut Self::Connection) -> bool {
        false
    }
}

pub fn bench_with_input(bencher: &mut Bencher, input: &(usize, usize)) {
    bencher.iter_custom(|iters| {
        let pool = Pool::builder()
            .max_size(input.0 as u32)
            .build(IntManager)
            .unwrap();
        drop((0..input.0).map(|_| pool.get()));
        let start = Instant::now();
        for _ in 0..iters {
            let handles = (0..input.1)
                .map(|_| {
                    let pool = pool.clone();
                    thread::spawn(move || {
                        let int = pool.get().unwrap();
                        loop_factorial20();
                        criterion::black_box(*int);
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
