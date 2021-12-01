# Quick Pool

> Rust Async Resource Pool

[![Crates.io](https://img.shields.io/crates/v/qp?style=for-the-badge)](https://crates.io/crates/qp)
[![Docs.rs](https://img.shields.io/docsrs/qp?style=for-the-badge)](https://docs.rs/qp)
[![Rust](https://img.shields.io/badge/rust-2021-black.svg?style=for-the-badge)](https://doc.rust-lang.org/edition-guide/rust-2021/index.html)
[![Rust](https://img.shields.io/badge/rustc->=1.56-black.svg?style=for-the-badge)](https://blog.rust-lang.org/2021/10/21/Rust-1.56.0.html)
[![GitHub Workflow](https://img.shields.io/github/workflow/status/Astro36/qp/Quick%20Pool?style=for-the-badge)](https://github.com/Astro36/qp/actions/workflows/qp.yml)
[![Crates.io](https://img.shields.io/crates/d/qp?style=for-the-badge)](https://crates.io/crates/qp)
[![License](https://img.shields.io/crates/l/qp?style=for-the-badge)](./LICENSE) 

## Usage

### DBCP

| Database     | Backend          | Adapter       | Version                |
| ------------ | ---------------- | ------------- | ---------------------- |
| [PostgreSQL] | [tokio-postgres] | [qp-postgres] | ![qp-postgres-version] |

[PostgreSQL]: https://www.postgresql.org/
[tokio-postgres]: https://crates.io/crates/tokio-postgres
[qp-postgres]: https://crates.io/crates/qp-postgres
[qp-postgres-version]: https://img.shields.io/crates/v/qp-postgres?style=for-the-badge

### Example

```rust
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

    // release the resource and put it back to the pool.
    drop(int);

    let mut int = pool.acquire().await.unwrap();
    dbg!(*int); // 1
    *int = 100;
    drop(int);

    let mut int = pool.acquire().await.unwrap();
    dbg!(*int); // 100
    *int = -1; // the resource will be disposed because `validate` is false.
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
```

## Alternatives

### Performance Comparison

| [bb8]        | [deadpool]        |
| ------------ | ----------------- |
| ![bb8-bench] | ![deadpool-bench] |

| [mobc]        | [qp]        |
| ------------- | ----------- |
| ![mobc-bench] | ![qp-bench] |

[bb8]: https://crates.io/crates/bb8
[deadpool]: https://crates.io/crates/deadpool
[mobc]: https://crates.io/crates/mobc
[qp]: https://crates.io/crates/qp
[bb8-bench]: https://astro36.github.io/qp/core/bb8/pool=16%20worker=64/report/pdf.svg
[deadpool-bench]: https://astro36.github.io/qp/core/deadpool/pool=16%20worker=64/report/pdf.svg
[mobc-bench]: https://astro36.github.io/qp/core/mobc/pool=16%20worker=64/report/pdf.svg
[qp-bench]: https://astro36.github.io/qp/core/qp/pool=16%20worker=64/report/pdf.svg

For more information, see [Quick Pool Benchmark](./qp-bench/README.md).

## License

```text
Copyright (c) 2021 Seungjae Park

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

Quick Pool is licensed under the [MIT License](./LICENSE).
