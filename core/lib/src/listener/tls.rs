use std::io;
use std::sync::Arc;

use serde::Deserialize;
use rustls::server::{ServerSessionMemoryCache, ServerConfig, WebPkiClientVerifier};
use tokio_rustls::TlsAcceptor;

use crate::tls::{TlsConfig, Error};
use crate::tls::util::{load_cert_chain, load_key, load_ca_certs};
use crate::listener::{Listener, Bindable, Connection, Certificates, Endpoint};

#[doc(inline)]
pub use tokio_rustls::server::TlsStream;

/// A TLS listener over some listener interface L.
pub struct TlsListener<L> {
    listener: L,
    acceptor: TlsAcceptor,
    config: TlsConfig,
}

#[derive(Clone, Deserialize)]
pub struct TlsBindable<I> {
    #[serde(flatten)]
    pub inner: I,
    pub tls: TlsConfig,
}

impl TlsConfig {
    pub(crate) fn acceptor(&self) -> Result<tokio_rustls::TlsAcceptor, Error> {
        let provider = rustls::crypto::CryptoProvider {
            cipher_suites: self.ciphers().map(|c| c.into()).collect(),
            ..rustls::crypto::ring::default_provider()
        };

        #[cfg(feature = "mtls")]
        let verifier = match self.mutual {
            Some(ref mtls) => {
                let ca_certs = load_ca_certs(&mut mtls.ca_certs_reader()?)?;
                let verifier = WebPkiClientVerifier::builder(Arc::new(ca_certs));
                match mtls.mandatory {
                    true => verifier.build()?,
                    false => verifier.allow_unauthenticated().build()?,
                }
            },
            None => WebPkiClientVerifier::no_client_auth(),
        };

        #[cfg(not(feature = "mtls"))]
        let verifier = WebPkiClientVerifier::no_client_auth();

        let key = load_key(&mut self.key_reader()?)?;
        let cert_chain = load_cert_chain(&mut self.certs_reader()?)?;
        let mut tls_config = ServerConfig::builder_with_provider(Arc::new(provider))
            .with_safe_default_protocol_versions()?
            .with_client_cert_verifier(verifier)
            .with_single_cert(cert_chain, key)?;

        tls_config.ignore_client_order = self.prefer_server_cipher_order;
        tls_config.session_storage = ServerSessionMemoryCache::new(1024);
        tls_config.ticketer = rustls::crypto::ring::Ticketer::new()?;
        tls_config.alpn_protocols = vec![b"http/1.1".to_vec()];
        if cfg!(feature = "http2") {
            tls_config.alpn_protocols.insert(0, b"h2".to_vec());
        }

        Ok(TlsAcceptor::from(Arc::new(tls_config)))
    }
}

impl<I: Bindable> Bindable for TlsBindable<I> {
    type Listener = TlsListener<I::Listener>;

    type Error = Error;

    async fn bind(self) -> Result<Self::Listener, Self::Error> {
        Ok(TlsListener {
            acceptor: self.tls.acceptor()?,
            listener: self.inner.bind().await.map_err(|e| Error::Bind(Box::new(e)))?,
            config: self.tls,
        })
    }
}

impl<L: Listener + Sync> Listener for TlsListener<L>
    where L::Connection: Unpin
{
    type Accept = L::Accept;

    type Connection = TlsStream<L::Connection>;

    async fn accept(&self) -> io::Result<Self::Accept> {
        self.listener.accept().await
    }

    async fn connect(&self, accept: L::Accept) -> io::Result<Self::Connection> {
        let conn = self.listener.connect(accept).await?;
        self.acceptor.accept(conn).await
    }

    fn socket_addr(&self) -> io::Result<Endpoint> {
        Ok(self.listener.socket_addr()?.with_tls(self.config.clone()))
    }
}

impl<C: Connection + Unpin> Connection for TlsStream<C> {
    fn peer_address(&self) -> io::Result<Endpoint> {
        Ok(self.get_ref().0.peer_address()?.assume_tls())
    }

    #[cfg(feature = "mtls")]
    fn peer_certificates(&self) -> Option<Certificates<'_>> {
        let cert_chain = self.get_ref().1.peer_certificates()?;
        Some(Certificates::from(cert_chain))
    }
}
