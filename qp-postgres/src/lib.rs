use qp::async_trait;
use qp::pool::Pool;
use qp::resource::Factory;
use tokio_postgres::tls::{MakeTlsConnect, TlsConnect};
use tokio_postgres::{Client, Config, Error, Socket};

pub use qp;
pub use tokio_postgres;

pub type PgPool<T> = Pool<PgConnFactory<T>>;

pub struct PgConnFactory<T>
where
    T: MakeTlsConnect<Socket> + Clone + Send + Sync,
    T::Stream: Send + Sync + 'static,
    T::TlsConnect: Send + Sync,
    <T::TlsConnect as TlsConnect<Socket>>::Future: Send,
{
    config: Config,
    tls: T,
}

#[async_trait]
impl<T> Factory for PgConnFactory<T>
where
    T: MakeTlsConnect<Socket> + Clone + Send + Sync,
    T::Stream: Send + Sync + 'static,
    T::TlsConnect: Send + Sync,
    <T::TlsConnect as TlsConnect<Socket>>::Future: Send,
{
    type Output = Client;
    type Error = Error;

    async fn try_create(&self) -> Result<Self::Output, Self::Error> {
        let (client, conn) = self.config.connect(self.tls.clone()).await?;
        tokio::spawn(conn);
        Ok(client)
    }

    async fn validate(&self, client: &Self::Output) -> bool {
        !client.is_closed()
    }
}

impl<T> PgConnFactory<T>
where
    T: MakeTlsConnect<Socket> + Clone + Send + Sync,
    T::Stream: Send + Sync + 'static,
    T::TlsConnect: Send + Sync,
    <T::TlsConnect as TlsConnect<Socket>>::Future: Send,
{
    pub fn new(config: Config, tls: T) -> Self {
        Self { config, tls }
    }
}

pub fn connect<T>(config: Config, tls: T, pool_size: usize) -> PgPool<T>
where
    T: MakeTlsConnect<Socket> + Clone + Send + Sync,
    T::Stream: Send + Sync + 'static,
    T::TlsConnect: Send + Sync,
    <T::TlsConnect as TlsConnect<Socket>>::Future: Send,
{
    Pool::new(PgConnFactory::new(config, tls), pool_size)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_postgres::NoTls;

    #[tokio::test]
    async fn test_connect() {
        let config = "postgresql://postgres:postgres@localhost".parse().unwrap();
        let pool = connect(config, NoTls, 1);
        let client = pool.acquire().await.unwrap();
        let row = client.query_one("SELECT 1", &[]).await.unwrap();
        let value: i32 = row.get(0);
        assert_eq!(value, 1);
    }
}
