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

use response::{Responder, Outcome};
use http::hyper::{header, FreshHyperResponse};
use http::mime::{Mime, TopLevel, SubLevel};
use http::ContentType;

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
pub struct Content<T: Responder>(pub ContentType, pub T);

/// Sets the Content-Type of the response to the wrapped `ContentType` then
/// delegates the remainder of the response to the wrapped responder.
impl<T: Responder> Responder for Content<T> {
    fn respond<'b>(&mut self, mut res: FreshHyperResponse<'b>) -> Outcome<'b> {
        res.headers_mut().set(header::ContentType(self.0.clone().into()));
        self.1.respond(res)
    }
}

macro_rules! ctrs {
    ($($(#[$attr:meta])* | $name:ident: $top:ident/$sub:ident),+) => {
        $(
            $(#[$attr])*
            ///
            /// Delagates the remainder of the response to the wrapped responder.
            #[derive(Debug)]
            pub struct $name<T: Responder>(pub T);

            /// Sets the Content-Type of the response then delegates the
            /// remainder of the response to the wrapped responder.
            impl<T: Responder> Responder for $name<T> {
                #[inline(always)]
                fn respond<'b>(&mut self, mut res: FreshHyperResponse<'b>) -> Outcome<'b> {
                    let mime = Mime(TopLevel::$top, SubLevel::$sub, vec![]);
                    res.headers_mut().set(header::ContentType(mime));
                    self.0.respond(res)
                }
            })+
    }
}

ctrs! {
    /// Sets the Content-Type of the response to JSON (`application/json`).
    | JSON: Application/Json,

    /// Sets the Content-Type of the response to XML (`text/xml`).
    | XML: Text/Xml,

    /// Sets the Content-Type of the response to HTML (`text/html`).
    | HTML: Text/Html,

    /// Sets the Content-Type of the response to plain text (`text/plain`).
    | Plain: Text/Plain,

    /// Sets the Content-Type of the response to CSS (`text/css`).
    | CSS: Text/Css,

    /// Sets the Content-Type of the response to JavaScript
    /// (`application/javascript`).
    | JavaScript: Application/Javascript
}

