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
use either::Either;
use futures::future;

pub trait FutureExt: Future + Sized {
    /// Await `self` or `other`, whichever finishes first.
    async fn race<B: Future>(self, other: B) -> Either<Self::Output, B::Output> {
        match future::select(pin!(self), pin!(other)).await {
            future::Either::Left((v, _)) => Either::Left(v),
            future::Either::Right((v, _)) => Either::Right(v),
        }
    }

    async fn race_io<T, K: Future>(self, trigger: K) -> io::Result<T>
        where Self: Future<Output = io::Result<T>>
    {
        match future::select(pin!(self), pin!(trigger)).await {
            future::Either::Left((v, _)) => v,
            future::Either::Right((_, _)) => Err(io::Error::other("i/o terminated")),
        }
    }
}

impl<F: Future + Sized> FutureExt for F { }

#[doc(hidden)]
#[macro_export]
macro_rules! for_both {
    ($value:expr, $pattern:pat => $result:expr) => {
        match $value {
            tokio_util::either::Either::Left($pattern) => $result,
            tokio_util::either::Either::Right($pattern) => $result,
        }
    };
}

pub use for_both;
