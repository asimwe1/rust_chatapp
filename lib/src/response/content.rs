//! Contains types that set the Content-Type of a response.
//!
//! # Usage
//!
//! Each type wraps a given responder. The `Responder` implementation of each
//! type simply sets the Content-Type and delegates the remainder of the
//! response to the wrapped responder. This is useful for setting the
//! Content-Type of a type that doesn't set it itself or for overriding one that
//! does.
//!
//! # Example
//!
//! The following snippet creates an `HTML` content response for a string.
//! Normally, raw strings set their response Content-Type to `text/plain`. By
//! using the `HTML` content response, the Content-Type will be set to
//! `text/html` instead.
//!
//! ```rust
//! use rocket::response::content;
//!
//! let response = content::HTML("<h1>Hello, world!</h1>");
//! ```

use response::{Response, Responder};
use http::{Status, ContentType};

/// Set the Content-Type to any arbitrary value.
///
/// Delagates the remainder of the response to the wrapped responder.
///
/// # Example
///
/// Set the Content-Type of a string to be PDF.
///
/// ```rust
/// use rocket::response::content::Content;
/// use rocket::http::ContentType;
///
/// let response = Content(ContentType::from_extension("pdf"), "Hi.");
/// ```
#[derive(Debug)]
pub struct Content<R>(pub ContentType, pub R);

/// Sets the Content-Type of the response to the wrapped `ContentType` then
/// delegates the remainder of the response to the wrapped responder.
impl<'r, R: Responder<'r>> Responder<'r> for Content<R> {
    #[inline(always)]
    fn respond(self) -> Result<Response<'r>, Status> {
        Response::build()
            .merge(self.1.respond()?)
            .header(self.0)
            .ok()
    }
}

macro_rules! ctrs {
    ($($name:ident: $name_str:expr, $ct_str:expr),+) => {
        $(
            #[doc="Set the `Content-Type` of the response to <b>"]
            #[doc=$name_str]
            #[doc="</b>, or <i>"]
            #[doc=$ct_str]
            #[doc="</i>."]
            ///
            /// Delagates the remainder of the response to the wrapped responder.
            #[derive(Debug)]
            pub struct $name<R>(pub R);

            /// Sets the Content-Type of the response then delegates the
            /// remainder of the response to the wrapped responder.
            impl<'r, R: Responder<'r>> Responder<'r> for $name<R> {
                fn respond(self) -> Result<Response<'r>, Status> {
                    Content(ContentType::$name, self.0).respond()
                }
            }
        )+
    }
}

ctrs! {
    JSON: "JSON", "application/json",
    XML: "XML", "text/xml",
    HTML: "HTML", "text/html",
    Plain: "plain text", "text/plain",
    CSS: "CSS", "text/css",
    JavaScript: "JavaScript", "application/javascript"
}

