//! High Performance Async Generic Pool PostgreSQL Adapter
use qp::async_trait;
use qp::resource::Manage;
use qp::Pool;
use tokio_postgres::tls::{MakeTlsConnect, TlsConnect};
use tokio_postgres::{Client, Config, Error, Socket};

pub use qp;
pub use tokio_postgres;

/// An alias for [`Pool`](qp::Pool), managing PostgreSQL connections.
pub type PgPool<T> = Pool<PgConnManager<T>>;

/// A PostgreSQL connection manager.
pub struct PgConnManager<T>
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
impl<T> Manage for PgConnManager<T>
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

impl<T> PgConnManager<T>
where
    T: MakeTlsConnect<Socket> + Clone + Send + Sync,
    T::Stream: Send + Sync + 'static,
    T::TlsConnect: Send + Sync,
    <T::TlsConnect as TlsConnect<Socket>>::Future: Send,
{
    /// Creates a new PostgreSQL connection manager.
    pub fn new(config: Config, tls: T) -> Self {
        Self { config, tls }
    }
}

/// Creates a new PostgreSQL connection pool.
pub fn connect<T>(config: Config, tls: T, pool_size: usize) -> PgPool<T>
where
    T: MakeTlsConnect<Socket> + Clone + Send + Sync,
    T::Stream: Send + Sync + 'static,
    T::TlsConnect: Send + Sync,
    <T::TlsConnect as TlsConnect<Socket>>::Future: Send,
{
    Pool::new(PgConnManager::new(config, tls), pool_size)
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
