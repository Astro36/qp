use axum::extract::{Extension, Query};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{AddExtensionLayer, Router, Server};
use qp_postgres::tokio_postgres::NoTls;
use qp_postgres::PgPool;
use serde::Deserialize;

const DB_URI: &str = "postgresql://postgres:postgres@localhost";
const SERVER_ADDRESS: &str = "0.0.0.0:3000";

type DbPool = PgPool<NoTls>;

#[derive(Debug, Deserialize)]
struct NumberAB {
    a: i32,
    b: i32,
}

async fn add_number(
    Extension(pool): Extension<DbPool>,
    Query(num): Query<NumberAB>,
) -> Result<String, StatusCode> {
    dbg!(&num);
    let client = pool
        .acquire()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let row = client
        .query_one("SELECT $1::INT4 + $2::INT4", &[&num.a, &num.b])
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let sum: i32 = row
        .try_get(0)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    dbg!(&sum);
    Ok(sum.to_string())
}

#[tokio::main]
async fn main() {
    let config = DB_URI.parse().unwrap();
    let pool = qp_postgres::connect(config, NoTls, 8);
    let app = Router::new()
        .route("/", get(add_number))
        .layer(AddExtensionLayer::new(pool));
    let addr = SERVER_ADDRESS.parse().unwrap();
    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
