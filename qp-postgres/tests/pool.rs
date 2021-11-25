use tokio_postgres::NoTls;

#[tokio::test]
async fn main() {
    let config = "host=localhost user=postgres password=postgres".parse().unwrap();
    let pool = qp_postgres::connect(config, NoTls, 4);
    let conn = pool.acquire().await.unwrap();
    let rows = conn.query("SELECT $1::TEXT", &[&"hello world"]).await.unwrap();
    let value: &str = rows[0].get(0);
    assert_eq!(value, "hello world");
}
