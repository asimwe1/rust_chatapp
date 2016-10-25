//! A tiny module for testing Rocket applications.
//!
//! # Usage
//!
//! The testing methadology is simple:
//!
//!   1. Construct a `Rocket` instance
//!   2. Construct a request.
//!   3. Dispatch the request using the Rocket instance.
//!   4. Inspect, validate, and verify the response.
//!
//! ## Construct a `Rocket` Instance
//!
//! Constructing a `Rocket` instance for testing is identical to constructing
//! one for launching, except you should not call the `launch` method. That is,
//! use `rocket::ignite`, then mount routes and catchers. That's it!
//!
//! ## Construct a (Mock)Request
//!
//! The [MockRequest](struct.MockRequest.html) object enables the creation of an
//! HTTP request without using any networking. A `MockRequest` object is
//! constructed using the builder pattern. For example, the following code
//! builds a request for submitting a login form with three fields:
//!
//! ```rust
//! use rocket::http::Method::*;
//! use rocket::testing::MockRequest;
//!
//! let (username, password, age) = ("user", "password", 32);
//! MockRequest::new(Post, "/login")
//!     .headers(&[("Content-Type", "application/x-www-form-urlencoded")])
//!     .body(&format!("username={}&password={}&age={}", username, password, age));
//! ```
//!
//! ## Dispatch and Validate
//!
//! Finally, requests can be dispatched using the
//! [dispatch_with](struct.MockRequest.html#method.dispatch_with) method on the
//! contructed `MockRequest` instance. The method returns the body of the
//! response. At present, the API does not allow for headers in the response to
//! be examined.

use std::io::Cursor;
use outcome::Outcome::*;
use http::{hyper, Method};
use request::{Request, Data};
use Rocket;

/// A type for mocking requests for testing Rocket applications.
pub struct MockRequest {
    request: Request,
    data: Data
}

impl MockRequest {
    /// Constructs a new mocked request with the given `method` and `uri`.
    pub fn new<S: AsRef<str>>(method: Method, uri: S) -> Self {
        MockRequest {
            request: Request::mock(method, uri.as_ref()),
            data: Data::new(vec![])
        }
    }

    /// Sets the headers for this request.
    ///
    /// # Examples
    ///
    /// Set the Content-Type header:
    ///
    /// ```rust
    /// use rocket::http::Method::*;
    /// use rocket::testing::MockRequest;
    ///
    /// let req = MockRequest::new(Get, "/").headers(&[
    ///     ("Content-Type", "application/json")
    /// ]);
    /// ```
    pub fn headers<'h, H: AsRef<[(&'h str, &'h str)]>>(mut self, headers: H) -> Self {
        let mut hyp_headers = hyper::HyperHeaders::new();

        for &(name, fields) in headers.as_ref() {
            let mut vec_fields = vec![];
            for field in fields.split(";") {
                vec_fields.push(field.as_bytes().to_vec());
            }

            hyp_headers.set_raw(name.to_string(), vec_fields);
        }

        self.request.set_headers(hyp_headers);
        self
    }

    /// Set the body (data) of the request.
    ///
    /// # Examples
    ///
    /// Set the body to be a JSON structure; also sets the Content-Type.
    ///
    /// ```rust
    /// use rocket::http::Method::*;
    /// use rocket::testing::MockRequest;
    ///
    /// let req = MockRequest::new(Post, "/").headers(&[
    ///     ("Content-Type", "application/json")
    /// ]).body(r#"
    ///    {
    ///        "key": "value",
    ///        "array": [1, 2, 3],
    ///    }
    /// "#);
    /// ```
    pub fn body<S: AsRef<str>>(mut self, body: S) -> Self {
        self.data = Data::new(body.as_ref().as_bytes().into());
        self
    }

    /// Dispatch this request using a given instance of Rocket.
    ///
    /// Returns the body of the response if there was a response. The return
    /// value is `None` if any of the following occurs:
    ///
    ///   1. The returned body was not valid UTF8.
    ///   2. The application failed to respond.
    ///
    /// # Examples
    ///
    /// Dispatch to a Rocket instance with the `"Hello, world!"` example
    /// mounted:
    ///
    /// ```rust
    /// # #![feature(plugin)]
    /// # #![plugin(rocket_codegen)]
    /// # extern crate rocket;
    /// #
    /// #[get("/")]
    /// fn hello() -> &'static str {
    ///     "Hello, world!"
    /// }
    ///
    /// use rocket::testing::MockRequest;
    /// use rocket::http::Method::*;
    ///
    /// # fn main() {
    /// let rocket = rocket::ignite().mount("/", routes![hello]);
    /// let req = MockRequest::new(Get, "/");
    /// let result = req.dispatch_with(&rocket);
    /// assert_eq!(result.unwrap().as_str(), "Hello, world!");
    /// # }
    /// ```
    pub fn dispatch_with(mut self, rocket: &Rocket) -> Option<String> {
        let request = self.request;
        let data = ::std::mem::replace(&mut self.data, Data::new(vec![]));

        let mut response = Cursor::new(vec![]);

        // Create a new scope so we can get the inner from response later.
        let ok = {
            let mut h_h = hyper::HyperHeaders::new();
            let res = hyper::FreshHyperResponse::new(&mut response, &mut h_h);
            match rocket.dispatch(&request, data) {
                Ok(mut responder) => {
                    match responder.respond(res) {
                        Success(_) => true,
                        Failure(_) => false,
                        Forward((code, r)) => rocket.handle_error(code, &request, r)
                    }
                }
                Err(code) => rocket.handle_error(code, &request, res)
            }
        };

        if !ok {
            return None;
        }

        match String::from_utf8(response.into_inner()) {
            Ok(string) => {
                // TODO: Expose the full response (with headers) somewhow.
                string.find("\r\n\r\n").map(|i| string[(i + 4)..].to_string())
            }
            Err(e) => {
                error_!("Could not create string from response: {:?}", e);
                None
            }
        }
    }
}
