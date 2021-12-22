use async_trait::async_trait;
use qp::resource::Manage;
use qp::{Pool, Pooled};

pub struct IntManager;

#[async_trait]
impl Manage for IntManager {
    type Output = i32;
    type Error = ();

    async fn try_create(&self) -> Result<Self::Output, Self::Error> {
        Ok(0)
    }

    async fn validate(&self, resource: &Self::Output) -> bool {
        resource >= &0
    }
}

#[tokio::main]
async fn main() {
    let pool = Pool::new(IntManager, 1); // max_size=1

    // create a resource when the pool is empty or all resources are occupied.
    let mut int = pool.acquire().await.unwrap();
    *int = 1;
    dbg!(*int); // 1
    dbg!(Pooled::is_valid(&int).await); // true; validate the resource.

    // release the resource and put it back to the pool.
    drop(int);

    let mut int = pool.acquire().await.unwrap();
    dbg!(*int); // 1
    *int = 100;
    drop(int);

    let mut int = pool.acquire().await.unwrap();
    dbg!(*int); // 100
    *int = -1; // the resource will be disposed because `validate` is false.
    dbg!(Pooled::is_valid(&int).await); // false
    drop(int);

    let int = pool.acquire_unchecked().await.unwrap();
    dbg!(*int); // -1; no validation before acquiring.
    drop(int);

    let int = pool.acquire().await.unwrap();
    dbg!(*int); // 0; old resource is disposed and create new one.

    // take the resource from the pool.
    let raw_int: i32 = Pooled::take(int); // raw resource
    dbg!(raw_int); // 0
    drop(raw_int);

    let _int = pool.acquire().await.unwrap();
    // `_int` will be auto released by `Pooled` destructor.
}
