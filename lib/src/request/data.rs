use std::io::{BufRead, Read, Cursor, BufReader};

pub struct Data {
    buffer: Vec<u8>,
    stream: Cursor<Vec<u8>>,
}

impl Data {
    pub fn open(self) -> impl BufRead {
        Cursor::new(self.buffer).chain(BufReader::new(self.stream))
    }

    pub fn peek(&self) -> &[u8] {
        &self.buffer
    }

    pub fn new() -> Data {
        Data {
            stream: Cursor::new(vec![]),
            buffer: vec![]
        }
    }
}
