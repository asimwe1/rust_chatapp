use std::io::{self, BufRead, Write, Cursor, BufReader};
use std::path::Path;
use std::fs::File;

use http::hyper::h1::HttpReader;
use http::hyper::net::NetworkStream;
use http::hyper::buffer;

pub type BodyReader<'a, 'b> =
    self::HttpReader<&'a mut self::buffer::BufReader<&'b mut NetworkStream>>;

const PEEK_BYTES: usize = 4096;

pub struct Data {
    data: Vec<u8>,
}

impl Data {
    pub fn open(self) -> impl BufRead {
        BufReader::new(Cursor::new(self.data))
    }

    #[inline(always)]
    pub fn peek(&self) -> &[u8] {
        &self.data[..::std::cmp::min(PEEK_BYTES, self.data.len())]
    }

    #[inline(always)]
    pub fn peek_complete(&self) -> bool {
        self.data.len() <= PEEK_BYTES
    }

    #[inline(always)]
    pub fn stream_to<W: Write>(self, writer: &mut W) -> io::Result<u64> {
        io::copy(&mut self.open(), writer)
    }

    #[inline(always)]
    pub fn stream_to_file<P: AsRef<Path>>(self, path: P) -> io::Result<u64> {
        io::copy(&mut self.open(), &mut File::create(path)?)
    }

    #[doc(hidden)]
    pub fn from_hyp(mut h_body: BodyReader) -> Result<Data, &'static str> {
        let mut vec = Vec::new();
        if let Err(_) = io::copy(&mut h_body, &mut vec) {
            return Err("Reading from body failed.");
        };

        Ok(Data::new(vec))
    }

    #[doc(hidden)]
    pub fn new(data: Vec<u8>) -> Data {
        Data { data: data }
    }
}
