use std::io::{self, BufRead, Read, Cursor, BufReader, Chain, Take};
use std::net::{SocketAddr, Shutdown};
use std::time::Duration;

#[cfg(feature = "tls")] use hyper_rustls::WrappedStream as RustlsStream;

use http::hyper::net::{HttpStream, NetworkStream};
use http::hyper::h1::HttpReader;

pub type StreamReader = HttpReader<HyperNetStream>;
pub type InnerStream = Chain<Take<Cursor<Vec<u8>>>, BufReader<StreamReader>>;

#[derive(Clone)]
pub enum HyperNetStream {
    Http(HttpStream),
    #[cfg(feature = "tls")]
    Https(RustlsStream)
}

macro_rules! with_inner {
    ($net:expr, |$stream:ident| $body:expr) => ({
        trace!("{}:{}", file!(), line!());
        match *$net {
            HyperNetStream::Http(ref $stream) => $body,
            #[cfg(feature = "tls")] HyperNetStream::Https(ref $stream) => $body
        }
    });
    ($net:expr, |mut $stream:ident| $body:expr) => ({
        trace!("{}:{}", file!(), line!());
        match *$net {
            HyperNetStream::Http(ref mut $stream) => $body,
            #[cfg(feature = "tls")] HyperNetStream::Https(ref mut $stream) => $body
        }
    })
}

impl io::Read for HyperNetStream {
    #[inline(always)]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        with_inner!(self, |mut stream| io::Read::read(stream, buf))
    }
}

impl io::Write for HyperNetStream {
    #[inline(always)]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        with_inner!(self, |mut stream| io::Write::write(stream, buf))
    }

    #[inline(always)]
    fn flush(&mut self) -> io::Result<()> {
        with_inner!(self, |mut stream| io::Write::flush(stream))
    }
}

impl NetworkStream for HyperNetStream {
    #[inline(always)]
    fn peer_addr(&mut self) -> io::Result<SocketAddr> {
        with_inner!(self, |mut stream| NetworkStream::peer_addr(stream))
    }

    #[inline(always)]
    fn set_read_timeout(&self, dur: Option<Duration>) -> io::Result<()> {
        with_inner!(self, |stream| NetworkStream::set_read_timeout(stream, dur))
    }

    #[inline(always)]
    fn set_write_timeout(&self, dur: Option<Duration>) -> io::Result<()> {
        with_inner!(self, |stream| NetworkStream::set_write_timeout(stream, dur))
    }

    #[inline(always)]
    fn close(&mut self, how: Shutdown) -> io::Result<()> {
        with_inner!(self, |mut stream| NetworkStream::close(stream, how))
    }
}

/// Raw data stream of a request body.
///
/// This stream can only be obtained by calling
/// [Data::open](/rocket/data/struct.Data.html#method.open). The stream contains
/// all of the data in the body of the request. It exposes no methods directly.
/// Instead, it must be used as an opaque `Read` or `BufRead` structure.
pub struct DataStream {
    stream: InnerStream,
    network: HyperNetStream,
}

impl DataStream {
    #[inline(always)]
    pub(crate) fn new(stream: InnerStream, network: HyperNetStream) -> DataStream {
        DataStream { stream, network }
    }
}

impl Read for DataStream {
    #[inline(always)]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.stream.read(buf)
    }
}

impl BufRead for DataStream {
    #[inline(always)]
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.stream.fill_buf()
    }

    #[inline(always)]
    fn consume(&mut self, amt: usize) {
        self.stream.consume(amt)
    }
}

// pub fn kill_stream<S: Read>(stream: &mut S, network: &mut HyperNetStream) {
pub fn kill_stream<S: Read, N: NetworkStream>(stream: &mut S, network: &mut N) {
    io::copy(&mut stream.take(1024), &mut io::sink()).expect("kill_stream: sink");

    // If there are any more bytes, kill it.
    let mut buf = [0];
    if let Ok(n) = stream.read(&mut buf) {
        if n > 0 {
            warn_!("Data left unread. Force closing network stream.");
            if let Err(e) = network.close(Shutdown::Both) {
                error_!("Failed to close network stream: {:?}", e);
            }
        }
    }
}

impl Drop for DataStream {
    // Be a bad citizen and close the TCP stream if there's unread data.
    fn drop(&mut self) {
        kill_stream(&mut self.stream, &mut self.network);
    }
}
