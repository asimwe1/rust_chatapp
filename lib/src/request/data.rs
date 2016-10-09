use std::io::{self, BufRead, Read, Cursor, BufReader, Chain, Take};
use std::time::Duration;
use std::net::Shutdown;

use http::hyper::{HyperBodyReader, HyperHttpStream, HyperHttpReader};
use http::hyper::HyperNetworkStream;
use http::hyper::HyperHttpReader::*;

type StreamReader = HyperHttpReader<HyperHttpStream>;

pub struct DataStream {
    stream: Chain<Take<Cursor<Vec<u8>>>, BufReader<StreamReader>>,
    network: HyperHttpStream,
}

impl Read for DataStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.stream.read(buf)
    }
}

impl BufRead for DataStream {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.stream.fill_buf()
    }

    fn consume(&mut self, amt: usize) {
        self.stream.consume(amt)
    }
}

fn try_sinking<N: HyperNetworkStream>(net: &mut N) -> bool {
    warn_!("Data left unread. Sinking 1k bytes.");
    io::copy(&mut net.take(1024), &mut io::sink()).expect("sink");

    // If there are any more bytes, kill it.
    let mut buf = [0];
    if let Ok(n) = net.read(&mut buf) {
        if n > 0 {
            warn_!("Data still remains. Force closing network stream.");
            return net.close(Shutdown::Both).is_ok();
        }
    }

    false
}

impl Drop for DataStream {
    // Be a bad citizen and close the TCP stream if there's unread data.
    // Unfortunately, Hyper forces us to do this.
    fn drop(&mut self) {
        try_sinking(&mut self.network);
    }
}

pub struct Data {
    buffer: Vec<u8>,
    stream: StreamReader,
    position: usize,
    capacity: usize,
}

impl Drop for Data {
    fn drop(&mut self) {
        try_sinking(self.stream.get_mut());
    }
}

impl Data {
    pub fn open(mut self) -> impl BufRead {
        // Swap out the buffer and stream for empty ones so we can move.
        let mut buffer = vec![];
        let mut stream = EmptyReader(self.stream.get_ref().clone());
        ::std::mem::swap(&mut buffer, &mut self.buffer);
        ::std::mem::swap(&mut stream, &mut self.stream);

        // Setup the underlying reader at the correct pointers.
        let mut cursor = Cursor::new(buffer);
        cursor.set_position(self.position as u64);
        let buffered = cursor.take((self.capacity - self.position) as u64);

        // Create the actual DataSteam.
        DataStream {
            network: stream.get_ref().clone(),
            stream: buffered.chain(BufReader::new(stream)),
        }
    }

    #[doc(hidden)]
    pub fn from_hyp(mut h_body: HyperBodyReader) -> Result<Data, &'static str> {
        // FIXME: This is asolutely terrible, thanks to Hyper.

        // Retrieve the underlying HTTPStream from Hyper.
        let mut stream = match h_body.get_ref().get_ref()
                                     .downcast_ref::<HyperHttpStream>() {
            Some(s) => {
                let owned_stream = s.clone();
                let buf_len = h_body.get_ref().get_buf().len() as u64;
                match h_body {
                    SizedReader(_, n) => SizedReader(owned_stream, n - buf_len),
                    EofReader(_) => EofReader(owned_stream),
                    EmptyReader(_) => EmptyReader(owned_stream),
                    ChunkedReader(_, n) =>
                        ChunkedReader(owned_stream, n.map(|k| k - buf_len)),
                }
            },
            None => return Err("Stream is not an HTTP stream!"),
        };

        // Set the read timeout to 5 seconds.
        stream.get_mut().set_read_timeout(Some(Duration::from_secs(5))).unwrap();

        // Create the Data object from hyper's buffer.
        let (vec, pos, cap) = h_body.get_mut().take_buf();
        Ok(Data::new(vec, pos, cap, stream))
    }

    pub fn peek(&self) -> &[u8] {
        &self.buffer[self.position..self.capacity]
    }

    pub fn new(buf: Vec<u8>, pos: usize, cap: usize, stream: StreamReader) -> Data {
        // TODO: Make sure we always try to get some number of bytes in the
        // buffer so that peek actually does something.
        // const PEEK_BYTES: usize = 4096;
        Data {
            buffer: buf,
            stream: stream,
            position: pos,
            capacity: cap,
        }
    }
}
