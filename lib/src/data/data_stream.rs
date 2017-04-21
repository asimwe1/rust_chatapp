use std::io::{self, BufRead, Read, Cursor, BufReader, Chain, Take};
use std::net::Shutdown;

use super::net_stream::NetStream;

use http::hyper::net::NetworkStream;
use http::hyper::h1::HttpReader;

pub type StreamReader = HttpReader<NetStream>;
pub type InnerStream = Chain<Take<Cursor<Vec<u8>>>, BufReader<StreamReader>>;

/// Raw data stream of a request body.
///
/// This stream can only be obtained by calling
/// [Data::open](/rocket/data/struct.Data.html#method.open). The stream contains
/// all of the data in the body of the request. It exposes no methods directly.
/// Instead, it must be used as an opaque `Read` or `BufRead` structure.
pub struct DataStream {
    stream: InnerStream,
    network: NetStream,
}

impl DataStream {
    #[inline(always)]
    pub(crate) fn new(stream: InnerStream, network: NetStream) -> DataStream {
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


pub fn kill_stream<S: Read, N: NetworkStream>(stream: &mut S, network: &mut N) {
    // Take <= 1k from the stream. If there might be more data, force close.
    const FLUSH_LEN: u64 = 1024;
    match io::copy(&mut stream.take(FLUSH_LEN), &mut io::sink()) {
        Ok(FLUSH_LEN) | Err(_) => {
            warn_!("Data left unread. Force closing network stream.");
            if let Err(e) = network.close(Shutdown::Both) {
                error_!("Failed to close network stream: {:?}", e);
            }
        }
        Ok(n) => debug!("flushed {} unread bytes", n)
    }
}

impl Drop for DataStream {
    // Be a bad citizen and close the TCP stream if there's unread data.
    fn drop(&mut self) {
        kill_stream(&mut self.stream, &mut self.network);
    }
}
