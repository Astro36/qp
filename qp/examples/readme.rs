use async_trait::async_trait;
use qp::pool::{self, Pool, Resource};
use std::convert::Infallible;

pub struct Int(i32);

#[async_trait]
impl Resource for Int {
    type Error = Infallible;

    async fn try_new() -> Result<Self, Self::Error> {
        Ok(Self(0))
    }

    async fn is_valid(&self) -> bool {
        self.0 >= 0
    }
}

#[tokio::main]
async fn main() {
    let pool: Pool<Int> = Pool::new(1); // max_size=1

    // create a resource when the pool is empty or all resources are occupied.
    let mut int = pool.acquire().await.unwrap();
    int.0 = 1;
    dbg!(int.0); // 1

    // release the resource and put it back to the pool.
    drop(int);

    let mut int = pool.acquire().await.unwrap();
    dbg!(int.0); // 1
    int.0 = 100;
    drop(int);

    let mut int = pool.acquire().await.unwrap();
    dbg!(int.0); // 100
    int.0 = -1; // the resource will be disposed because `is_valid` is false.
    drop(int);

    let int = pool.acquire().await.unwrap();
    dbg!(int.0); // 0; old resource is disposed and create new one.

    // take the resource from the pool.
    let raw_int: Int = pool::take_resource(int); // raw resource
    dbg!(raw_int.0); // 0
    drop(raw_int);

    let _int = pool.acquire().await.unwrap();
    // `_int` will be auto released by `ResourceGuard` destructor.
}
