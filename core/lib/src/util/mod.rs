mod chain;
mod reader_stream;
mod join;

#[cfg(unix)]
pub mod unix;

pub use chain::Chain;
pub use reader_stream::ReaderStream;
pub use join::join;

#[track_caller]
pub fn spawn_inspect<E, F, Fut>(or: F, future: Fut)
    where F: FnOnce(&E) + Send + Sync + 'static,
          E: Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<(), E>> + Send + 'static,
{
    use futures::TryFutureExt;
    tokio::spawn(future.inspect_err(or));
}

use std::io;
use std::pin::pin;
use std::future::Future;
use futures::future::{select, Either};

pub trait FutureExt: Future + Sized {
    /// Await `self` or `other`, whichever finishes first.
    async fn or<B: Future>(self, other: B) -> Either<Self::Output, B::Output> {
        match futures::future::select(pin!(self), pin!(other)).await {
            Either::Left((v, _)) => Either::Left(v),
            Either::Right((v, _)) => Either::Right(v),
        }
    }

    /// Await `self` unless `trigger` completes. Returns `Ok(Some(T))` if `self`
    /// completes successfully before `trigger`, `Err(E)` if `self` completes
    /// unsuccessfully, and `Ok(None)` if `trigger` completes before `self`.
    async fn unless<T, E, K: Future>(self, trigger: K) -> Result<Option<T>, E>
        where Self: Future<Output = Result<T, E>>
    {
        match select(pin!(self), pin!(trigger)).await {
            Either::Left((v, _)) => Ok(Some(v?)),
            Either::Right((_, _)) => Ok(None),
        }
    }

    /// Await `self` unless `trigger` completes. If `self` completes before
    /// `trigger`, returns the result. Otherwise, always returns an `Err`.
    async fn io_unless<T, K: Future>(self, trigger: K) -> std::io::Result<T>
        where Self: Future<Output = std::io::Result<T>>
    {
        match select(pin!(self), pin!(trigger)).await {
            Either::Left((v, _)) => v,
            Either::Right((_, _)) => Err(io::Error::other("I/O terminated")),
        }
    }
}

impl<F: Future + Sized> FutureExt for F { }
