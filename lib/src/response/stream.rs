use std::io::{Read, Write, ErrorKind};
use std::fmt::{self, Debug};

use response::{Responder, Outcome};
use http::hyper::FreshHyperResponse;
use outcome::Outcome::*;

// TODO: Support custom chunk sizes.
/// The default size of each chunk in the streamed response.
pub const CHUNK_SIZE: usize = 4096;

/// Streams a response to a client from an arbitrary `Read`er type.
///
/// The client is sent a "chunked" response, where the chunk size is at most
/// 4KiB. This means that at most 4KiB are stored in memory while the response
/// is being sent. This type should be used when sending responses that are
/// arbitrarily large in size, such as when streaming from a local socket.
pub struct Stream<T: Read>(T);

impl<T: Read> Stream<T> {
    /// Create a new stream from the given `reader`.
    ///
    /// # Example
    ///
    /// Stream a response from whatever is in `stdin`. Note: you probably
    /// shouldn't do this.
    ///
    /// ```rust
    /// use std::io;
    /// use rocket::response::Stream;
    ///
    /// let response = Stream::from(io::stdin());
    /// ```
    pub fn from(reader: T) -> Stream<T> {
        Stream(reader)
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

impl<T: Read + Debug> Debug for Stream<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Stream({:?})", self.0)
    }
}

/// Sends a response to the client using the "Chunked" transfer encoding. The
/// maximum chunk size is 4KiB.
///
/// # Failure
///
/// If reading from the input stream fails at any point during the response, the
/// response is abandoned, and the response ends abruptly. An error is printed
/// to the console with an indication of what went wrong.
impl<T: Read> Responder for Stream<T> {
    fn respond<'a>(&mut self, res: FreshHyperResponse<'a>) -> Outcome<'a> {
        let mut stream = match res.start() {
            Ok(s) => s,
            Err(ref err) => {
                error_!("Failed opening response stream: {:?}", err);
                return Failure(());
            }
        };

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
                        return Failure(());
                    }
                }
            }

            if let Err(e) = stream.write_all(&buffer[..read]) {
                error_!("Stream write_all() failed: {:?}", e);
                return Failure(());
            }
        }

        if let Err(e) = stream.end() {
            error_!("Stream end() failed: {:?}", e);
            return Failure(());
        }

        Success(())
    }
}
