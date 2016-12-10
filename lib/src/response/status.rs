//! Contains types that set the status code and correspoding headers of a
//! response.
//!
//! These types are designed to make it easier to respond with a given status
//! code. Each type takes in the minimum number of parameters required to
//! construct a proper response with that status code. Some types take in
//! responders; when they do, the responder finalizes the response by writing
//! out additional headers and, importantly, the body of the response.

use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

use response::{Responder, Outcome};
use outcome::IntoOutcome;
use http::hyper::{StatusCode, FreshHyperResponse, header};

/// Sets the status of the response to 201 (Created).
///
/// The `String` field is set as the value of the `Location` header in the
/// response. The optional `Responder` field is used to finalize the response.
///
/// # Example
///
/// ```rust
/// use rocket::response::status;
///
/// let url = "http://myservice.com/resource.json".to_string();
/// let content = "{ 'resource': 'Hello, world!' }";
/// let response = status::Created(url, Some(content));
/// ```
pub struct Created<R: Responder>(pub String, pub Option<R>);

/// Sets the status code of the response to 201 Created. Sets the `Location`
/// header to the `String` parameter in the constructor.
///
/// The optional responder finalizes the response if it exists. The wrapped
/// responder should write the body of the response so that it contains
/// information about the created resource. If no responder is provided, the
/// response body will be empty.
impl<R: Responder> Responder for Created<R> {
    default fn respond<'b>(&mut self, mut res: FreshHyperResponse<'b>) -> Outcome<'b> {
        *res.status_mut() = StatusCode::Created;
        res.headers_mut().set(header::Location(self.0.clone()));
        match self.1 {
            Some(ref mut r) => r.respond(res),
            None => res.send(&[]).into_outcome()
        }
    }
}

/// In addition to setting the status code, `Location` header, and finalizing
/// the response with the `Responder`, the `ETag` header is set conditionally if
/// a `Responder` is provided that implements `Hash`. The `ETag` header is set
/// to a hash value of the responder.
impl<R: Responder + Hash> Responder for Created<R> {
    fn respond<'b>(&mut self, mut res: FreshHyperResponse<'b>) -> Outcome<'b> {
        *res.status_mut() = StatusCode::Created;
        res.headers_mut().set(header::Location(self.0.clone()));

        let mut hasher = DefaultHasher::default();
        match self.1 {
            Some(ref mut responder) => {
                responder.hash(&mut hasher);
                let tag = header::EntityTag::strong(hasher.finish().to_string());
                res.headers_mut().set(header::ETag(tag));
                responder.respond(res)
            }
            None => res.send(&[]).into_outcome()
        }
    }
}

/// Sets the status of the response to 202 (Accepted).
///
/// If a responder is supplied, the remainder of the response is delegated to
/// it. If there is no responder, the body of the response will be empty.
///
/// # Examples
///
/// A 202 Accepted response without a body:
///
/// ```rust
/// use rocket::response::status;
///
/// let response = status::Accepted::<()>(None);
/// ```
///
/// A 202 Accepted response _with_ a body:
///
/// ```rust
/// use rocket::response::status;
///
/// let response = status::Accepted(Some("processing"));
/// ```
pub struct Accepted<R: Responder>(pub Option<R>);

/// Sets the status code of the response to 202 Accepted. If the responder is
/// `Some`, it is used to finalize the response.
impl<R: Responder> Responder for Accepted<R> {
    fn respond<'b>(&mut self, mut res: FreshHyperResponse<'b>) -> Outcome<'b> {
        *res.status_mut() = StatusCode::Accepted;
        match self.0 {
            Some(ref mut r) => r.respond(res),
            None => res.send(&[]).into_outcome()
        }
    }
}

/// Sets the status of the response to 204 (No Content).
///
/// # Example
///
/// ```rust
/// use rocket::response::status;
///
/// let response = status::NoContent;
/// ```
// TODO: This would benefit from Header support.
pub struct NoContent;

/// Sets the status code of the response to 204 No Content. The body of the
/// response will be empty.
impl Responder for NoContent {
    fn respond<'b>(&mut self, mut res: FreshHyperResponse<'b>) -> Outcome<'b> {
        *res.status_mut() = StatusCode::NoContent;
        res.send(&[]).into_outcome()
    }
}


/// Sets the status of the response to 205 (Reset Content).
///
/// # Example
///
/// ```rust
/// use rocket::response::status;
///
/// let response = status::Reset;
/// ```
pub struct Reset;

/// Sets the status code of the response to 205 Reset Content. The body of the
/// response will be empty.
impl Responder for Reset {
    fn respond<'b>(&mut self, mut res: FreshHyperResponse<'b>) -> Outcome<'b> {
        *res.status_mut() = StatusCode::ResetContent;
        res.send(&[]).into_outcome()
    }
}

/// Creates a response with the given status code and underyling responder.
///
/// # Example
///
/// ```rust
/// use rocket::response::status;
/// use rocket::http::StatusCode;
///
/// let response = status::Custom(StatusCode::ImATeapot, "Hi!");
/// ```
pub struct Custom<R: Responder>(pub StatusCode, pub R);

/// Sets the status code of the response and then delegates the remainder of the
/// response to the wrapped responder.
impl<R: Responder> Responder for Custom<R> {
    fn respond<'b>(&mut self, mut res: FreshHyperResponse<'b>) -> Outcome<'b> {
        *(res.status_mut()) = self.0;
        self.1.respond(res)
    }
}

// The following are unimplemented.
// 206 Partial Content (variant), 203 Non-Authoritative Information (headers).
