use response::*;
use std::io::{Read, Write, ErrorKind};

/// The size of each chunk in the streamed response.
pub const CHUNK_SIZE: usize = 4096;

pub struct Stream<T: Read>(pub Box<T>);

impl<T: Read> Stream<T> {
    pub fn from(reader: T) -> Stream<T> {
        Stream(Box::new(reader))
    }
}

impl<T: Read> Responder for Stream<T> {
    fn respond<'a>(&mut self, res: FreshHyperResponse<'a>) -> Outcome<'a> {
        let mut stream = res.start().unwrap();
        let mut buffer = [0; CHUNK_SIZE];
        let mut complete = false;
        while !complete {
            let mut left = CHUNK_SIZE;
            while left > 0 && !complete {
                match self.0.read(&mut buffer[..left]) {
                    Ok(n) if n == 0 => complete = true,
                    Ok(n) if n < left => left -= n,
                    Ok(n) if n == left => left = CHUNK_SIZE,
                    Ok(n) => unreachable!("Impossible byte count {}/{}!", n, left),
                    Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
                    Err(ref e) => {
                        error_!("Error streaming response: {:?}", e);
                        return Outcome::FailStop;
                    }
                }
            }

            if let Err(e) = stream.write_all(&buffer) {
                error_!("Stream write_all() failed: {:?}", e);
                return Outcome::FailStop;
            }
        }

        if let Err(e) = stream.end() {
            error_!("Stream end() failed: {:?}", e);
            return Outcome::FailStop;
        }

        Outcome::Complete
    }
}
