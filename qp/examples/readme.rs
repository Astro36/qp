use async_trait::async_trait;
use qp::pool::{self, Pool};
use qp::resource::Factory;
use std::convert::Infallible;

pub struct IntFactory;

#[async_trait]
impl Factory for IntFactory {
    type Output = i32;
    type Error = Infallible;

    async fn try_create(&self) -> Result<Self::Output, Self::Error> {
        Ok(0)
    }

    async fn validate(&self, resource: &Self::Output) -> bool {
        resource >= &0
    }
}

#[tokio::main]
async fn main() {
    let pool = Pool::new(IntFactory, 1); // max_size=1

    // create a resource when the pool is empty or all resources are occupied.
    let mut int = pool.acquire().await.unwrap();
    *int = 1;
    dbg!(*int); // 1
    dbg!(int.is_valid().await); // true; validate the resource.

    // release the resource and put it back to the pool.
    drop(int);

    let mut int = pool.acquire().await.unwrap();
    dbg!(*int); // 1
    *int = 100;
    drop(int);

    let mut int = pool.acquire().await.unwrap();
    dbg!(*int); // 100
    *int = -1; // the resource will be disposed because `validate` is false.
    dbg!(int.is_valid().await); // false
    drop(int);

    let int = pool.acquire_unchecked().await.unwrap();
    dbg!(*int); // -1; no validation before acquiring.
    drop(int);

    let int = pool.acquire().await.unwrap();
    dbg!(*int); // 0; old resource is disposed and create new one.

    // take the resource from the pool.
    let raw_int: i32 = pool::take_resource(int); // raw resource
    dbg!(raw_int); // 0
    drop(raw_int);

    let _int = pool.acquire().await.unwrap();
    // `_int` will be auto released by `Pooled` destructor.
}
