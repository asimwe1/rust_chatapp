use crate::listener::{Endpoint, Listener};

pub trait Bind<T>: Listener + 'static {
    type Error: std::error::Error + Send + 'static;

    #[crate::async_bound(Send)]
    async fn bind(to: T) -> Result<Self, Self::Error>;

    fn bind_endpoint(to: &T) -> Result<Endpoint, Self::Error>;
}
