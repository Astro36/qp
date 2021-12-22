pub mod bb8;
pub mod deadpool;
pub mod mobc;
pub mod qp;
pub mod r2d2;

pub fn factorial(n: i64) -> i64 {
    (1..=n).product()
}

pub fn loop_factorial20() {
    for _ in 0..1_000 {
        criterion::black_box(factorial(20));
    }
}
