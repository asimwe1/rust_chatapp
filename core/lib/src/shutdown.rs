use std::sync::Arc;

use tokio::sync::Notify;

use crate::request::{FromRequest, Outcome, Request};

/// A request guard to gracefully shutdown a Rocket server.
///
/// A server shutdown is manually requested by calling [`Shutdown::notify()`]
/// or, if enabled, by pressing `Ctrl-C`. Rocket will finish handling any
/// pending requests and return `Ok()` to the caller of [`Rocket::launch()`].
///
/// [`Rocket::launch()`]: crate::Rocket::launch()
///
/// # Example
///
/// ```rust,no_run
/// # #[macro_use] extern crate rocket;
/// #
/// use rocket::Shutdown;
///
/// #[get("/shutdown")]
/// fn shutdown(shutdown: Shutdown) -> &'static str {
///     shutdown.notify();
///     "Shutting down..."
/// }
///
/// #[rocket::main]
/// async fn main() {
///     let result = rocket::build()
///         .mount("/", routes![shutdown])
///         .launch()
///         .await;
///
///     // If the server shut down (by visiting `/shutdown`), `result` is `Ok`.
///     result.expect("server failed unexpectedly");
/// }
/// ```
#[must_use = "a shutdown request is only sent on `shutdown.notify()`"]
#[derive(Debug, Clone)]
pub struct Shutdown(pub(crate) Arc<Notify>);

impl Shutdown {
    /// Notify Rocket to shut down gracefully.
    ///
    /// This function returns immediately; pending requests will continue to run
    /// until completion before the actual shutdown occurs.
    #[inline]
    pub fn notify(self) {
        self.0.notify_one();
        info!("Server shutdown requested, waiting for all pending requests to finish.");
    }
}

#[crate::async_trait]
impl<'r> FromRequest<'r> for Shutdown {
    type Error = std::convert::Infallible;

    #[inline]
    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let notifier = request.rocket().shutdown.clone();
        Outcome::Success(Shutdown(notifier))
    }
}
