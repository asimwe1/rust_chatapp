use std::cell::RefCell;
use std::fmt;

use term_painter::Color::*;
use term_painter::ToStyle;

use error::Error;
use super::{FromParam, FromSegments};

use router::Route;
use http::uri::{URI, URIBuf};
use http::hyper::{header, HyperCookie, HyperHeaders, HyperMethod, HyperRequestUri};
use http::{Method, ContentType, Cookies};

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
    headers: HyperHeaders, // Don't use hyper's headers.
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
    #[inline(always)]
    pub fn get_param<'r, T: FromParam<'r>>(&'r self, n: usize) -> Result<T, Error> {
        let params = self.params.borrow();
        if n >= params.len() {
            debug!("{} is >= param count {}", n, params.len());
            Err(Error::NoKey)
        } else {
            T::from_param(params[n]).map_err(|_| Error::BadParse)
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
    pub fn get_segments<'r, T: FromSegments<'r>>(&'r self, i: usize) -> Result<T, Error> {
        if i >= self.uri().segment_count() {
            debug!("{} is >= segment count {}", i, self.uri().segment_count());
            Err(Error::NoKey)
        } else {
            // TODO: Really want to do self.uri.segments().skip(i).into_inner(),
            // but the std lib doesn't implement it for Skip.
            let mut segments = self.uri.as_uri().segments();
            for _ in segments.by_ref().take(i) { /* do nothing */ }

            T::from_segments(segments).map_err(|_| Error::BadParse)
        }
    }

    // FIXME: Implement a testing framework for Rocket.
    #[doc(hidden)]
    pub fn mock(method: Method, uri: &str) -> Request {
        Request {
            params: RefCell::new(vec![]),
            method: method,
            cookies: Cookies::new(&[]),
            uri: URIBuf::from(uri),
            headers: HyperHeaders::new(),
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
    pub fn headers(&self) -> &HyperHeaders {
        // FIXME: Get rid of Hyper.
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
        let hyp_ct = self.headers().get::<header::ContentType>();
        hyp_ct.map_or(ContentType::any(), |ct| ContentType::from(&ct.0))
    }

    /// <div class="stability" style="margin-left: 0;">
    ///   <em class="stab unstable">
	///     Unstable
    ///     (<a href="https://github.com/SergioBenitez/Rocket/issues/17">#17</a>):
    ///     The underlying HTTP library/types are likely to change before v1.0.
    ///   </em>
    /// </div>
    ///
    /// Returns the first content-type accepted by this request.
    pub fn accepts(&self) -> ContentType {
        let accept = self.headers().get::<header::Accept>();
        accept.map_or(ContentType::any(), |accept| {
            let items = &accept.0;
            if items.len() < 1 {
                return ContentType::any();
            } else {
                return ContentType::from(items[0].item.clone());
            }
        })
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
        // TODO: Find a way to enforce these. Look at OwningRef for inspiration.
        use ::std::mem::transmute;
        *self.params.borrow_mut() = unsafe {
            transmute(route.get_params(self.uri.as_uri()))
        };
    }

    #[doc(hidden)]
    #[inline(always)]
    pub fn set_headers(&mut self, h_headers: HyperHeaders) {
        let cookies = match h_headers.get::<HyperCookie>() {
            // TODO: Retrieve key from config.
            Some(cookie) => cookie.to_cookie_jar(&[]),
            None => Cookies::new(&[]),
        };

        self.headers = h_headers;
        self.cookies = cookies;
    }

    #[doc(hidden)]
    pub fn new(h_method: HyperMethod,
               h_headers: HyperHeaders,
               h_uri: HyperRequestUri)
               -> Result<Request, String> {
        let uri = match h_uri {
            HyperRequestUri::AbsolutePath(s) => URIBuf::from(s),
            _ => return Err(format!("Bad URI: {}", h_uri)),
        };

        let method = match Method::from_hyp(&h_method) {
            Some(method) => method,
            _ => return Err(format!("Bad method: {}", h_method)),
        };

        let cookies = match h_headers.get::<HyperCookie>() {
            // TODO: Retrieve key from config.
            Some(cookie) => cookie.to_cookie_jar(&[]),
            None => Cookies::new(&[]),
        };

        let request = Request {
            params: RefCell::new(vec![]),
            method: method,
            cookies: cookies,
            uri: uri,
            headers: h_headers,
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
