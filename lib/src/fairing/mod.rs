//! Fairings: structured interposition at launch, request, and response time.
//!
//! Fairings allow for structured interposition at various points in the
//! application lifetime. Fairings can be seen as a restricted form of
//! "middleware". A fairing is an arbitrary structure with methods representing
//! callbacks that Rocket will run at requested points in a program. You can use
//! fairings to rewrite or record information about requests and responses, or
//! to perform an action once a Rocket application has launched.
//!
//! ## Attaching
//!
//! You must inform Rocket about fairings that you wish to be active by calling
//! the [`attach`](/rocket/struct.Rocket.html#method.attach) method on the
//! [`Rocket`](/rocket/struct.Rocket.html) instance and passing in the
//! appropriate [`Fairing`](/rocket/fairing/trait.Fairing.html). For instance,
//! to attach fairings named `req_fairing` and `res_fairing` to a new Rocket
//! instance, you might write:
//!
//! ```rust
//! # use rocket::fairing::AdHoc;
//! # let req_fairing = AdHoc::on_request(|_, _| ());
//! # let res_fairing = AdHoc::on_response(|_, _| ());
//! let rocket = rocket::ignite()
//!     .attach(req_fairing)
//!     .attach(res_fairing);
//! ```
//!
//! Once a fairing is attached, Rocket will execute it at the appropiate time,
//! which varies depending on the fairing type. See the
//! [`Fairing`](/rocket/fairing/trait.Fairing.html) trait documentation for more
//! information on the dispatching of fairing methods.
//!
//! ## Ordering
//!
//! `Fairing`s are executed in the same order in which they are attached: the
//! first attached fairing has its callbacks executed before all others. Because
//! fairing callbacks may not be commutative, it is important to communicate to
//! the user every consequence of a fairing. Furthermore, a `Fairing` should
//! take care to act locally so that the actions of other `Fairings` are not
//! jeopardized.
use {Rocket, Request, Response, Data};

mod fairings;
mod ad_hoc;
mod info_kind;

pub(crate) use self::fairings::Fairings;
pub use self::ad_hoc::AdHoc;
pub use self::info_kind::{Info, Kind};

// We might imagine that a request fairing returns an `Outcome`. If it returns
// `Success`, we don't do any routing and use that response directly. Same if it
// returns `Failure`. We only route if it returns `Forward`. I've chosen not to
// go this direction because I feel like request guards are the correct
// mechanism to use here. In other words, enabling this at the fairing level
// encourages implicit handling, a bad practice. Fairings can still, however,
// return a default `Response` if routing fails via a response fairing. For
// instance, to automatically handle preflight in CORS, a response fairing can
// check that the user didn't handle the `OPTIONS` request (404) and return an
// appropriate response. This allows the users to handle `OPTIONS` requests
// when they'd like but default to the fairing when they don't want to.

/// Trait implemented by fairings: Rocket's structured middleware.
///
/// ## Fairing Information
///
/// Every `Fairing` must implement the
/// [`info`](/rocket/fairing/trait.Fairing.html#tymethod.info) method, which
/// returns an [`Info`](http://localhost:8000/rocket/fairing/struct.Info.html)
/// structure. This structure is used by Rocket to:
///
///   1. Assign a name to the `Fairing`.
///
///     This is the `name` field, which can be any arbitrary string. Name your
///     fairing something illustrative. The name will be logged during the
///     application's launch procedures.
///
///   2. Determine which callbacks to actually issue on the `Fairing`.
///
///     This is the `kind` field of type
///     [`Kind`](/rocket/fairing/struct.Kind.html). This field is a bitset that
///     represents the kinds of callbacks the fairing wishes to receive. Rocket
///     will only invoke the callbacks that are flagged in this set. `Kind`
///     structures can be `or`d together to represent any combination of kinds
///     of callbacks. For instance, to request launch and response callbacks,
///     return a `kind` field with the value `Kind::Launch | Kind::Response`.
///
/// See the [top-level documentation](/rocket/fairing/) for more general
/// information.
///
/// ## Fairing Callbacks
///
/// There are three kinds of fairing callbacks: launch, request, and response.
/// As mentioned above, a fairing can request any combination of these callbacks
/// through the `kind` field of the `Info` structure returned from the `info`
/// method. Rocket will only invoke the callbacks set in the `kind` field.
///
/// The three callback kinds are as follows:
///
///   * **Launch (`on_launch`)**
///
///     A launch callback, represented by the
///     [`on_launch`](/rocket/fairing/trait.Fairing.html#method.on_launch)
///     method, is called immediately before the Rocket application has
///     launched. At this point, Rocket has opened a socket for listening but
///     has not yet begun accepting connections. A launch callback can
///     arbitrarily modify the `Rocket` instance being launched. It returns `Ok`
///     if it would like launching to proceed nominally and `Err` otherwise. If
///     a launch callback returns `Err`, launch is aborted.
///
///   * **Request (`on_request`)**
///
///     A request callback, represented by the
///     [`on_request`](/rocket/fairing/trait.Fairing.html#method.on_request)
///     method, is called just after a request is received. At this point,
///     Rocket has parsed the incoming HTTP into
///     [`Request`](/rocket/struct.Request.html) and
///     [`Data`](/rocket/struct.Data.html) structures but has not routed the
///     request. A request callback can modify the request at will and
///     [`peek`](/rocket/struct.Data.html#method.peek) into the incoming data.
///     It may not, however, abort or respond directly to the request; these
///     issues are better handled via [request
///     guards](/rocket/request/trait.FromRequest.html) or via response
///     callbacks. A modified request is routed as if it was the original
///     request.
///
///   * **Response (`on_response`)**
///
///     A response callback is called when a response is ready to be sent to the
///     client. At this point, Rocket has completed all routing, including to
///     error catchers, and has generated the would-be final response. A
///     response callback can modify the response at will. For exammple, a
///     response callback can provide a default response when the user fails to
///     handle the request by checking for 404 responses.
///
/// # Implementing
///
/// A `Fairing` implementation has one required method: `info`. A `Fairing` can
/// also implement any of the available callbacks: `on_launch`, `on_request`,
/// and `on_response`. A `Fairing` _must_ set the appropriate callback kind in
/// the `kind` field of the returned `Info` structure from `info` for a callback
/// to actually be issued by Rocket.
///
/// A `Fairing` must be `Send + Sync + 'static`. This means that the fairing
/// must be sendable across thread boundaries (`Send`), thread-safe (`Sync`),
/// and have no non-`'static` reference (`'static`). Note that these bounds _do
/// not_ prohibit a `Fairing` from having state: the state need simply be
/// thread-safe and statically available or heap allocated.
///
/// # Example
///
/// Imagine that we want to record the number of `GET` and `POST` requests that
/// our application has received. While we could do this with [request
/// guards](/rocket/request/trait.FromRequest.html) and [managed
/// state](/rocket/request/struct.State.html), it would require us to annotate
/// every `GET` and `POST` request with custom types, polluting handler
/// signatures. Instead, we can create a simple fairing that does this globally.
///
/// The `Counter` fairing below records the number of all `GET` and `POST`
/// requests received. It makes these counts available at a special `'/counts'`
/// path.
///
/// ```rust
/// use std::io::Cursor;
/// use std::sync::atomic::{AtomicUsize, Ordering};
///
/// use rocket::{Request, Data, Response};
/// use rocket::fairing::{Fairing, Info, Kind};
/// use rocket::http::{Method, ContentType, Status};
///
/// #[derive(Default)]
/// struct Counter {
///     get: AtomicUsize,
///     post: AtomicUsize,
/// }
///
/// impl Fairing for Counter {
///     fn info(&self) -> Info {
///         Info {
///             name: "GET/POST Counter",
///             kind: Kind::Request | Kind::Response
///         }
///     }
///
///     fn on_request(&self, request: &mut Request, _: &Data) {
///         if request.method() == Method::Get {
///             self.get.fetch_add(1, Ordering::Relaxed);
///         } else if request.method() == Method::Post {
///             self.post.fetch_add(1, Ordering::Relaxed);
///         }
///     }
///
///     fn on_response(&self, request: &Request, response: &mut Response) {
///         // Don't change a successful user's response, ever.
///         if response.status() != Status::NotFound {
///             return
///         }
///
///         if request.method() == Method::Get && request.uri().path() == "/counts" {
///             let get_count = self.get.load(Ordering::Relaxed);
///             let post_count = self.post.load(Ordering::Relaxed);
///
///             let body = format!("Get: {}\nPost: {}", get_count, post_count);
///             response.set_status(Status::Ok);
///             response.set_header(ContentType::Plain);
///             response.set_sized_body(Cursor::new(body));
///         }
///     }
/// }
/// ```
pub trait Fairing: Send + Sync + 'static {
    /// Returns an [`Info`](/rocket/fairing/struct.Info.html) structure
    /// containing the `name` and [`Kind`](/rocket/fairing/struct.Kind.html) of
    /// this fairing. The `name` can be any arbitrary string. `Kind` must be an
    /// `or`d set of `Kind` variants.
    ///
    /// This is the only required method of a `Fairing`. All other methods have
    /// no-op default implementations.
    ///
    /// Rocket will only dispatch callbacks to this fairing for the kinds in the
    /// `kind` field of the returned `Info` structure. For instance, if
    /// `Kind::Launch | Kind::Request` is used, then Rocket will only call the
    /// `on_launch` and `on_request` methods of the fairing. Similarly, if
    /// `Kind::Response` is used, Rocket will only call the `on_response` method
    /// of this fairing.
    ///
    /// # Example
    ///
    /// An `info` implementation for `MyFairing`: a fairing named "My Custom
    /// Fairing" that is both a launch and response fairing.
    ///
    /// ```rust
    /// use rocket::fairing::{Fairing, Info, Kind};
    ///
    /// struct MyFairing;
    ///
    /// impl Fairing for MyFairing {
    ///     fn info(&self) -> Info {
    ///         Info {
    ///             name: "My Custom Fairing",
    ///             kind: Kind::Launch | Kind::Response
    ///         }
    ///     }
    /// }
    /// ```
    fn info(&self) -> Info;

    /// The launch callback. Returns `Ok` if launch should proceed and `Err` if
    /// launch should be aborted.
    ///
    /// This method is called just prior to launching an application if
    /// `Kind::Launch` is in the `kind` field of the `Info` structure for this
    /// fairing. The `rocket` parameter is the `Rocket` instance that was built
    /// for this application.
    ///
    /// The default implementation of this method simply returns `Ok(rocket)`.
    fn on_launch(&self, rocket: Rocket) -> Result<Rocket, Rocket> { Ok(rocket) }

    /// The request callback.
    ///
    /// This method is called when a new request is received if `Kind::Request`
    /// is in the `kind` field of the `Info` structure for this fairing. The
    /// `&mut Request` parameter is the incoming request, and the `&Data`
    /// parameter is the incoming data in the request.
    ///
    /// The default implementation of this method does nothing.
    #[allow(unused_variables)]
    fn on_request(&self, request: &mut Request, data: &Data) { }

    /// The response callback.
    ///
    /// This method is called when a response is ready to be issued to a client
    /// if `Kind::Response` is in the `kind` field of the `Info` structure for
    /// this fairing. The `&Request` parameter is the request that was routed,
    /// and the `&mut Response` parameter is the resulting response.
    ///
    /// The default implementation of this method does nothing.
    #[allow(unused_variables)]
    fn on_response(&self, request: &Request, response: &mut Response) { }
}
