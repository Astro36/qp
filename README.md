# Quick Pool

> High Performance Async Generic Pool

[![Crates.io](https://img.shields.io/crates/v/qp?style=for-the-badge)](https://crates.io/crates/qp)
[![Docs.rs](https://img.shields.io/docsrs/qp?style=for-the-badge)](https://docs.rs/qp)
[![Rust](https://img.shields.io/badge/rust-2021-black.svg?style=for-the-badge)](https://doc.rust-lang.org/edition-guide/rust-2021/index.html)
[![Rust](https://img.shields.io/badge/rustc-1.56+-black.svg?style=for-the-badge)](https://blog.rust-lang.org/2021/10/21/Rust-1.56.0.html)
[![GitHub Workflow](https://img.shields.io/github/workflow/status/Astro36/qp/CI?style=for-the-badge)](https://github.com/Astro36/qp/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/d/qp?style=for-the-badge)](https://crates.io/crates/qp)
[![License](https://img.shields.io/crates/l/qp?style=for-the-badge)](./LICENSE) 

## Usage

### DBCP

| Database     | Backend          | Adapter       | Version                |
| ------------ | ---------------- | ------------- | ---------------------- |
| [PostgreSQL] | [tokio-postgres] | [qp-postgres] | ![qp-postgres-version] |

### Example

```rust
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

    dbg!(pool.max_size()); // 1
    dbg!(pool.size()); // 1

    // create a resource when the pool is empty or all resources are occupied.
    let mut int = pool.acquire().await.unwrap();
    *int = 1;
    dbg!(*int); // 1
    dbg!(Pooled::is_valid(&int).await); // true; validate the resource.

    dbg!(pool.size()); // 0

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
```

## Alternatives

| Crate      | Version             |
| ---------- | ------------------- |
| [bb8]      | ![bb8-version]      |
| [deadpool] | ![deadpool-version] |
| [mobc]     | ![mobc-version]     |
| [r2d2]     | ![r2d2-version]     |

### bb8 vs qp

`bb8` implements a resource waiter queue using `futures-channel` and uses the `parking_lot` for mutex.

On the other hand, `qp` uses a **lock-free** waiter queue using `crossbeam-queue`.
`qp` doesn't use mutex.

### deadpool vs qp

`deadpool` implements a idle resource queue using [`VecDeque`](https://doc.rust-lang.org/std/collections/struct.VecDeque.html) and [`Mutex`](https://doc.rust-lang.org/std/sync/struct.Mutex.html) and controls access to resources using [`tokio::sync::Semaphore`](https://docs.rs/tokio/latest/tokio/sync/struct.Semaphore.html).

On the other hand, `qp` uses a semaphore implemented using a lock-free queue.
Also, `qp` is a **lock-free** data structure that never uses lock in idle resource queue.

### Performance Comparison

> Resource Acquisition Time Benchmark

![Benchmark](https://raw.githubusercontent.com/Astro36/rust-pool-benchmark/main/results/benchmark(p08_w064).svg)

![Benchmark](https://raw.githubusercontent.com/Astro36/rust-pool-benchmark/main/results/benchmark(p16_w064).svg)

For more information, see [Rust Pool Benchmark](/../../../rust-pool-benchmark/blob/main/results/README.md).

## License

```text
Copyright (c) 2022 Seungjae Park

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

*Quick Pool* is licensed under the [MIT License](/qp/LICENSE).

[PostgreSQL]: https://www.postgresql.org/
[tokio-postgres]: https://crates.io/crates/tokio-postgres
[qp-postgres]: https://crates.io/crates/qp-postgres
[qp-postgres-version]: https://img.shields.io/crates/v/qp-postgres?style=for-the-badge

[bb8]: https://crates.io/crates/bb8
[deadpool]: https://crates.io/crates/deadpool
[mobc]: https://crates.io/crates/mobc
[qp]: https://crates.io/crates/qp
[r2d2]: https://crates.io/crates/r2d2

[bb8-version]: https://img.shields.io/crates/v/bb8?style=for-the-badge
[deadpool-version]: https://img.shields.io/crates/v/deadpool?style=for-the-badge
[mobc-version]: https://img.shields.io/crates/v/mobc?style=for-the-badge
[qp-version]: https://img.shields.io/crates/v/qp?style=for-the-badge
[r2d2-version]: https://img.shields.io/crates/v/r2d2?style=for-the-badge
