use std::{io, fmt, str};
use std::borrow::Cow;

use http::{Header, HeaderMap};
use response::Responder;
use http::Status;

pub const DEFAULT_CHUNK_SIZE: u64 = 4096;

pub enum Body<T> {
    Sized(T, u64),
    Chunked(T, u64)
}

impl<T> Body<T> {
    pub fn as_mut(&mut self) -> Body<&mut T> {
        match *self {
            Body::Sized(ref mut b, n) => Body::Sized(b, n),
            Body::Chunked(ref mut b, n) => Body::Chunked(b, n)
        }
    }

    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Body<U> {
        match self {
            Body::Sized(b, n) => Body::Sized(f(b), n),
            Body::Chunked(b, n) => Body::Chunked(f(b), n)
        }
    }
}

impl<T: io::Read> Body<T> {
    pub fn into_string(self) -> Option<String> {
        let (mut body, mut string) = match self {
            Body::Sized(b, size) => (b, String::with_capacity(size as usize)),
            Body::Chunked(b, _) => (b, String::new())
        };

        if let Err(e) = body.read_to_string(&mut string) {
            error_!("Error reading body: {:?}", e);
            return None;
        }

        Some(string)
    }
}

impl<T> fmt::Debug for Body<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Body::Sized(_, n) => writeln!(f, "Sized Body [{} bytes]", n),
            Body::Chunked(_, n) => writeln!(f, "Chunked Body [{} bytes]", n),
        }
    }
}

pub struct ResponseBuilder<'r> {
    response: Response<'r>
}

impl<'r> ResponseBuilder<'r> {
    #[inline(always)]
    pub fn new(base: Response<'r>) -> ResponseBuilder<'r> {
        ResponseBuilder {
            response: base
        }
    }

    #[inline(always)]
    pub fn status(&mut self, status: Status) -> &mut ResponseBuilder<'r> {
        self.response.set_status(status);
        self
    }

    #[inline(always)]
    pub fn raw_status(&mut self, code: u16, reason: &'static str)
            -> &mut ResponseBuilder<'r> {
        self.response.set_raw_status(code, reason);
        self
    }

    #[inline(always)]
    pub fn header<'h: 'r, H>(&mut self, header: H) -> &mut ResponseBuilder<'r>
        where H: Into<Header<'h>>
    {
        self.response.set_header(header);
        self
    }

    #[inline(always)]
    pub fn header_adjoin<'h: 'r, H>(&mut self, header: H) -> &mut ResponseBuilder<'r>
        where H: Into<Header<'h>>
    {
        self.response.adjoin_header(header);
        self
    }

    #[inline(always)]
    pub fn raw_header<'a: 'r, 'b: 'r, N, V>(&mut self, name: N, value: V)
            -> &mut ResponseBuilder<'r>
        where N: Into<Cow<'a, str>>, V: Into<Cow<'b, str>>
    {
        self.response.set_raw_header(name, value);
        self
    }

    #[inline(always)]
    pub fn raw_header_adjoin<'a: 'r, 'b: 'r, N, V>(&mut self, name: N, value: V)
            -> &mut ResponseBuilder<'r>
        where N: Into<Cow<'a, str>>, V: Into<Cow<'b, str>>
    {
        self.response.adjoin_raw_header(name, value);
        self
    }

    #[inline(always)]
    pub fn sized_body<B>(&mut self, body: B) -> &mut ResponseBuilder<'r>
        where B: io::Read + io::Seek + 'r
    {
        self.response.set_sized_body(body);
        self
    }

    #[inline(always)]
    pub fn streamed_body<B>(&mut self, body: B) -> &mut ResponseBuilder<'r>
        where B: io::Read + 'r
    {
        self.response.set_streamed_body(body);
        self
    }

    #[inline(always)]
    pub fn chunked_body<B: io::Read + 'r>(&mut self, body: B, chunk_size: u64)
            -> &mut ResponseBuilder<'r>
    {
        self.response.set_chunked_body(body, chunk_size);
        self
    }

    #[inline(always)]
    pub fn merge(&mut self, other: Response<'r>) -> &mut ResponseBuilder<'r> {
        self.response.merge(other);
        self
    }

    #[inline(always)]
    pub fn join(&mut self, other: Response<'r>) -> &mut ResponseBuilder<'r> {
        self.response.join(other);
        self
    }

    #[inline(always)]
    pub fn finalize(&mut self) -> Response<'r> {
        ::std::mem::replace(&mut self.response, Response::new())
    }

    #[inline(always)]
    pub fn ok<T>(&mut self) -> Result<Response<'r>, T> {
        Ok(self.finalize())
    }
}

// `join`? Maybe one does one thing, the other does another? IE: `merge`
// replaces, `join` adds. One more thing that could be done: we could make it
// some that _some_ headers default to replacing, and other to joining.
/// Return type of a thing.
#[derive(Default)]
pub struct Response<'r> {
    status: Option<Status>,
    headers: HeaderMap<'r>,
    body: Option<Body<Box<io::Read + 'r>>>,
}

impl<'r> Response<'r> {
    #[inline(always)]
    pub fn new() -> Response<'r> {
        Response {
            status: None,
            headers: HeaderMap::new(),
            body: None,
        }
    }

    #[inline(always)]
    pub fn build() -> ResponseBuilder<'r> {
        Response::build_from(Response::new())
    }

    #[inline(always)]
    pub fn build_from(other: Response<'r>) -> ResponseBuilder<'r> {
        ResponseBuilder::new(other)
    }

    #[inline(always)]
    pub fn status(&self) -> Status {
        self.status.unwrap_or(Status::Ok)
    }

    #[inline(always)]
    pub fn set_status(&mut self, status: Status) {
        self.status = Some(status);
    }

    #[inline(always)]
    pub fn set_raw_status(&mut self, code: u16, reason: &'static str) {
        self.status = Some(Status::new(code, reason));
    }

    #[inline(always)]
    pub fn headers<'a>(&'a self) -> impl Iterator<Item=Header<'a>> {
        self.headers.iter()
    }

    #[inline(always)]
    pub fn get_header_values<'h>(&'h self, name: &str)
            -> impl Iterator<Item=&'h str> {
        self.headers.get(name)
    }

    #[inline(always)]
    pub fn set_header<'h: 'r, H: Into<Header<'h>>>(&mut self, header: H) -> bool {
        self.headers.replace(header)
    }

    #[inline(always)]
    pub fn set_raw_header<'a: 'r, 'b: 'r, N, V>(&mut self, name: N, value: V) -> bool
        where N: Into<Cow<'a, str>>, V: Into<Cow<'b, str>>
    {
        self.set_header(Header::new(name, value))
    }

    #[inline(always)]
    pub fn adjoin_header<'h: 'r, H: Into<Header<'h>>>(&mut self, header: H) {
        self.headers.add(header)
    }

    #[inline(always)]
    pub fn adjoin_raw_header<'a: 'r, 'b: 'r, N, V>(&mut self, name: N, value: V)
        where N: Into<Cow<'a, str>>, V: Into<Cow<'b, str>>
    {
        self.adjoin_header(Header::new(name, value));
    }

    #[inline(always)]
    pub fn remove_header(&mut self, name: &str) {
        self.headers.remove(name);
    }

    #[inline(always)]
    pub fn body(&mut self) -> Option<Body<&mut io::Read>> {
        // Looks crazy, right? Needed so Rust infers lifetime correctly. Weird.
        match self.body.as_mut() {
            Some(body) => Some(match body.as_mut() {
                Body::Sized(b, size) => Body::Sized(b, size),
                Body::Chunked(b, chunk_size) => Body::Chunked(b, chunk_size),
            }),
            None => None
        }
    }

    #[inline(always)]
    pub fn take_body(&mut self) -> Option<Body<Box<io::Read + 'r>>> {
        self.body.take()
    }

    // Removes any actual body, but leaves the size if it exists. Only meant to
    // be used to handle HEAD requests automatically.
    #[doc(hidden)]
    #[inline(always)]
    pub fn strip_body(&mut self) {
        if let Some(body) = self.take_body() {
            self.body = match body {
                Body::Sized(_, n) => Some(Body::Sized(Box::new(io::empty()), n)),
                Body::Chunked(..) => None
            };
        }
    }

    #[inline(always)]
    pub fn set_sized_body<B>(&mut self, mut body: B)
        where B: io::Read + io::Seek + 'r
    {
        let size = body.seek(io::SeekFrom::End(0))
            .expect("Attempted to retrieve size by seeking, but failed.");
        body.seek(io::SeekFrom::Start(0))
            .expect("Attempted to reset body by seeking after getting size.");
        self.body = Some(Body::Sized(Box::new(body), size));
    }

    #[inline(always)]
    pub fn set_streamed_body<B>(&mut self, body: B) where B: io::Read + 'r {
        self.set_chunked_body(body, DEFAULT_CHUNK_SIZE);
    }

    #[inline(always)]
    pub fn set_chunked_body<B>(&mut self, body: B, chunk_size: u64)
            where B: io::Read + 'r {
        self.body = Some(Body::Chunked(Box::new(body), chunk_size));
    }

    /// Replaces this response's status and body with that of `other`, if they
    /// exist in `other`. Any headers that exist in `other` replace the ones in
    /// `self`. Any in `self` that aren't in `other` remain in `self`.
    pub fn merge(&mut self, other: Response<'r>) {
        if let Some(status) = other.status {
            self.status = Some(status);
        }

        if let Some(body) = other.body {
            self.body = Some(body);
        }

        for (name, values) in other.headers.into_iter_raw() {
            self.headers.replace_all(name, values);
        }
    }

    // Sets `self`'s status and body to that of `other` if they are not already
    // set in `self`. Any headers present in both `other` and `self` are
    // adjoined.
    pub fn join(&mut self, other: Response<'r>) {
        if self.status.is_none() {
            self.status = other.status;
        }

        if self.body.is_none() {
            self.body = other.body;
        }

        for (name, mut values) in other.headers.into_iter_raw() {
            self.headers.add_all(name, &mut values);
        }
    }
}

impl<'r> fmt::Debug for Response<'r> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}", self.status())?;

        for header in self.headers() {
            writeln!(f, "{}", header)?;
        }

        match self.body {
            Some(ref body) => writeln!(f, "{:?}", body),
            None => writeln!(f, "Empty Body")
        }
    }
}

impl<'r> Responder<'r> for Response<'r> {
    /// This is the identity implementation. It simply returns `Ok(self)`.
    fn respond(self) -> Result<Response<'r>, Status> {
        Ok(self)
    }
}
