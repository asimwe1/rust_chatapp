use futures::TryFutureExt;

use crate::listener::Listener;

pub trait Bindable: Sized {
    type Listener: Listener + 'static;

    type Error: std::error::Error + Send + 'static;

    async fn bind(self) -> Result<Self::Listener, Self::Error>;
}

impl<L: Listener + 'static> Bindable for L {
    type Listener = L;

    type Error = std::convert::Infallible;

    async fn bind(self) -> Result<Self::Listener, Self::Error> {
        Ok(self)
    }
}

impl<A: Bindable, B: Bindable> Bindable for either::Either<A, B> {
    type Listener = tokio_util::either::Either<A::Listener, B::Listener>;

    type Error = either::Either<A::Error, B::Error>;

    async fn bind(self) -> Result<Self::Listener, Self::Error> {
        match self {
            either::Either::Left(a) => a.bind()
                .map_ok(tokio_util::either::Either::Left)
                .map_err(either::Either::Left)
                .await,
            either::Either::Right(b) => b.bind()
                .map_ok(tokio_util::either::Either::Right)
                .map_err(either::Either::Right)
                .await,
        }
    }
}
