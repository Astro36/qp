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
