use std::io::{BufRead, Read, Cursor, BufReader};
use std::net::TcpStream;

use request::Request;

pub struct Data {
    stream: Cursor<Vec<u8>>,
    buffer: Vec<u8>
}

impl Data {
    fn open(self) -> impl BufRead {
        Cursor::new(self.buffer).chain(BufReader::new(self.stream))
    }

    fn peek(&self) -> &[u8] {
        &self.buffer
    }

    pub fn new() -> Data {
        Data {
            stream: Cursor::new(vec![]),
            buffer: vec![]
        }
    }
}
