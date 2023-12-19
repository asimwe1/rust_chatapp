use std::fmt;
use std::path::{Path, PathBuf};
use std::any::Any;
use std::net::{SocketAddr as TcpAddr, Ipv4Addr, AddrParseError};
use std::str::FromStr;
use std::sync::Arc;

use serde::de;

use crate::http::uncased::AsUncased;

pub trait EndpointAddr: fmt::Display + fmt::Debug + Sync + Send + Any { }

impl<T: fmt::Display + fmt::Debug + Sync + Send + Any> EndpointAddr for T {}

#[cfg(not(feature = "tls"))] type TlsInfo = Option<()>;
#[cfg(feature = "tls")]      type TlsInfo = Option<crate::tls::TlsConfig>;

/// # Conversions
///
/// * [`&str`] - parse with [`FromStr`]
/// * [`tokio::net::unix::SocketAddr`] - must be path: [`ListenerAddr::Unix`]
/// * [`std::net::SocketAddr`] - infallibly as [ListenerAddr::Tcp]
/// * [`PathBuf`] - infallibly as [`ListenerAddr::Unix`]
// TODO: Rename to something better. `Endpoint`?
#[derive(Debug)]
pub enum Endpoint {
    Tcp(TcpAddr),
    Unix(PathBuf),
    Tls(Arc<Endpoint>, TlsInfo),
    Custom(Arc<dyn EndpointAddr>),
}

impl Endpoint {
    pub fn new<T: EndpointAddr>(value: T) -> Endpoint {
        Endpoint::Custom(Arc::new(value))
    }

    pub fn tcp(&self) -> Option<TcpAddr> {
        match self {
            Endpoint::Tcp(addr) => Some(*addr),
            _ => None,
        }
    }

    pub fn unix(&self) -> Option<&Path> {
        match self {
            Endpoint::Unix(addr) => Some(addr),
            _ => None,
        }
    }

    pub fn tls(&self) -> Option<&Endpoint> {
        match self {
            Endpoint::Tls(addr, _) => Some(addr),
            _ => None,
        }
    }

    #[cfg(feature = "tls")]
    pub fn tls_config(&self) -> Option<&crate::tls::TlsConfig> {
        match self {
            Endpoint::Tls(_, Some(ref config)) => Some(config),
            _ => None,
        }
    }

    #[cfg(feature = "mtls")]
    pub fn mtls_config(&self) -> Option<&crate::mtls::MtlsConfig> {
        match self {
            Endpoint::Tls(_, Some(config)) => config.mutual(),
            _ => None,
        }
    }

    pub fn downcast<T: 'static>(&self) -> Option<&T> {
        match self {
            Endpoint::Tcp(addr) => (&*addr as &dyn Any).downcast_ref(),
            Endpoint::Unix(addr) => (&*addr as &dyn Any).downcast_ref(),
            Endpoint::Custom(addr) => (&*addr as &dyn Any).downcast_ref(),
            Endpoint::Tls(inner, ..) => inner.downcast(),
        }
    }

    pub fn is_tcp(&self) -> bool {
        self.tcp().is_some()
    }

    pub fn is_unix(&self) -> bool {
        self.unix().is_some()
    }

    pub fn is_tls(&self) -> bool {
        self.tls().is_some()
    }

    #[cfg(feature = "tls")]
    pub fn with_tls(self, config: crate::tls::TlsConfig) -> Endpoint {
        if self.is_tls() {
            return self;
        }

        Self::Tls(Arc::new(self), Some(config))
    }

    pub fn assume_tls(self) -> Endpoint {
        if self.is_tls() {
            return self;
        }

        Self::Tls(Arc::new(self), None)
    }
}

impl fmt::Display for Endpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Endpoint::*;

        match self {
            Tcp(addr) => write!(f, "http://{addr}"),
            Unix(addr) => write!(f, "unix:{}", addr.display()),
            Custom(inner) => inner.fmt(f),
            Tls(inner, c) => match (&**inner, c.as_ref()) {
                #[cfg(feature = "mtls")]
                (Tcp(i), Some(c)) if c.mutual().is_some() => write!(f, "https://{i} (TLS + MTLS)"),
                (Tcp(i), _) => write!(f, "https://{i} (TLS)"),
                #[cfg(feature = "mtls")]
                (i, Some(c)) if c.mutual().is_some() => write!(f, "{i} (TLS + MTLS)"),
                (inner, _) => write!(f, "{inner} (TLS)"),
            },
        }
    }
}

impl From<std::net::SocketAddr> for Endpoint {
    fn from(value: std::net::SocketAddr) -> Self {
        Self::Tcp(value)
    }
}

impl From<std::net::SocketAddrV4> for Endpoint {
    fn from(value: std::net::SocketAddrV4) -> Self {
        Self::Tcp(value.into())
    }
}

impl From<std::net::SocketAddrV6> for Endpoint {
    fn from(value: std::net::SocketAddrV6) -> Self {
        Self::Tcp(value.into())
    }
}

impl From<PathBuf> for Endpoint {
    fn from(value: PathBuf) -> Self {
        Self::Unix(value)
    }
}

#[cfg(unix)]
impl TryFrom<tokio::net::unix::SocketAddr> for Endpoint {
    type Error = std::io::Error;

    fn try_from(v: tokio::net::unix::SocketAddr) -> Result<Self, Self::Error> {
        v.as_pathname()
            .ok_or_else(|| std::io::Error::other("unix socket is not path"))
            .map(|path| Endpoint::Unix(path.to_path_buf()))
    }
}

impl TryFrom<&str> for Endpoint {
    type Error = AddrParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl Default for Endpoint {
    fn default() -> Self {
        Endpoint::Tcp(TcpAddr::new(Ipv4Addr::LOCALHOST.into(), 8000))
    }
}

/// Parses an address into a `ListenerAddr`.
///
/// The syntax is:
///
/// ```text
/// listener_addr = 'tcp' ':' tcp_addr | 'unix' ':' unix_addr | tcp_addr
/// tcp_addr := IP_ADDR | SOCKET_ADDR
/// unix_addr := PATH
///
/// IP_ADDR := `std::net::IpAddr` string as defined by Rust
/// SOCKET_ADDR := `std::net::SocketAddr` string as defined by Rust
/// PATH := `PathBuf` (any UTF-8) string as defined by Rust
/// ```
///
/// If `IP_ADDR` is specified, the port defaults to `8000`.
impl FromStr for Endpoint {
    type Err = AddrParseError;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        fn parse_tcp(string: &str, def_port: u16) -> Result<TcpAddr, AddrParseError> {
            string.parse().or_else(|_| string.parse().map(|ip| TcpAddr::new(ip, def_port)))
        }

        if let Some((proto, string)) = string.split_once(':') {
            if proto.trim().as_uncased() == "tcp" {
                return parse_tcp(string.trim(), 8000).map(Self::Tcp);
            } else if proto.trim().as_uncased() == "unix" {
                return Ok(Self::Unix(PathBuf::from(string.trim())));
            }
        }

        parse_tcp(string.trim(), 8000).map(Self::Tcp)
    }
}

impl<'de> de::Deserialize<'de> for Endpoint {
    fn deserialize<D: de::Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = Endpoint;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("TCP or Unix address")
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                v.parse::<Endpoint>().map_err(|e| E::custom(e.to_string()))
            }
        }

        de.deserialize_any(Visitor)
    }
}

impl Eq for Endpoint { }

impl PartialEq for Endpoint {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Tcp(l0), Self::Tcp(r0)) => l0 == r0,
            (Self::Unix(l0), Self::Unix(r0)) => l0 == r0,
            (Self::Tls(l0, _), Self::Tls(r0, _)) => l0 == r0,
            (Self::Custom(l0), Self::Custom(r0)) => l0.to_string() == r0.to_string(),
            _ => false,
        }
    }
}

impl PartialEq<std::net::SocketAddr> for Endpoint {
    fn eq(&self, other: &std::net::SocketAddr) -> bool {
        self.tcp() == Some(*other)
    }
}

impl PartialEq<std::net::SocketAddrV4> for Endpoint {
    fn eq(&self, other: &std::net::SocketAddrV4) -> bool {
        self.tcp() == Some((*other).into())
    }
}

impl PartialEq<std::net::SocketAddrV6> for Endpoint {
    fn eq(&self, other: &std::net::SocketAddrV6) -> bool {
        self.tcp() == Some((*other).into())
    }
}

impl PartialEq<PathBuf> for Endpoint {
    fn eq(&self, other: &PathBuf) -> bool {
        self.unix() == Some(other.as_path())
    }
}

impl PartialEq<Path> for Endpoint {
    fn eq(&self, other: &Path) -> bool {
        self.unix() == Some(other)
    }
}
