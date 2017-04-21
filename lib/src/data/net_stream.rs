use std::io::{self, Cursor};
use std::net::{SocketAddr, Shutdown};
use std::time::Duration;

#[cfg(feature = "tls")] use hyper_rustls::WrappedStream as RustlsStream;
use http::hyper::net::{HttpStream, NetworkStream};

use self::NetStream::*;

// This is a representation of all of the possible network streams we might get.
// This really shouldn't be necessary, but, you know, Hyper.
#[derive(Clone)]
pub enum NetStream {
    Http(HttpStream),
    #[cfg(feature = "tls")]
    Https(RustlsStream),
    Local(Cursor<Vec<u8>>)
}

impl io::Read for NetStream {
    #[inline(always)]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match *self {
            Http(ref mut stream) => stream.read(buf),
            Local(ref mut stream) => stream.read(buf),
            #[cfg(feature = "tls")] Https(ref mut stream) => stream.read(buf)
        }
    }
}

impl io::Write for NetStream {
    #[inline(always)]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match *self {
            Http(ref mut stream) => stream.write(buf),
            Local(ref mut stream) => stream.write(buf),
            #[cfg(feature = "tls")] Https(ref mut stream) => stream.write(buf)
        }
    }

    #[inline(always)]
    fn flush(&mut self) -> io::Result<()> {
        match *self {
            Http(ref mut stream) => stream.flush(),
            Local(ref mut stream) => stream.flush(),
            #[cfg(feature = "tls")] Https(ref mut stream) => stream.flush()
        }
    }
}

impl NetworkStream for NetStream {
    #[inline(always)]
    fn peer_addr(&mut self) -> io::Result<SocketAddr> {
        match *self {
            Http(ref mut stream) => stream.peer_addr(),
            #[cfg(feature = "tls")] Https(ref mut stream) => stream.peer_addr(),
            Local(_) => Err(io::Error::from(io::ErrorKind::AddrNotAvailable)),
        }
    }

    #[inline(always)]
    fn set_read_timeout(&self, dur: Option<Duration>) -> io::Result<()> {
        match *self {
            Http(ref stream) => stream.set_read_timeout(dur),
            #[cfg(feature = "tls")] Https(ref stream) => stream.set_read_timeout(dur),
            Local(_) => Ok(()),
        }
    }

    #[inline(always)]
    fn set_write_timeout(&self, dur: Option<Duration>) -> io::Result<()> {
        match *self {
            Http(ref stream) => stream.set_write_timeout(dur),
            #[cfg(feature = "tls")] Https(ref stream) => stream.set_write_timeout(dur),
            Local(_) => Ok(()),
        }
    }

    #[inline(always)]
    fn close(&mut self, how: Shutdown) -> io::Result<()> {
        match *self {
            Http(ref mut stream) => stream.close(how),
            #[cfg(feature = "tls")] Https(ref mut stream) => stream.close(how),
            Local(_) => Ok(()),
        }
    }
}
