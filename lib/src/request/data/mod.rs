//! Talk about the data thing.

mod from_data;
mod data_stream;

pub use self::from_data::{FromData, DataOutcome};

use std::io::{self, BufRead, Read, Write, Cursor, BufReader};
use std::path::Path;
use std::fs::File;
use std::time::Duration;
use std::mem::transmute;

use self::data_stream::{DataStream, StreamReader, kill_stream};
use request::Request;
use http::hyper::{HyperBodyReader, HyperHttpStream};
use http::hyper::HyperNetworkStream;
use http::hyper::HyperHttpReader::*;

pub struct Data {
    buffer: Vec<u8>,
    is_done: bool,
    stream: StreamReader,
    position: usize,
    capacity: usize,
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

    #[inline(always)]
    pub fn peek(&self) -> &[u8] {
        &self.buffer[self.position..self.capacity]
    }

    #[inline(always)]
    pub fn peek_complete(&self) -> bool {
        self.is_done
    }

    #[inline(always)]
    pub fn stream_to<W: Write>(self, writer: &mut W) -> io::Result<u64> {
        io::copy(&mut self.open(), writer)
    }

    #[inline(always)]
    pub fn stream_to_file<P: AsRef<Path>>(self, path: P) -> io::Result<u64> {
        io::copy(&mut self.open(), &mut File::create(path)?)
    }

    pub fn new(mut buf: Vec<u8>,
               pos: usize,
               mut cap: usize,
               mut stream: StreamReader)
               -> Data {
        // TODO: Make sure we always try to get some number of bytes in the
        // buffer so that peek actually does something.

        // Make sure the buffer is large enough for the bytes we want to peek.
        const PEEK_BYTES: usize = 4096;
        if buf.len() < PEEK_BYTES {
            trace!("Resizing peek buffer from {} to {}.", buf.len(), PEEK_BYTES);
            buf.resize(PEEK_BYTES, 0);
        }

        trace!("Init buffer cap: {}", cap);
        let buf_len = buf.len();
        let eof = match stream.read(&mut buf[cap..(buf_len - 1)]) {
            Ok(n) if n == 0 => true,
            Ok(n) => {
                trace!("Filled peek buf with {} bytes.", n);
                cap += n;
                match stream.read(&mut buf[cap..(cap + 1)]) {
                    Ok(n) => {
                        cap += n;
                        n == 0
                    }
                    Err(e) => {
                        error_!("Failed to check stream EOF status: {:?}", e);
                        false
                    }
                }
            }
            Err(e) => {
                error_!("Failed to read into peek buffer: {:?}", e);
                false
            }
        };

        trace!("Peek buffer size: {}, remaining: {}", buf_len, buf_len - cap);
        Data {
            buffer: buf,
            stream: stream,
            is_done: eof,
            position: pos,
            capacity: cap,
        }
    }
}

impl Drop for Data {
    fn drop(&mut self) {
        // This is okay since the network stream expects to be shared mutably.
        unsafe {
            let stream: &mut StreamReader = transmute(self.stream.by_ref());
            kill_stream(stream, self.stream.get_mut());
        }
    }
}

impl FromData for Data {
    type Error = ();

    fn from_data(_: &Request, data: Data) -> DataOutcome<Self, Self::Error> {
        DataOutcome::success(data)
    }
}
