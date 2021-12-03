use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Error, Response, Server};
use qp_postgres::tokio_postgres::NoTls;

const DB_URI: &str = "postgresql://postgres:postgres@localhost";
const SERVER_ADDRESS: &str = "0.0.0.0:3000";

#[tokio::main]
async fn main() {
    let config = DB_URI.parse().unwrap();
    let pool = qp_postgres::connect(config, NoTls, 8);
    let addr = SERVER_ADDRESS.parse().unwrap();
    Server::bind(&addr)
        .serve(make_service_fn(move |_| {
            let pool = pool.clone();
            async move {
                Ok::<_, Error>(service_fn(move |_req| {
                    let pool = pool.clone();
                    async move {
                        let _client = pool.acquire().await.unwrap();
                        Ok::<_, Error>(Response::new(Body::from("ok")))
                    }
                }))
            }
        }))
        .await
        .unwrap();
}
