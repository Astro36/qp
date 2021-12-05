use std::error::Error as StdError;
use std::result::Result as StdResult;

pub type BoxDynError = Box<dyn StdError + Send + Sync + 'static>;

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    #[error("attempted to acquire a resource on a closed pool")]
    PoolClosed,

    #[error("error returned from a resource")]
    Resource(#[source] BoxDynError),
}

pub type Result<T> = StdResult<T, Error>;
