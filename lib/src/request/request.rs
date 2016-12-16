use std::cell::RefCell;
use std::fmt;

use term_painter::Color::*;
use term_painter::ToStyle;

use error::Error;
use super::{FromParam, FromSegments};

use router::Route;
use http::uri::{URI, URIBuf, Segments};
use http::{Method, ContentType, Header, HeaderMap, Cookies};

use http::hyper;

#[derive(Debug, Clone, PartialEq)]
pub enum CowURI<'b> {
    Borrowed(URI<'b>),
    Owned(URIBuf)
}

impl<'b> CowURI<'b> {
    /// Returns raw URI.
    fn as_str(&self) -> &str {
        match *self {
            CowURI::Borrowed(ref uri) => uri.as_str(),
            CowURI::Owned(ref uri) => uri.as_str()
        }
    }

    fn segment_count(&self) -> usize {
        match *self {
            CowURI::Borrowed(ref uri) => uri.segment_count(),
            CowURI::Owned(ref uri) => uri.segment_count()
        }
    }

    fn segments(&self) -> Segments {
        match *self {
            CowURI::Borrowed(ref uri) => uri.segments(),
            CowURI::Owned(ref uri) => uri.as_uri().segments()
        }
    }
}

impl<'b> From<URI<'b>> for CowURI<'b> {
    fn from(uri: URI<'b>) -> CowURI<'b> {
        CowURI::Borrowed(uri)
    }
}

impl<'b> From<URIBuf> for CowURI<'b> {
    fn from(uri: URIBuf) -> CowURI<'b> {
        CowURI::Owned(uri)
    }
}

impl<'b> fmt::Display for CowURI<'b> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CowURI::Borrowed(ref uri) => uri.fmt(f),
            CowURI::Owned(ref uri) => uri.fmt(f)
        }
    }
}

/// The type of an incoming web request.
///
/// This should be used sparingly in Rocket applications. In particular, it
/// should likely only be used when writing
/// [FromRequest](trait.FromRequest.html) implementations. It contains all of
/// the information for a given web request except for the body data. This
/// includes the HTTP method, URI, cookies, headers, and more.
pub struct Request<'r> {
    method: Method,
    uri: CowURI<'r>,
    headers: HeaderMap<'r>,
    params: RefCell<Vec<(usize, usize)>>,
    cookies: Cookies,
}

impl<'r> Request<'r> {
    pub fn new<U: Into<CowURI<'r>>>(method: Method, uri: U) -> Request<'r> {
        Request {
            method: method,
            uri: uri.into(),
            headers: HeaderMap::new(),
            params: RefCell::new(Vec::new()),
            cookies: Cookies::new(&[]),
        }
    }

    #[inline(always)]
    pub fn method(&self) -> Method {
        self.method
    }

    #[inline(always)]
    pub fn set_method(&mut self, method: Method) {
        self.method = method;
    }

    /// Retrieves the URI from the request. Rocket only allows absolute URIs, so
    /// the URI will be absolute.
    #[inline(always)]
    pub fn uri(&self) -> URI {
        match self.uri {
            CowURI::Borrowed(ref uri) => uri.clone(),
            CowURI::Owned(ref uri) => uri.as_uri(),
        }
    }

    // Sets the URI for the request. To retrieve parameters, the `set_params`
    // method needs to be called first.
    #[inline(always)]
    pub fn set_uri<'u: 'r, U: Into<CowURI<'u>>>(&mut self, uri: U) {
        self.uri = uri.into();
        self.params = RefCell::new(Vec::new());
    }

    // Add the `header` to this request's header list.
    #[inline(always)]
    pub fn add_header(&mut self, header: Header<'r>) {
        self.headers.add(header);
    }

    /// Returns the headers in this request.
    #[inline(always)]
    pub fn headers(&self) -> &HeaderMap<'r> {
        &self.headers
    }

    /// Returns a borrow to the cookies sent with this request. Note that
    /// `Cookie` implements internal mutability, so this method allows you to
    /// get _and_ set cookies in the given Request.
    #[inline(always)]
    pub fn cookies(&self) -> &Cookies {
        &self.cookies
    }

    #[inline(always)]
    pub fn set_cookies(&mut self, cookies: Cookies) {
        self.cookies = cookies;
    }

    /// Returns the Content-Type of the request. Returns `ContentType::Any` if
    /// there was none or if the Content-Type was "*/*".
    #[inline(always)]
    pub fn content_type(&self) -> ContentType {
        self.headers().get_one("Content-Type")
            .and_then(|value| value.parse().ok())
            .unwrap_or(ContentType::Any)
    }

    /// Retrieves and parses into `T` the `n`th dynamic parameter from the
    /// request. Returns `Error::NoKey` if `n` is greater than the number of
    /// params. Returns `Error::BadParse` if the parameter type `T` can't be
    /// parsed from the parameter.
    ///
    /// # Example
    ///
    /// To retrieve parameter `n` as some type `T` that implements
    /// [FromParam](trait.FromParam.html) inside a
    /// [FromRequest](trait.FromRequest.html) implementation:
    ///
    /// ```rust,ignore
    /// fn from_request(request: &'r Request<'c>) -> .. {
    ///     let my_param: T = request.get_param(n);
    /// }
    /// ```
    pub fn get_param<'a, T: FromParam<'a>>(&'a self, n: usize) -> Result<T, Error> {
        let param = self.get_param_str(n).ok_or(Error::NoKey)?;
        T::from_param(param).map_err(|_| Error::BadParse)
    }

    #[doc(hidden)]
    #[inline(always)]
    pub fn set_params(&self, route: &Route) {
        *self.params.borrow_mut() = route.get_param_indexes(self.uri());
    }

    /// Get the `n`th path parameter, if it exists.
    #[doc(hidden)]
    pub fn get_param_str(&self, n: usize) -> Option<&str> {
        let params = self.params.borrow();
        if n >= params.len() {
            debug!("{} is >= param count {}", n, params.len());
            return None;
        }

        let (i, j) = params[n];
        let uri_str = self.uri.as_str();
        if j > uri_str.len() {
            error!("Couldn't retrieve parameter: internal count incorrect.");
            return None;
        }

        Some(&uri_str[i..j])
    }

    /// Retrieves and parses into `T` all of the path segments in the request
    /// URI beginning and including the 0-indexed `i`. `T` must implement
    /// [FromSegments](trait.FromSegments.html), which is used to parse the
    /// segments.
    ///
    /// For example, if the request URI is `"/hello/there/i/am/here"`, then
    /// `request.get_segments::<T>(1)` will attempt to parse the segments
    /// `"there/i/am/here"` as type `T`.
    pub fn get_segments<'a, T: FromSegments<'a>>(&'a self, i: usize)
            -> Result<T, Error> {
        let segments = self.get_raw_segments(i).ok_or(Error::NoKey)?;
        T::from_segments(segments).map_err(|_| Error::BadParse)
    }

    /// Get the segments beginning at the `i`th, if they exists.
    #[doc(hidden)]
    pub fn get_raw_segments(&self, i: usize) -> Option<Segments> {
        if i >= self.uri.segment_count() {
            debug!("{} is >= segment count {}", i, self.uri().segment_count());
            None
        } else {
            // TODO: Really want to do self.uri.segments().skip(i).into_inner(),
            // but the std lib doesn't implement `into_inner` for Skip.
            let mut segments = self.uri.segments();
            for _ in segments.by_ref().take(i) { /* do nothing */ }
            Some(segments)
        }
    }

    #[doc(hidden)]
    pub fn from_hyp(h_method: hyper::Method,
                    h_headers: hyper::header::Headers,
                    h_uri: hyper::RequestUri)
                    -> Result<Request<'static>, String> {
        // Get a copy of the URI for later use.
        let uri = match h_uri {
            hyper::RequestUri::AbsolutePath(s) => URIBuf::from(s),
            _ => return Err(format!("Bad URI: {}", h_uri)),
        };

        // Ensure that the method is known. TODO: Allow made-up methods?
        let method = match Method::from_hyp(&h_method) {
            Some(method) => method,
            None => return Err(format!("Invalid method: {}", h_method))
        };

        // Construct the request object.
        let mut request = Request::new(method, uri);

        // Set the request cookies, if they exist. TODO: Use session key.
        if let Some(cookies) = h_headers.get::<hyper::header::Cookie>() {
            request.set_cookies(cookies.to_cookie_jar(&[]));
        }

        // Set the rest of the headers.
        for hyp in h_headers.iter() {
            let header = Header::new(hyp.name().to_string(), hyp.value_string());
            request.add_header(header);
        }

        Ok(request)
    }
}

impl<'r> fmt::Display for Request<'r> {
    /// Pretty prints a Request. This is primarily used by Rocket's logging
    /// infrastructure.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", Green.paint(&self.method), Blue.paint(&self.uri))?;
        if self.method.supports_payload() && !self.content_type().is_any() {
            write!(f, " {}", Yellow.paint(self.content_type()))?;
        }

        Ok(())
    }
}
