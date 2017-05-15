use {Rocket, Request, Response, Data};
use fairing::{Fairing, Kind, Info};

/// A ad-hoc fairing that can be created from a function or closure.
///
/// This enum can be used to create a fairing from a simple function or clusure
/// without creating a new structure or implementing `Fairing` directly.
///
/// # Usage
///
/// Use the [`on_launch`](#method.on_launch),
/// [`on_request`](#method.on_request), or [`on_response`](#method.on_response)
/// constructors to create an `AdHoc` structure from a function or closure.
/// Then, simply attach the structure to the `Rocket` instance.
///
/// # Example
///
/// The following snippet creates a `Rocket` instance with two ad-hoc fairings.
/// The first, a launch fairing, simply prints a message indicating that the
/// application is about to the launch. The second, a request fairing, rewrites
/// the method of all requests to be `PUT`.
///
/// ```rust
/// use rocket::fairing::AdHoc;
/// use rocket::http::Method;
///
/// rocket::ignite()
///     .attach(AdHoc::on_launch(|rocket| {
///         println!("Rocket is about to launch! Exciting! Here we go...");
///         Ok(rocket)
///     }))
///     .attach(AdHoc::on_request(|req, _| {
///         req.set_method(Method::Put);
///     }));
/// ```
pub enum AdHoc {
    /// An ad-hoc **launch** fairing. Called just before Rocket launches.
    #[doc(hidden)]
    Launch(Box<Fn(Rocket) -> Result<Rocket, Rocket> + Send + Sync + 'static>),
    /// An ad-hoc **request** fairing. Called when a request is received.
    #[doc(hidden)]
    Request(Box<Fn(&mut Request, &Data) + Send + Sync + 'static>),
    /// An ad-hoc **response** fairing. Called when a response is ready to be
    /// sent to a client.
    #[doc(hidden)]
    Response(Box<Fn(&Request, &mut Response) + Send + Sync + 'static>)
}

impl AdHoc {
    /// Constructs an `AdHoc` launch fairing. The function `f` will be called by
    /// Rocket just prior to launching.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket::fairing::AdHoc;
    ///
    /// // The no-op launch fairing.
    /// let fairing = AdHoc::on_launch(|rocket| Ok(rocket));
    /// ```
    pub fn on_launch<F>(f: F) -> AdHoc
        where F: Fn(Rocket) -> Result<Rocket, Rocket> + Send + Sync + 'static
    {
        AdHoc::Launch(Box::new(f))
    }

    /// Constructs an `AdHoc` request fairing. The function `f` will be called
    /// by Rocket when a new request is received.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket::fairing::AdHoc;
    ///
    /// // The no-op request fairing.
    /// let fairing = AdHoc::on_request(|req, data| {
    ///     // do something with the request and data...
    /// #   let (_, _) = (req, data);
    /// });
    /// ```
    pub fn on_request<F>(f: F) -> AdHoc
        where F: Fn(&mut Request, &Data) + Send + Sync + 'static
    {
        AdHoc::Request(Box::new(f))
    }

    /// Constructs an `AdHoc` response fairing. The function `f` will be called
    /// by Rocket when a response is ready to be sent.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket::fairing::AdHoc;
    ///
    /// // The no-op response fairing.
    /// let fairing = AdHoc::on_response(|req, resp| {
    ///     // do something with the request and pending response...
    /// #   let (_, _) = (req, resp);
    /// });
    /// ```
    pub fn on_response<F>(f: F) -> AdHoc
        where F: Fn(&Request, &mut Response) + Send + Sync + 'static
    {
        AdHoc::Response(Box::new(f))
    }
}

impl Fairing for AdHoc {
    fn info(&self) -> Info {
        use self::AdHoc::*;
        match *self {
            Launch(_) => Info { name: "AdHoc::Launch", kind: Kind::Launch },
            Request(_) => Info { name: "AdHoc::Request", kind: Kind::Request },
            Response(_) => Info { name: "AdHoc::Response", kind: Kind::Response }
        }
    }

    fn on_launch(&self, rocket: Rocket) -> Result<Rocket, Rocket> {
        match *self {
            AdHoc::Launch(ref launch_fn) => launch_fn(rocket),
            _ => Ok(rocket)
        }
    }

    fn on_request(&self, request: &mut Request, data: &Data) {
        if let AdHoc::Request(ref callback) = *self {
            callback(request, data)
        }
    }

    fn on_response(&self, request: &Request, response: &mut Response) {
        if let AdHoc::Response(ref callback) = *self {
            callback(request, response)
        }
    }
}
