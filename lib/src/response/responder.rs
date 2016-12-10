use std::fs::File;
use std::fmt;

use http::mime::{Mime, TopLevel, SubLevel};
use http::hyper::{header, FreshHyperResponse, StatusCode};
use outcome::{self, IntoOutcome};
use outcome::Outcome::*;
use response::Stream;


/// Type alias for the `Outcome` of a `Responder`.
pub type Outcome<'a> = outcome::Outcome<(), (), (StatusCode, FreshHyperResponse<'a>)>;

impl<'a, T, E> IntoOutcome<(), (), (StatusCode, FreshHyperResponse<'a>)> for Result<T, E> {
    fn into_outcome(self) -> Outcome<'a> {
        match self {
            Ok(_) => Success(()),
            Err(_) => Failure(())
        }
    }
}

/// Trait implemented by types that send a response to clients.
///
/// Types that implement this trait can be used as the return type of a handler,
/// as illustrated below:
///
/// ```rust,ignore
/// #[get("/")]
/// fn index() -> T { ... }
/// ```
///
/// In this example, `T` can be any type that implements `Responder`.
///
/// # Outcomes
///
/// The returned [Outcome](/rocket/outcome/index.html) of a `respond` call
/// determines how the response will be processed, if at all.
///
/// * **Success**
///
///   An `Outcome` of `Success` indicates that the responder was successful in
///   sending the response to the client. No further processing will occur as a
///   result.
///
/// * **Failure**
///
///   An `Outcome` of `Failure` indicates that the responder failed after
///   beginning a response. The response is incomplete, and there is no way to
///   salvage the response. No further processing will occur.
///
/// * **Forward**(StatusCode, FreshHyperResponse<'a>)
///
///   If the `Outcome` is `Forward`, the response will be forwarded to the
///   designated error [Catcher](/rocket/struct.Catcher.html) for the given
///   `StatusCode`. This requires that a response wasn't started and thus is
///   still fresh.
///
/// # Provided Implementations
///
/// Rocket implements `Responder` for several standard library types. Their
/// behavior is documented here. Note that the `Result` implementation is
/// overloaded, allowing for two `Responder`s to be used at once, depending on
/// the variant.
///
///   * **impl<'a> Responder for &'a str**
///
///     Sets the `Content-Type`t to `text/plain` if it is not already set. Sends
///     the string as the body of the response.
///
///   * **impl Responder for String**
///
///     Sets the `Content-Type`t to `text/html` if it is not already set. Sends
///     the string as the body of the response.
///
///   * **impl Responder for File**
///
///     Streams the `File` to the client. This is essentially an alias to
///     Stream<File>.
///
///   * **impl Responder for ()**
///
///     Responds with an empty body.
///
///   * **impl<T: Responder> Responder for Option<T>**
///
///     If the `Option` is `Some`, the wrapped responder is used to respond to
///     respond to the client. Otherwise, the response is forwarded to the 404
///     error catcher and a warning is printed to the console.
///
///   * **impl<T: Responder, E: Debug> Responder for Result<T, E>**
///
///     If the `Result` is `Ok`, the wrapped responder is used to respond to the
///     client. Otherwise, the response is forwarded to the 500 error catcher
///     and the error is printed to the console using the `Debug`
///     implementation.
///
///   * **impl<T: Responder, E: Responder + Debug> Responder for Result<T, E>**
///
///     If the `Result` is `Ok`, the wrapped `Ok` responder is used to respond
///     to the client. If the `Result` is `Err`, the wrapped error responder is
///     used to respond to the client.
///
/// # Implementation Tips
///
/// This section describes a few best practices to take into account when
/// implementing `Responder`.
///
/// ## Debug
///
/// A type implementing `Responder` should implement the `Debug` trait when
/// possible. This is because the `Responder` implementation for `Result`
/// requires its `Err` type to implement `Debug`. Therefore, a type implementing
/// `Debug` can more easily be composed.
///
/// ## Check Before Changing
///
/// Unless a given type is explicitly designed to change some information in the
/// response, it should first _check_ that some information hasn't been set
/// before _changing_ that information. For example, before setting the
/// `Content-Type` header of a response, first check that the header hasn't been
/// set.
///
/// # Example
///
/// Say that you have a custom type, `Person`:
///
/// ```rust
/// struct Person {
///     name: String,
///     age: u16
/// }
/// ```
///
/// You'd like to use `Person` as a `Responder` so that you can return a
/// `Person` directly from a handler:
///
/// ```rust,ignore
/// #[get("/person/<id>")]
/// fn person(id: usize) -> Option<Person> {
///     Person::from_id(id)
/// }
/// ```
///
/// You want the `Person` responder to set two header fields: `X-Person-Name`
/// and `X-Person-Age` as well as supply a custom representation of the object
/// (`Content-Type: application/x-person`) in the body of the response. The
/// following `Responder` implementation accomplishes this:
///
/// ```rust
/// # #[derive(Debug)]
/// # struct Person { name: String, age: u16 }
/// #
/// use std::str::FromStr;
/// use std::fmt::Write;
///
/// use rocket::response::{Responder, Outcome};
/// use rocket::outcome::IntoOutcome;
/// use rocket::http::hyper::{FreshHyperResponse, header};
/// use rocket::http::ContentType;
///
/// impl Responder for Person {
///     fn respond<'b>(&mut self, mut res: FreshHyperResponse<'b>) -> Outcome<'b> {
///         // Set the custom headers.
///         let name_bytes = self.name.clone().into_bytes();
///         let age_bytes = self.age.to_string().into_bytes();
///         res.headers_mut().set_raw("X-Person-Name", vec![name_bytes]);
///         res.headers_mut().set_raw("X-Person-Age", vec![age_bytes]);
///
///         // Set the custom Content-Type header.
///         let ct = ContentType::from_str("application/x-person").unwrap();
///         res.headers_mut().set(header::ContentType(ct.into()));
///
///         // Write out the "custom" body, here just the debug representation.
///         let mut repr = String::with_capacity(50);
///         write!(&mut repr, "{:?}", *self);
///         res.send(repr.as_bytes()).into_outcome()
///     }
/// }
/// ```
pub trait Responder {
    /// Attempts to write a response to `res`.
    ///
    /// If writing the response successfully completes, an outcome of `Success`
    /// is returned. If writing the response begins but fails, an outcome of
    /// `Failure` is returned. If writing a response fails before writing
    /// anything out, an outcome of `Forward` can be returned, which causes the
    /// response to be written by the appropriate error catcher instead.
    fn respond<'a>(&mut self, res: FreshHyperResponse<'a>) -> Outcome<'a>;
}

/// Sets the `Content-Type`t to `text/plain` if it is not already set. Sends the
/// string as the body of the response.
impl<'a> Responder for &'a str {
    fn respond<'b>(&mut self, mut res: FreshHyperResponse<'b>) -> Outcome<'b> {
        if res.headers().get::<header::ContentType>().is_none() {
            let mime = Mime(TopLevel::Text, SubLevel::Plain, vec![]);
            res.headers_mut().set(header::ContentType(mime));
        }

        res.send(self.as_bytes()).into_outcome()
    }
}

impl Responder for String {
    fn respond<'a>(&mut self, mut res: FreshHyperResponse<'a>) -> Outcome<'a> {
        if res.headers().get::<header::ContentType>().is_none() {
            let mime = Mime(TopLevel::Text, SubLevel::Html, vec![]);
            res.headers_mut().set(header::ContentType(mime));
        }

        res.send(self.as_bytes()).into_outcome()
    }
}

/// Essentially aliases Stream<File>.
impl Responder for File {
    fn respond<'a>(&mut self, res: FreshHyperResponse<'a>) -> Outcome<'a> {
        Stream::from(self).respond(res)
    }
}

/// Empty response.
impl Responder for () {
    fn respond<'a>(&mut self, res: FreshHyperResponse<'a>) -> Outcome<'a> {
        res.send(&[]).into_outcome()
    }
}

impl<T: Responder> Responder for Option<T> {
    fn respond<'a>(&mut self, res: FreshHyperResponse<'a>) -> Outcome<'a> {
        if let Some(ref mut val) = *self {
            val.respond(res)
        } else {
            warn_!("Response was `None`.");
            Forward((StatusCode::NotFound, res))
        }
    }
}

impl<T: Responder, E: fmt::Debug> Responder for Result<T, E> {
    // prepend with `default` when using impl specialization
    default fn respond<'a>(&mut self, res: FreshHyperResponse<'a>) -> Outcome<'a> {
        match *self {
            Ok(ref mut val) => val.respond(res),
            Err(ref e) => {
                error_!("{:?}", e);
                Forward((StatusCode::InternalServerError, res))
            }
        }
    }
}

impl<T: Responder, E: Responder + fmt::Debug> Responder for Result<T, E> {
    fn respond<'a>(&mut self, res: FreshHyperResponse<'a>) -> Outcome<'a> {
        match *self {
            Ok(ref mut responder) => responder.respond(res),
            Err(ref mut responder) => responder.respond(res),
        }
    }
}
