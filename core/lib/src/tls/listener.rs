use std::io;
use std::sync::Arc;

use futures::TryFutureExt;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_rustls::LazyConfigAcceptor;
use rustls::server::{Acceptor, ServerConfig};

use crate::{Ignite, Rocket};
use crate::listener::{Bind, Certificates, Connection, Endpoint, Listener};
use crate::tls::{Error, TlsConfig};
use super::resolver::DynResolver;

#[doc(inline)]
pub use tokio_rustls::server::TlsStream;

/// A TLS listener over some listener interface L.
pub struct TlsListener<L> {
    listener: L,
    config: TlsConfig,
    default: Arc<ServerConfig>,
}

impl<T: Send, L: Bind<T>> Bind<(T, TlsConfig)> for TlsListener<L>
    where L: Listener<Accept = <L as Listener>::Connection>,
{
    type Error = Error;

    async fn bind((inner, config): (T, TlsConfig)) -> Result<Self, Self::Error> {
        Ok(TlsListener {
            default: Arc::new(config.server_config().await?),
            listener: L::bind(inner).map_err(|e| Error::Bind(Box::new(e))).await?,
            config,
        })
    }

    fn bind_endpoint((inner, config): &(T, TlsConfig)) -> Result<Endpoint, Self::Error> {
        L::bind_endpoint(inner)
            .map(|e| e.with_tls(config))
            .map_err(|e| Error::Bind(Box::new(e)))
    }
}

impl<'r, L> Bind<&'r Rocket<Ignite>> for TlsListener<L>
    where L: Bind<&'r Rocket<Ignite>> + Listener<Accept = <L as Listener>::Connection>
{
    type Error = Error;

    async fn bind(rocket: &'r Rocket<Ignite>) -> Result<Self, Self::Error> {
        let mut config: TlsConfig = rocket.figment().extract_inner("tls")?;
        config.resolver = DynResolver::extract(rocket);
        <Self as Bind<_>>::bind((rocket, config)).await
    }

    fn bind_endpoint(rocket: &&'r Rocket<Ignite>) -> Result<Endpoint, Self::Error> {
        let config: TlsConfig = rocket.figment().extract_inner("tls")?;
        <Self as Bind<_>>::bind_endpoint(&(*rocket, config))
    }
}

impl<L> Listener for TlsListener<L>
    where L: Listener<Accept = <L as Listener>::Connection>,
          L::Connection: AsyncRead + AsyncWrite
{
    type Accept = L::Connection;

    type Connection = TlsStream<L::Connection>;

    async fn accept(&self) -> io::Result<Self::Accept> {
        self.listener.accept().await
    }

    async fn connect(&self, conn: L::Connection) -> io::Result<Self::Connection> {
        let acceptor = LazyConfigAcceptor::new(Acceptor::default(), conn);
        let handshake = acceptor.await?;
        let hello = handshake.client_hello();
        let config = match &self.config.resolver {
            Some(r) => r.resolve(hello).await.unwrap_or_else(|| self.default.clone()),
            None => self.default.clone(),
        };

        handshake.into_stream(config).await
    }

    fn endpoint(&self) -> io::Result<Endpoint> {
        Ok(self.listener.endpoint()?.with_tls(&self.config))
    }
}

impl<C: Connection> Connection for TlsStream<C> {
    fn endpoint(&self) -> io::Result<Endpoint> {
        Ok(self.get_ref().0.endpoint()?.assume_tls())
    }

    fn certificates(&self) -> Option<Certificates<'_>> {
        #[cfg(feature = "mtls")] {
            let cert_chain = self.get_ref().1.peer_certificates()?;
            Some(Certificates::from(cert_chain))
        }

        #[cfg(not(feature = "mtls"))]
        None
    }
}
