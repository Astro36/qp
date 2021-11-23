# Quick Pool

> Rust Async Resource Pool

[![Rust](https://img.shields.io/badge/rust-2021-black.svg?logo=rust&logoColor=white&style=for-the-badge)](https://doc.rust-lang.org/edition-guide/rust-2021/index.html)
[![Tokio](https://img.shields.io/badge/tokio-black.svg?logo=data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCAxMjAgMTA4Ij48ZyBmaWxsPSIjZmZmIiBmaWxsLXJ1bGU9ImV2ZW5vZGQiPjxwYXRoIGQ9Ik0yOS44OTMgNjguNzk0bC0xLjY4Ny45NzQgMiAzLjQ2NCAxLjY4Ny0uOTc0IDEyLjIxNC03LjA1MiAxLjY4Ny0uOTc0LTItMy40NjQtMS42ODcuOTc0ek04OC4xMDcgNzIuMjU4bDEuNjg3Ljk3NCAyLTMuNDY0LTEuNjg3LS45NzQtMTIuMjE0LTcuMDUyLTEuNjg3LS45NzQtMiAzLjQ2NCAxLjY4Ny45NzR6Ii8+PHBhdGggZD0iTTgwIDU0YzAtMTEuMDQ2LTguOTU0LTIwLTIwLTIwcy0yMCA4Ljk1NC0yMCAyMCA4Ljk1NCAyMCAyMCAyMCAyMC04Ljk1NCAyMC0yMHptLTM1Ljc2NyAwYzAtOC43MDggNy4wNTktMTUuNzY3IDE1Ljc2Ny0xNS43NjcgOC43MDggMCAxNS43NjcgNy4wNTkgMTUuNzY3IDE1Ljc2NyAwIDguNzA4LTcuMDU5IDE1Ljc2Ny0xNS43NjcgMTUuNzY3LTguNzA4IDAtMTUuNzY3LTcuMDU5LTE1Ljc2Ny0xNS43Njd6Ii8+PGNpcmNsZSBjeD0iMjQiIGN5PSI3NSIgcj0iMyIvPjxjaXJjbGUgY3g9IjYwIiBjeT0iOTYiIHI9IjMiLz48Y2lyY2xlIGN4PSI2MCIgY3k9IjEyIiByPSIzIi8+PGNpcmNsZSBjeD0iOTYiIGN5PSIzMyIgcj0iMyIvPjxjaXJjbGUgY3g9IjI0IiBjeT0iMzMiIHI9IjMiLz48Y2lyY2xlIGN4PSI5NiIgY3k9Ijc1IiByPSIzIi8+PGNpcmNsZSBjeD0iNjAiIGN5PSI1NCIgcj0iMyIvPjxwYXRoIGQ9Ik0yIDUySDB2NGg0MnYtNGgtMnpNODAgNTJoLTJ2NGg0MnYtNGgtMnpNNzIuODMgNzAuMzg5bC0xLjAwNy0xLjcyOS0zLjQ1NiAyLjAxNCAxLjAwNyAxLjcyOCAxOS4xMjggMzIuODM1IDEuMDA3IDEuNzI4IDMuNDU2LTIuMDE0LTEuMDA3LTEuNzI4ek0zMi41ODEgMi4xNEwzMS41NzUuNDEybC0zLjQ1NyAyLjAxMyAxLjAwNyAxLjcyOSAxOS4xMjggMzIuODM0IDEuMDA3IDEuNzI4IDMuNDU2LTIuMDEzLTEuMDA2LTEuNzI4ek05MC43MSA0LjE1NGwxLjAwNi0xLjcyOUw4OC4yNi40MTIgODcuMjUzIDIuMTQgNjguMTI1IDM0Ljk3NWwtMS4wMDcgMS43MjggMy40NTcgMi4wMTMgMS4wMDYtMS43Mjh6TTUwLjgyNyA3Mi41OTdsMS4wMDYtMS43MjgtMy40NTYtMi4wMTQtMS4wMDcgMS43MjgtMTkuMTI4IDMyLjgzNS0xLjAwNyAxLjcyOCAzLjQ1NyAyLjAxNCAxLjAwNi0xLjcyOHpNNTggODcuMDE3Vjg5aDRWNzFoLTR2MS45ODN6TTU4IDM1LjA1MlYzN2g0VjE5aC00djEuOTQ4ek00Mi4xMDcgNDYuMjU4bDEuNjg3Ljk3NCAyLTMuNDY0LTEuNjg3LS45NzQtMTIuMjE0LTcuMDUyLTEuNjg3LS45NzQtMiAzLjQ2NCAxLjY4Ny45NzR6TTc1Ljg5MyA0Mi43OTRsLTEuNjg3Ljk3NCAyIDMuNDY0IDEuNjg3LS45NzQgMTIuMjE0LTcuMDUyIDEuNjg3LS45NzQtMi0zLjQ2NC0xLjY4Ny45NzR6Ii8+PC9nPjwvc3ZnPg==&style=for-the-badge)](https://tokio.rs/)
[![GitHub Workflow](https://img.shields.io/github/workflow/status/Astro36/qp/Rust?logo=github&logoColor=white&style=for-the-badge)](https://github.com/Astro36/qp/actions)
[![License](https://img.shields.io/github/license/Astro36/qp?style=for-the-badge)](./LICENSE) 

## Usage

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

    async fn try_create_resource(&self) -> Result<Self::Output, Self::Error> {
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
