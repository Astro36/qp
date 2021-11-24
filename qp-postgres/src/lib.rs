use qp::async_trait;
use qp::pool::Pool;
use qp::resource::Factory;
use tokio_postgres::tls::{MakeTlsConnect, TlsConnect};
use tokio_postgres::{Client, Config, Error, Socket};

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

    async fn try_create_resource(&self) -> Result<Self::Output, Self::Error> {
        let (client, conn) = self.config.connect(self.tls.clone()).await?;
        tokio::spawn(conn);
        Ok(client)
    }

    async fn validate(&self, conn: &Self::Output) -> bool {
        conn.is_closed()
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

pub fn connect<T>(config: Config, tls: T, pool_size: usize) -> Pool<PgConnFactory<T>>
where
    T: MakeTlsConnect<Socket> + Clone + Send + Sync,
    T::Stream: Send + Sync + 'static,
    T::TlsConnect: Send + Sync,
    <T::TlsConnect as TlsConnect<Socket>>::Future: Send,
{
    Pool::new(PgConnFactory::new(config, tls), pool_size)
}
