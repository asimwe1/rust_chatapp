use std::io::Read;
use std::cell::RefCell;
use std::fmt;

use term_painter::Color::*;
use term_painter::ToStyle;

use error::Error;
use super::{FromParam, FromSegments};
use method::Method;

use content_type::ContentType;
use hyper::uri::RequestUri as HyperRequestUri;
use hyper::header;
use router::URIBuf;
use router::URI;
use router::Route;

// Hyper stuff.
use request::{Cookies, HyperCookie, HyperHeaders, HyperRequest};

/// The type for all incoming web requests.
///
/// This should be used sparingly in Rocket applications. In particular, it
/// should likely only be used when writing
/// [FromRequest](trait.FromRequest.html) implementations. It contains all of
/// the information for a given web request. This includes the HTTP method, URI,
/// cookies, headers, and more.
pub struct Request<'a> {
    /// The HTTP method associated with the request.
    pub method: Method,
    /// The URI associated with the request.
    pub uri: URIBuf, // FIXME: Should be URI (without Hyper).
    /// <div class="stability" style="margin-left: 0;">
    ///   <em class="stab unstable">
	///     Unstable
    ///     (<a href="https://github.com/SergioBenitez/Rocket/issues/17">#17</a>):
    ///     The underlying HTTP library/types are likely to change before v1.0.
    ///   </em>
    /// </div>
    ///
    /// The data in the request.
    pub data: Vec<u8>, // FIXME: Don't read this! (bad Hyper.)
    cookies: Cookies,
    headers: HyperHeaders, // This sucks.
    params: RefCell<Option<Vec<&'a str>>>, // This also sucks.
}

impl<'a> Request<'a> {
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
    pub fn get_param<T: FromParam<'a>>(&self, n: usize) -> Result<T, Error> {
        let params = self.params.borrow();
        if params.is_none() || n >= params.as_ref().unwrap().len() {
            debug!("{} is >= param count {}", n, params.as_ref().unwrap().len());
            Err(Error::NoKey)
        } else {
            T::from_param(params.as_ref().unwrap()[n]).map_err(|_| Error::BadParse)
        }
    }

    /// Returns a borrow to the cookies sent with this request. Note that
    /// `Cookie` implements internal mutability, so this method allows you to
    /// get _and_ set cookies in the given Request.
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
    pub fn get_segments<'r: 'a, T: FromSegments<'a>>(&'r self,
                                                     i: usize)
                                                     -> Result<T, Error> {
        if i >= self.uri().segment_count() {
            debug!("{} is >= segment count {}", i, self.uri().segment_count());
            Err(Error::NoKey)
        } else {
            // TODO: Really want to do self.uri.segments().skip(i).into_inner(),
            // but the std lib doesn't implement it for Skip.
            let mut segments = self.uri.segments();
            for _ in segments.by_ref().take(i) { /* do nothing */ }

            T::from_segments(segments).map_err(|_| Error::BadParse)
        }
    }

    // FIXME: Implement a testing framework for Rocket.
    #[doc(hidden)]
    pub fn mock(method: Method, uri: &str) -> Request {
        Request {
            params: RefCell::new(None),
            method: method,
            cookies: Cookies::new(&[]),
            uri: URIBuf::from(uri),
            data: vec![],
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
    pub fn content_type(&self) -> ContentType {
        let hyp_ct = self.headers().get::<header::ContentType>();
        hyp_ct.map_or(ContentType::any(), |ct| ContentType::from(&ct.0))
    }

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
    pub fn uri(&'a self) -> URI<'a> {
        self.uri.as_uri()
    }

    // FIXME: Don't need a refcell for this.
    #[doc(hidden)]
    pub fn set_params(&'a self, route: &Route) {
        *self.params.borrow_mut() = Some(route.get_params(self.uri.as_uri()))
    }

    #[doc(hidden)]
    #[cfg(test)]
    pub fn set_content_type(&mut self, ct: ContentType) {
        let hyper_ct = header::ContentType(ct.into());
        self.headers.set::<header::ContentType>(hyper_ct)
    }

    /// Create a Rocket Request from a Hyper Request.
    #[doc(hidden)]
    pub fn from_hyp<'h, 'k>(hyper_req: HyperRequest<'h, 'k>)
                            -> Result<Request<'a>, String> {
        let (_, h_method, h_headers, h_uri, _, mut h_body) = hyper_req.deconstruct();

        let uri = match h_uri {
            HyperRequestUri::AbsolutePath(s) => URIBuf::from(s),
            _ => return Err(format!("Bad URI: {}", h_uri)),
        };

        let method = match Method::from_hyp(&h_method) {
            Some(m) => m,
            _ => return Err(format!("Bad method: {}", h_method)),
        };

        let cookies = match h_headers.get::<HyperCookie>() {
            // TODO: What to do about key?
            Some(cookie) => cookie.to_cookie_jar(&[]),
            None => Cookies::new(&[]),
        };

        // FIXME: GRRR.
        let mut data = vec![];
        h_body.read_to_end(&mut data).unwrap();

        let request = Request {
            params: RefCell::new(None),
            method: method,
            cookies: cookies,
            uri: uri,
            data: data,
            headers: h_headers,
        };

        Ok(request)
    }
}

impl<'r> fmt::Display for Request<'r> {
    /// Pretty prints a Request. This is primarily used by Rocket's logging
    /// infrastructure.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", Green.paint(&self.method), Blue.paint(&self.uri))
    }
}
