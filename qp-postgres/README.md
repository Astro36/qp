# Quick Pool for PostgreSQL

> Rust Async Resource Pool PostgreSQL Adapter

[![Crates.io](https://img.shields.io/crates/v/qp-postgres?style=for-the-badge)](https://crates.io/crates/qp-postgres)
[![Docs.rs](https://img.shields.io/docsrs/qp-postgres?style=for-the-badge)](https://docs.rs/qp-postgres)
[![Rust](https://img.shields.io/badge/rust-2021-black.svg?style=for-the-badge)](https://doc.rust-lang.org/edition-guide/rust-2021/index.html)
[![Rust](https://img.shields.io/badge/rustc-1.56+-black.svg?style=for-the-badge)](https://blog.rust-lang.org/2021/10/21/Rust-1.56.0.html)
[![GitHub Workflow](https://img.shields.io/github/workflow/status/Astro36/qp/Quick%20Pool%20for%20PostgreSQL?style=for-the-badge)](https://github.com/Astro36/qp/actions/workflows/qp-postgres.yml)
[![Crates.io](https://img.shields.io/crates/d/qp-postgres?style=for-the-badge)](https://crates.io/crates/qp-postgres)
[![License](https://img.shields.io/crates/l/qp-postgres?style=for-the-badge)](./LICENSE) 

## Usage

### Example

```rust
use tokio_postgres::NoTls;

#[tokio::main]
async fn main() {
    let config = "postgresql://postgres:postgres@localhost".parse().unwrap();
    let pool = qp_postgres::connect(config, NoTls, 8);
    let client = pool.acquire().await.unwrap();
    let row = client.query_one("SELECT 1", &[]).await.unwrap();
    let int: i32 = row.get(0);
    dbg!(&int);
}
```

## Alternatives

| Backend          | Adapter             | Version                      |
| ---------------- | ------------------- | ---------------------------- |
| [tokio-postgres] | [bb8-postgres]      | ![bb8-postgres-version]      |
| [tokio-postgres] | [deadpool-postgres] | ![deadpool-postgres-version] |
| [tokio-postgres] | [mobc-postgres]     | ![mobc-postgres-version]     |
| [postgres]       | [r2d2-postgres]     | ![r2d2-postgres-version]     |
| [sqlx]           | -                   | ![sqlx-version]              |

### Performance Comparison

<table>
<tr>
<th colspan="2"><img src="https://astro36.github.io/qp/postgres/pool=16%20worker=64/report/violin.svg" alt="total"></th>
</tr>
<tr>
<td><code>bb8-postgres</code> has timeout issue</td>
<td><img src="https://astro36.github.io/qp/postgres/deadpool/pool=16%20worker=64/report/pdf.svg" alt="deadpool-postgres"></td>
</tr>
<tr>
<td><img src="https://astro36.github.io/qp/postgres/mobc/pool=16%20worker=64/report/pdf.svg" alt="mobc-postgres"></td>
<td><img src="https://astro36.github.io/qp/postgres/qp/pool=16%20worker=64/report/pdf.svg" alt="qp-postgres"></td>
</tr>
<tr>
<td><img src="https://astro36.github.io/qp/postgres/r2d2/pool=16%20worker=64/report/pdf.svg" alt="r2d2-postgres"></td>
<td><img src="https://astro36.github.io/qp/postgres/sqlx/pool=16%20worker=64/report/pdf.svg" alt="sqlx"></td>
</tr>
</table>

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

*Quick Pool for PostgreSQL* is licensed under the [MIT License](./LICENSE).

[tokio-postgres]: https://crates.io/crates/tokio-postgres
[postgres]: https://crates.io/crates/postgres
[sqlx]: https://crates.io/crates/sqlx

[bb8-postgres]: https://crates.io/crates/bb8-postgres
[deadpool-postgres]: https://crates.io/crates/deadpool-postgres
[mobc-postgres]: https://crates.io/crates/mobc-postgres
[qp-postgres]: https://crates.io/crates/qp-postgres
[r2d2-postgres]: https://crates.io/crates/r2d2-postgres

[bb8-postgres-version]: https://img.shields.io/crates/v/bb8-postgres?style=for-the-badge
[deadpool-postgres-version]: https://img.shields.io/crates/v/deadpool-postgres?style=for-the-badge
[mobc-postgres-version]: https://img.shields.io/crates/v/mobc-postgres?style=for-the-badge
[qp-postgres-version]: https://img.shields.io/crates/v/qp-postgres?style=for-the-badge
[r2d2-postgres-version]: https://img.shields.io/crates/v/r2d2-postgres?style=for-the-badge
[sqlx-version]: https://img.shields.io/crates/v/sqlx?style=for-the-badge
