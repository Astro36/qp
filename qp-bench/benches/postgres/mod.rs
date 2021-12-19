pub mod bb8;
pub mod deadpool;
pub mod mobc;
pub mod qp;
pub mod qp_tokio_semaphore;
pub mod r2d2;
pub mod sqlx;

pub const DB_URI: &'static str = "postgresql://postgres:postgres@localhost";
