use std::io::{Read, Write, ErrorKind};

use response::{Responder, Outcome, ResponseOutcome};
use http::hyper::FreshHyperResponse;

// TODO: Support custom chunk sizes.
/// The default size of each chunk in the streamed response.
pub const CHUNK_SIZE: usize = 4096;

pub struct Stream<T: Read>(Box<T>);

impl<T: Read> Stream<T> {
    pub fn from(reader: T) -> Stream<T> {
        Stream(Box::new(reader))
    }

    //     pub fn chunked(mut self, size: usize) -> Self {
    //         self.1 = size;
    //         self
    //     }

    //     #[inline(always)]
    //     pub fn chunk_size(&self) -> usize {
    //         self.1
    //     }
}

impl<T: Read> Responder for Stream<T> {
    fn respond<'a>(&mut self, res: FreshHyperResponse<'a>) -> ResponseOutcome<'a> {
        let mut stream = res.start().unwrap();
        let mut buffer = [0; CHUNK_SIZE];
        let mut complete = false;
        while !complete {
            let mut read = 0;
            while read < buffer.len() && !complete {
                match self.0.read(&mut buffer[read..]) {
                    Ok(n) if n == 0 => complete = true,
                    Ok(n) => read += n,
                    Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
                    Err(ref e) => {
                        error_!("Error streaming response: {:?}", e);
                        return Outcome::FailStop;
                    }
                }
            }

            if let Err(e) = stream.write_all(&buffer[..read]) {
                error_!("Stream write_all() failed: {:?}", e);
                return Outcome::FailStop;
            }
        }

        if let Err(e) = stream.end() {
            error_!("Stream end() failed: {:?}", e);
            return Outcome::FailStop;
        }

        Outcome::Success
    }
}
