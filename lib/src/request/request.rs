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

/// The type of an incoming web request.
///
/// This should be used sparingly in Rocket applications. In particular, it
/// should likely only be used when writing
/// [FromRequest](trait.FromRequest.html) implementations. It contains all of
/// the information for a given web request except for the body data. This
/// includes the HTTP method, URI, cookies, headers, and more.
pub struct Request {
    /// The HTTP method associated with the request.
    pub method: Method,
    uri: URIBuf, // FIXME: Should be URI (without hyper).
    params: RefCell<Vec<&'static str>>,
    cookies: Cookies,
    // TODO: Allow non-static here.
    headers: HeaderMap<'static>,
}

impl Request {
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
    pub fn get_param<'r, T: FromParam<'r>>(&'r self, n: usize) -> Result<T, Error> {
        let param = self.get_param_str(n).ok_or(Error::NoKey)?;
        T::from_param(param).map_err(|_| Error::BadParse)
    }

    /// Get the `n`th path parameter, if it exists.
    #[doc(hidden)]
    pub fn get_param_str(&self, n: usize) -> Option<&str> {
        let params = self.params.borrow();
        if n >= params.len() {
            debug!("{} is >= param count {}", n, params.len());
            None
        } else {
            Some(&params[n])
        }
    }

    /// Returns a borrow to the cookies sent with this request. Note that
    /// `Cookie` implements internal mutability, so this method allows you to
    /// get _and_ set cookies in the given Request.
    #[inline(always)]
    pub fn cookies<'r>(&'r self) -> &'r Cookies {
        &self.cookies
    }

    /// Retrieves and parses into `T` all of the path segments in the request
    /// URI beginning and including the 0-indexed `i`. `T` must implement
    /// [FromSegments](trait.FromSegments.html), which is used to parse the
    /// segments.
    ///
    /// For example, if the request URI is `"/hello/there/i/am/here"`, then
    /// `request.get_segments::<T>(1)` will attempt to parse the segments
    /// `"there/i/am/here"` as type `T`.
    pub fn get_segments<'r, T: FromSegments<'r>>(&'r self, i: usize)
            -> Result<T, Error> {
        let segments = self.get_raw_segments(i).ok_or(Error::NoKey)?;
        T::from_segments(segments).map_err(|_| Error::BadParse)
    }

    /// Get the segments beginning at the `i`th, if they exists.
    #[doc(hidden)]
    pub fn get_raw_segments(&self, i: usize) -> Option<Segments> {
        if i >= self.uri().segment_count() {
            debug!("{} is >= segment count {}", i, self.uri().segment_count());
            None
        } else {
            // TODO: Really want to do self.uri.segments().skip(i).into_inner(),
            // but the std lib doesn't implement it for Skip.
            let mut segments = self.uri.as_uri().segments();
            for _ in segments.by_ref().take(i) { /* do nothing */ }
            Some(segments)
        }
    }

    // FIXME: Make this `new`. Make current `new` a `from_hyp` method.
    #[doc(hidden)]
    pub fn mock(method: Method, uri: &str) -> Request {
        Request {
            params: RefCell::new(vec![]),
            method: method,
            cookies: Cookies::new(&[]),
            uri: URIBuf::from(uri),
            headers: HeaderMap::new(),
        }
    }

    /// <div class="stability" style="margin-left: 0;">
    ///   <em class="stab unstable">
	///     Unstable
    ///     (<a href="https://github.com/SergioBenitez/Rocket/issues/17">#17</a>):
    ///     The underlying HTTP library/types are likely to change before v1.0.
    ///   </em>
    /// </div>
    ///
    /// Returns the headers in this request.
    #[inline(always)]
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    /// <div class="stability" style="margin-left: 0;">
    ///   <em class="stab unstable">
	///     Unstable
    ///     (<a href="https://github.com/SergioBenitez/Rocket/issues/17">#17</a>):
    ///     The underlying HTTP library/types are likely to change before v1.0.
    ///   </em>
    /// </div>
    ///
    /// Returns the Content-Type from the request. Althought you can retrieve
    /// the content-type from the headers directly, this method is considered to
    /// be more stable. If no Content-Type was specified in the request, a
    /// Content-Type of [any](struct.ContentType.html#method.any) is returned.
    #[inline(always)]
    pub fn content_type(&self) -> ContentType {
        self.headers().get_one("Content-Type")
            .and_then(|value| value.parse().ok())
            .unwrap_or(ContentType::Any)
    }

    /// Retrieves the URI from the request. Rocket only allows absolute URIs, so
    /// the URI will be absolute.
    #[inline(always)]
    pub fn uri(&self) -> URI {
        self.uri.as_uri()
    }

    #[doc(hidden)]
    #[inline(always)]
    pub fn set_params(&self, route: &Route) {
        // We use transmute to cast the lifetime of self.uri.as_uri() to
        // 'static. This is because that lifetime refers to the String in URIBuf
        // in this structure, which is (obviously) guaranteed to live as long as
        // the structure AS LONG AS it is not moved out or changed. AS A RESULT,
        // the `uri` fields MUST NEVER be changed once it is set.
        //
        // TODO: Find a way to ecapsulate this better. Look at OwningRef/Rental
        // for inspiration.
        use ::std::mem::transmute;
        *self.params.borrow_mut() = unsafe {
            transmute(route.get_params(self.uri.as_uri()))
        };
    }

    #[doc(hidden)]
    #[inline(always)]
    pub fn add_header(&mut self, header: Header<'static>) {
        self.headers.add(header);
    }

    #[doc(hidden)]
    pub fn new(h_method: hyper::Method,
               h_headers: hyper::header::Headers,
               h_uri: hyper::RequestUri)
               -> Result<Request, String> {
        let uri = match h_uri {
            hyper::RequestUri::AbsolutePath(s) => URIBuf::from(s),
            _ => return Err(format!("Bad URI: {}", h_uri)),
        };

        let method = match Method::from_hyp(&h_method) {
            Some(method) => method,
            _ => return Err(format!("Bad method: {}", h_method)),
        };

        let cookies = match h_headers.get::<hyper::header::Cookie>() {
            // TODO: Retrieve key from config.
            Some(cookie) => cookie.to_cookie_jar(&[]),
            None => Cookies::new(&[]),
        };

        let mut headers = HeaderMap::new();
        for h_header in h_headers.iter() {
            headers.add_raw(h_header.name().to_string(), h_header.value_string())
        }

        let request = Request {
            params: RefCell::new(vec![]),
            method: method,
            cookies: cookies,
            uri: uri,
            headers: headers,
        };

        Ok(request)
    }
}

impl fmt::Display for Request {
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
