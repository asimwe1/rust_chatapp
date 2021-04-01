use std::sync::Mutex;

use futures::future::{Future, BoxFuture};

use crate::{Rocket, Request, Response, Data};
use crate::fairing::{Fairing, Kind, Info};

/// A ad-hoc fairing that can be created from a function or closure.
///
/// This enum can be used to create a fairing from a simple function or closure
/// without creating a new structure or implementing `Fairing` directly.
///
/// # Usage
///
/// Use [`AdHoc::on_launch`], [`AdHoc::on_liftoff`], [`AdHoc::on_request()`], or
/// [`AdHoc::on_response()`] to create an `AdHoc` structure from a function or
/// closure. Then, simply attach the structure to the `Rocket` instance.
///
/// # Example
///
/// The following snippet creates a `Rocket` instance with two ad-hoc fairings.
/// The first, a liftoff fairing named "Liftoff Printer", simply prints a message
/// indicating that Rocket has launched. The second named "Put Rewriter", a
/// request fairing, rewrites the method of all requests to be `PUT`.
///
/// ```rust
/// use rocket::fairing::AdHoc;
/// use rocket::http::Method;
///
/// rocket::ignite()
///     .attach(AdHoc::on_liftoff("Liftoff Printer", |_| Box::pin(async move {
///         println!("...annnddd we have liftoff!");
///     })))
///     .attach(AdHoc::on_request("Put Rewriter", |req, _| Box::pin(async move {
///         req.set_method(Method::Put);
///     })));
/// ```
pub struct AdHoc {
    name: &'static str,
    kind: AdHocKind,
}

struct Once<F: ?Sized>(Mutex<Option<Box<F>>>);

impl<F: ?Sized> Once<F> {
    fn new(f: Box<F>) -> Self { Once(Mutex::new(Some(f))) }

    #[track_caller]
    fn take(&self) -> Box<F> {
        self.0.lock().expect("Once::lock()").take().expect("Once::take() called once")
    }
}

type Result<T = Rocket, E = Rocket> = std::result::Result<T, E>;

enum AdHocKind {
    /// An ad-hoc **launch** fairing. Called just before Rocket launches.
    Launch(Once<dyn FnOnce(Rocket) -> BoxFuture<'static, Result> + Send + 'static>),

    /// An ad-hoc **liftoff** fairing. Called just after Rocket launches.
    Liftoff(Once<dyn for<'a> FnOnce(&'a Rocket) -> BoxFuture<'a, ()> + Send + 'static>),

    /// An ad-hoc **request** fairing. Called when a request is received.
    Request(Box<dyn for<'a> Fn(&'a mut Request<'_>, &'a Data)
        -> BoxFuture<'a, ()> + Send + Sync + 'static>),

    /// An ad-hoc **response** fairing. Called when a response is ready to be
    /// sent to a client.
    Response(Box<dyn for<'r, 'b> Fn(&'r Request<'_>, &'b mut Response<'r>)
        -> BoxFuture<'b, ()> + Send + Sync + 'static>),
}

impl AdHoc {
    /// Constructs an `AdHoc` launch fairing named `name`. The function `f` will
    /// be called by Rocket just prior to launch. Returning an `Err` aborts
    /// launch.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket::fairing::AdHoc;
    ///
    /// // The no-op launch fairing.
    /// let fairing = AdHoc::on_launch("Boom!", |rocket| async move {
    ///     Ok(rocket)
    /// });
    /// ```
    pub fn on_launch<F, Fut>(name: &'static str, f: F) -> AdHoc
        where F: FnOnce(Rocket) -> Fut + Send + 'static,
              Fut: Future<Output=Result<Rocket, Rocket>> + Send + 'static,
    {
        AdHoc { name, kind: AdHocKind::Launch(Once::new(Box::new(|r| Box::pin(f(r))))) }
    }

    /// Constructs an `AdHoc` launch fairing named `name`. The function `f` will
    /// be called by Rocket just after launching.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket::fairing::AdHoc;
    ///
    /// // A fairing that prints a message just before launching.
    /// let fairing = AdHoc::on_liftoff("Boom!", |_| Box::pin(async move {
    ///     println!("Rocket has lifted off!");
    /// }));
    /// ```
    pub fn on_liftoff<F: Send + Sync + 'static>(name: &'static str, f: F) -> AdHoc
        where F: for<'a> FnOnce(&'a Rocket) -> BoxFuture<'a, ()>
    {
        AdHoc { name, kind: AdHocKind::Liftoff(Once::new(Box::new(f))) }
    }

    /// Constructs an `AdHoc` request fairing named `name`. The function `f`
    /// will be called and the returned `Future` will be `await`ed by Rocket
    /// when a new request is received.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket::fairing::AdHoc;
    ///
    /// // The no-op request fairing.
    /// let fairing = AdHoc::on_request("Dummy", |req, data| {
    ///     Box::pin(async move {
    ///         // do something with the request and data...
    /// #       let (_, _) = (req, data);
    ///     })
    /// });
    /// ```
    pub fn on_request<F: Send + Sync + 'static>(name: &'static str, f: F) -> AdHoc
        where F: for<'a> Fn(&'a mut Request<'_>, &'a Data) -> BoxFuture<'a, ()>
    {
        AdHoc { name, kind: AdHocKind::Request(Box::new(f)) }
    }

    // FIXME(rustc): We'd like to allow passing `async fn` to these methods...
    // https://github.com/rust-lang/rust/issues/64552#issuecomment-666084589

    /// Constructs an `AdHoc` response fairing named `name`. The function `f`
    /// will be called and the returned `Future` will be `await`ed by Rocket
    /// when a response is ready to be sent.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket::fairing::AdHoc;
    ///
    /// // The no-op response fairing.
    /// let fairing = AdHoc::on_response("Dummy", |req, resp| {
    ///     Box::pin(async move {
    ///         // do something with the request and pending response...
    /// #       let (_, _) = (req, resp);
    ///     })
    /// });
    /// ```
    pub fn on_response<F: Send + Sync + 'static>(name: &'static str, f: F) -> AdHoc
        where F: for<'b, 'r> Fn(&'r Request<'_>, &'b mut Response<'r>) -> BoxFuture<'b, ()>
    {
        AdHoc { name, kind: AdHocKind::Response(Box::new(f)) }
    }

    /// Constructs an `AdHoc` launch fairing that extracts a configuration of
    /// type `T` from the configured provider and stores it in managed state. If
    /// extractions fails, pretty-prints the error message and aborts launch.
    ///
    /// # Example
    ///
    /// ```rust
    /// use serde::Deserialize;
    /// use rocket::fairing::AdHoc;
    ///
    /// #[derive(Deserialize)]
    /// struct Config {
    ///     field: String,
    ///     other: usize,
    ///     /* and so on.. */
    /// }
    ///
    /// let fairing = AdHoc::config::<Config>();
    /// ```
    pub fn config<'de, T>() -> AdHoc
        where T: serde::Deserialize<'de> + Send + Sync + 'static
    {
        AdHoc::on_launch(std::any::type_name::<T>(), |rocket| async {
            let app_config = match rocket.figment().extract::<T>() {
                Ok(config) => config,
                Err(e) => {
                    crate::config::pretty_print_error(e);
                    return Err(rocket);
                }
            };

            Ok(rocket.manage(app_config))
        })
    }
}

#[crate::async_trait]
impl Fairing for AdHoc {
    fn info(&self) -> Info {
        let kind = match self.kind {
            AdHocKind::Launch(_) => Kind::Launch,
            AdHocKind::Liftoff(_) => Kind::Liftoff,
            AdHocKind::Request(_) => Kind::Request,
            AdHocKind::Response(_) => Kind::Response,
        };

        Info { name: self.name, kind }
    }

    async fn on_launch(&self, rocket: Rocket) -> Result<Rocket, Rocket> {
        match self.kind {
            AdHocKind::Launch(ref f) => (f.take())(rocket).await,
            _ => Ok(rocket)
        }
    }

    async fn on_liftoff(&self, rocket: &Rocket) {
        if let AdHocKind::Liftoff(ref f) = self.kind {
            (f.take())(rocket).await
        }
    }

    async fn on_request(&self, req: &mut Request<'_>, data: &mut Data) {
        if let AdHocKind::Request(ref f) = self.kind {
            f(req, data).await
        }
    }

    async fn on_response<'r>(&self, req: &'r Request<'_>, res: &mut Response<'r>) {
        if let AdHocKind::Response(ref f) = self.kind {
            f(req, res).await
        }
    }
}
