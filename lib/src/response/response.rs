use std::{io, fmt, str};
use std::borrow::Cow;

use http::{Header, HeaderMap};
use response::Responder;
use http::Status;

/// The default size, in bytes, of a chunk for streamed responses.
pub const DEFAULT_CHUNK_SIZE: u64 = 4096;

#[derive(PartialEq, Clone, Hash)]
/// The body of a response: can be sized or streamed/chunked.
pub enum Body<T> {
    /// A fixed-size body.
    Sized(T, u64),
    /// A streamed/chunked body, akin to `Transfer-Encoding: chunked`.
    Chunked(T, u64)
}

impl<T> Body<T> {
    /// Returns a new `Body` with a mutable borrow to `self`'s inner type.
    pub fn as_mut(&mut self) -> Body<&mut T> {
        match *self {
            Body::Sized(ref mut b, n) => Body::Sized(b, n),
            Body::Chunked(ref mut b, n) => Body::Chunked(b, n)
        }
    }

    /// Consumes `self`. Passes the inner type as a parameter to `f` and
    /// constructs a new body with the size of `self` and the return value of
    /// the call to `f`.
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Body<U> {
        match self {
            Body::Sized(b, n) => Body::Sized(f(b), n),
            Body::Chunked(b, n) => Body::Chunked(f(b), n)
        }
    }
}

impl<T: io::Read> Body<T> {
    /// Attepts to read `self` into a `String` and returns it. If reading or
    /// conversion fails, returns `None`.
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

/// Type for easily building `Response`s.
///
/// Building a [Response](struct.Response.html) can be a low-level ordeal; this
/// structure presents a higher-level API that simplified building `Response`s.
///
/// # Usage
///
/// `ResponseBuilder` follows the builder pattern and is usually obtained by
/// calling [build](struct.Response.html#method.build) on `Response`. Almost all
/// methods take the current builder as a mutable reference and return the same
/// mutable reference with field(s) modified in the `Responder` being built.
/// These method calls can be chained: `build.a().b()`.
///
/// To finish building and retrieve the built `Response`, use the
/// [finalize](#method.finalize) or [ok](#method.ok) methods.
///
/// ## Headers
///
/// When building a `Response`, headers can either be _replaced_ or _adjoined_;
/// the default behavior (using `header(..)`) is to _replace_. When a header is
/// _replaced_, any existing values for headers with the same name are removed,
/// and the new value is set. If no header exists, the header is simply added.
/// On the other hand, when a header is `adjoined`, all existing values will
/// remain, and the `value` of the adjoined header will be added to the set of
/// existing values, if any. Adjoining maintains order: headers adjoined first
/// will appear first in the `Response`.
///
/// ## Joining and Merging
///
/// It is often necessary to combine multiple `Response`s in some way. The
/// [merge](#method.merge) and [join](#method.join) methods facilitate this. The
/// `merge` method replaces all of the fields in `self` with those present in
/// `other`. The `join` method sets any fields not set in `self` to the value in
/// `other`. See their documentation for more details.
/// ## Example
///
/// The following example builds a `Response` with:
///
///   * **Status**: `418 I'm a teapot`
///   * **Content-Type** header: `text/plain; charset=utf-8`
///   * **X-Teapot-Make** header: `Rocket`
///   * **X-Teapot-Model** headers: `Utopia`, `Series 1`
///   * **Body**: fixed-size string `"Brewing the best coffee!"`
///
/// ```rust
/// use std::io::Cursor;
/// use rocket::response::Response;
/// use rocket::http::{Status, ContentType};
///
/// let response = Response::build()
///     .status(Status::ImATeapot)
///     .header(ContentType::Plain)
///     .raw_header("X-Teapot-Make", "Rocket")
///     .raw_header("X-Teapot-Model", "Utopia")
///     .raw_header_adjoin("X-Teapot-Model", "Series 1")
///     .sized_body(Cursor::new("Brewing the best coffee!"))
///     .finalize();
/// ```
///
pub struct ResponseBuilder<'r> {
    response: Response<'r>
}

impl<'r> ResponseBuilder<'r> {
    /// Creates a new `ResponseBuilder` that will build on top of the `base`
    /// `Response`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket::response::{ResponseBuilder, Response};
    ///
    /// let builder = ResponseBuilder::new(Response::new());
    /// ```
    #[inline(always)]
    pub fn new(base: Response<'r>) -> ResponseBuilder<'r> {
        ResponseBuilder {
            response: base
        }
    }

    /// Sets the status of the `Response` being built to `status`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket::Response;
    /// use rocket::http::Status;
    ///
    /// let response = Response::build()
    ///     .status(Status::NotFound)
    ///     .finalize();
    /// ```
    #[inline(always)]
    pub fn status(&mut self, status: Status) -> &mut ResponseBuilder<'r> {
        self.response.set_status(status);
        self
    }

    /// Sets the status of the `Response` being built to a custom status
    /// constructed from the `code` and `reason` phrase.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket::Response;
    ///
    /// let response = Response::build()
    ///     .raw_status(699, "Alien Encounter")
    ///     .finalize();
    /// ```
    #[inline(always)]
    pub fn raw_status(&mut self, code: u16, reason: &'static str)
            -> &mut ResponseBuilder<'r> {
        self.response.set_raw_status(code, reason);
        self
    }

    /// Adds `header` to the `Response`, replacing any header with the same name
    /// that already exists in the response. If multiple headers with
    /// the same name exist, they are all removed, and only the new header and
    /// value will remain.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket::Response;
    /// use rocket::http::ContentType;
    ///
    /// let response = Response::build()
    ///     .header(ContentType::JSON)
    ///     .header(ContentType::HTML)
    ///     .finalize();
    ///
    /// assert_eq!(response.header_values("Content-Type").count(), 1);
    /// ```
    #[inline(always)]
    pub fn header<'h: 'r, H>(&mut self, header: H) -> &mut ResponseBuilder<'r>
        where H: Into<Header<'h>>
    {
        self.response.set_header(header);
        self
    }

    /// Adds `header` to the `Response` by adjoining the header with any
    /// existing headers with the same name that already exist in the
    /// `Response`. This allow for multiple headers with the same name and
    /// potentially different values to be present in the `Response`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket::Response;
    /// use rocket::http::hyper::header::Accept;
    ///
    /// let response = Response::build()
    ///     .header_adjoin(Accept::json())
    ///     .header_adjoin(Accept::text())
    ///     .finalize();
    ///
    /// assert_eq!(response.header_values("Accept").count(), 2);
    /// ```
    #[inline(always)]
    pub fn header_adjoin<'h: 'r, H>(&mut self, header: H) -> &mut ResponseBuilder<'r>
        where H: Into<Header<'h>>
    {
        self.response.adjoin_header(header);
        self
    }

    /// Adds custom a header to the `Response` with the given name and value,
    /// replacing any header with the same name that already exists in the
    /// response. If multiple headers with the same name exist, they are all
    /// removed, and only the new header and value will remain.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket::Response;
    /// use rocket::http::ContentType;
    ///
    /// let response = Response::build()
    ///     .raw_header("X-Custom", "first")
    ///     .raw_header("X-Custom", "second")
    ///     .finalize();
    ///
    /// assert_eq!(response.header_values("X-Custom").count(), 1);
    /// ```
    #[inline(always)]
    pub fn raw_header<'a: 'r, 'b: 'r, N, V>(&mut self, name: N, value: V)
            -> &mut ResponseBuilder<'r>
        where N: Into<Cow<'a, str>>, V: Into<Cow<'b, str>>
    {
        self.response.set_raw_header(name, value);
        self
    }

    /// Adds custom header to the `Response` with the given name and value,
    /// adjoining the header with any existing headers with the same name that
    /// already exist in the `Response`. This allow for multiple headers with
    /// the same name and potentially different values to be present in the
    /// `Response`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket::Response;
    ///
    /// let response = Response::build()
    ///     .raw_header_adjoin("X-Custom", "first")
    ///     .raw_header_adjoin("X-Custom", "second")
    ///     .finalize();
    ///
    /// assert_eq!(response.header_values("X-Custom").count(), 2);
    /// ```
    #[inline(always)]
    pub fn raw_header_adjoin<'a: 'r, 'b: 'r, N, V>(&mut self, name: N, value: V)
            -> &mut ResponseBuilder<'r>
        where N: Into<Cow<'a, str>>, V: Into<Cow<'b, str>>
    {
        self.response.adjoin_raw_header(name, value);
        self
    }

    /// Sets the body of the `Response` to be the fixed-sized `body`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket::Response;
    /// use std::fs::File;
    /// # use std::io;
    ///
    /// # fn test() -> io::Result<()> {
    /// let response = Response::build()
    ///     .sized_body(File::open("body.txt")?)
    ///     .finalize();
    /// # Ok(())
    /// # }
    /// ```
    #[inline(always)]
    pub fn sized_body<B>(&mut self, body: B) -> &mut ResponseBuilder<'r>
        where B: io::Read + io::Seek + 'r
    {
        self.response.set_sized_body(body);
        self
    }

    /// Sets the body of the `Response` to be the streamed `body`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket::Response;
    /// use std::fs::File;
    /// # use std::io;
    ///
    /// # fn test() -> io::Result<()> {
    /// let response = Response::build()
    ///     .streamed_body(File::open("body.txt")?)
    ///     .finalize();
    /// # Ok(())
    /// # }
    /// ```
    #[inline(always)]
    pub fn streamed_body<B>(&mut self, body: B) -> &mut ResponseBuilder<'r>
        where B: io::Read + 'r
    {
        self.response.set_streamed_body(body);
        self
    }

    /// Sets the body of the `Response` to be the streamed `body` with a custom
    /// chunk size, in bytes.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket::Response;
    /// use std::fs::File;
    /// # use std::io;
    ///
    /// # fn test() -> io::Result<()> {
    /// let response = Response::build()
    ///     .chunked_body(File::open("body.txt")?, 8096)
    ///     .finalize();
    /// # Ok(())
    /// # }
    /// ```
    #[inline(always)]
    pub fn chunked_body<B: io::Read + 'r>(&mut self, body: B, chunk_size: u64)
            -> &mut ResponseBuilder<'r>
    {
        self.response.set_chunked_body(body, chunk_size);
        self
    }

    /// Merges the `other` `Response` into `self` by setting any fields in
    /// `self` to the corresponding value in `other` if they are set in `other`.
    /// Fields in `self` are unchanged if they are not set in `other`. If a
    /// header is set in both `self` and `other`, the values in `other` are
    /// kept. Headers set only in `self` remain.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket::Response;
    /// use rocket::http::{Status, ContentType};
    ///
    /// let base = Response::build()
    ///     .status(Status::NotFound)
    ///     .header(ContentType::HTML)
    ///     .raw_header("X-Custom", "value 1")
    ///     .finalize();
    ///
    /// let response = Response::build()
    ///     .status(Status::ImATeapot)
    ///     .raw_header("X-Custom", "value 2")
    ///     .raw_header_adjoin("X-Custom", "value 3")
    ///     .merge(base)
    ///     .finalize();
    ///
    /// assert_eq!(response.status(), Status::NotFound);
    ///
    /// # {
    /// let ctype: Vec<_> = response.header_values("Content-Type").collect();
    /// assert_eq!(ctype, vec![ContentType::HTML.to_string()]);
    /// # }
    ///
    /// # {
    /// let custom_values: Vec<_> = response.header_values("X-Custom").collect();
    /// assert_eq!(custom_values, vec!["value 1"]);
    /// # }
    /// ```
    #[inline(always)]
    pub fn merge(&mut self, other: Response<'r>) -> &mut ResponseBuilder<'r> {
        self.response.merge(other);
        self
    }

    /// Joins the `other` `Response` into `self` by setting any fields in `self`
    /// to the corresponding value in `other` if they are set in `self`. Fields
    /// in `self` are unchanged if they are already set. If a header is set in
    /// both `self` and `other`, the values are adjoined, with the values in
    /// `self` coming first. Headers only in `self` or `other` are set in
    /// `self`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket::Response;
    /// use rocket::http::{Status, ContentType};
    ///
    /// let other = Response::build()
    ///     .status(Status::NotFound)
    ///     .header(ContentType::HTML)
    ///     .raw_header("X-Custom", "value 1")
    ///     .finalize();
    ///
    /// let response = Response::build()
    ///     .status(Status::ImATeapot)
    ///     .raw_header("X-Custom", "value 2")
    ///     .raw_header_adjoin("X-Custom", "value 3")
    ///     .join(other)
    ///     .finalize();
    ///
    /// assert_eq!(response.status(), Status::ImATeapot);
    ///
    /// # {
    /// let ctype: Vec<_> = response.header_values("Content-Type").collect();
    /// assert_eq!(ctype, vec![ContentType::HTML.to_string()]);
    /// # }
    ///
    /// # {
    /// let custom_values: Vec<_> = response.header_values("X-Custom").collect();
    /// assert_eq!(custom_values, vec!["value 2", "value 3", "value 1"]);
    /// # }
    /// ```
    #[inline(always)]
    pub fn join(&mut self, other: Response<'r>) -> &mut ResponseBuilder<'r> {
        self.response.join(other);
        self
    }

    /// Retrieve the built `Response`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket::Response;
    ///
    /// let response = Response::build()
    ///     // build the response
    ///     .finalize();
    /// ```
    #[inline(always)]
    pub fn finalize(&mut self) -> Response<'r> {
        ::std::mem::replace(&mut self.response, Response::new())
    }

    /// Retrieve the built `Response` wrapped in `Ok`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket::Response;
    ///
    /// let response: Result<Response, ()> = Response::build()
    ///     // build the response
    ///     .ok();
    ///
    /// assert!(response.is_ok());
    /// ```
    #[inline(always)]
    pub fn ok<T>(&mut self) -> Result<Response<'r>, T> {
        Ok(self.finalize())
    }
}

// `join`? Maybe one does one thing, the other does another? IE: `merge`
// replaces, `join` adds. One more thing that could be done: we could make it
// some that _some_ headers default to replacing, and other to joining.
/// An HTTP/Rocket response, returned by `Responder`s.
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
    pub fn header_values<'h>(&'h self, name: &str) -> impl Iterator<Item=&'h str> {
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
