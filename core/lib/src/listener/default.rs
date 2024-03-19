use either::Either;

use crate::listener::{Bindable, Endpoint};
use crate::error::{Error, ErrorKind};

#[derive(serde::Deserialize)]
pub struct DefaultListener {
    #[serde(default)]
    pub address: Endpoint,
    pub port: Option<u16>,
    pub reuse: Option<bool>,
    #[cfg(feature = "tls")]
    pub tls: Option<crate::tls::TlsConfig>,
}

#[cfg(not(unix))] type BaseBindable = Either<std::net::SocketAddr, std::net::SocketAddr>;
#[cfg(unix)]      type BaseBindable = Either<std::net::SocketAddr, super::unix::UdsConfig>;

#[cfg(not(feature = "tls"))] type TlsBindable<T> = Either<T, T>;
#[cfg(feature = "tls")]      type TlsBindable<T> = Either<super::tls::TlsBindable<T>, T>;

impl DefaultListener {
    pub(crate) fn base_bindable(&self) -> Result<BaseBindable, crate::Error> {
        match &self.address {
            Endpoint::Tcp(mut address) => {
                self.port.map(|port| address.set_port(port));
                Ok(BaseBindable::Left(address))
            },
            #[cfg(unix)]
            Endpoint::Unix(path) => {
                let uds = super::unix::UdsConfig { path: path.clone(), reuse: self.reuse, };
                Ok(BaseBindable::Right(uds))
            },
            #[cfg(not(unix))]
            e@Endpoint::Unix(_) => {
                let msg = "Unix domain sockets unavailable on non-unix platforms.";
                let boxed = Box::<dyn std::error::Error + Send + Sync>::from(msg);
                Err(Error::new(ErrorKind::Bind(Some(e.clone()), boxed)))
            },
            other => {
                let msg = format!("unsupported default listener address: {other}");
                let boxed = Box::<dyn std::error::Error + Send + Sync>::from(msg);
                Err(Error::new(ErrorKind::Bind(Some(other.clone()), boxed)))
            }
        }
    }

    pub(crate) fn tls_bindable<T>(&self, inner: T) -> TlsBindable<T> {
        #[cfg(feature = "tls")]
        if let Some(tls) = self.tls.clone() {
            return TlsBindable::Left(super::tls::TlsBindable { inner, tls });
        }

        TlsBindable::Right(inner)
    }

    pub fn bindable(&self) -> Result<impl Bindable, crate::Error> {
        self.base_bindable()
            .map(|b| b.map_either(|b| self.tls_bindable(b), |b| self.tls_bindable(b)))
    }
}
