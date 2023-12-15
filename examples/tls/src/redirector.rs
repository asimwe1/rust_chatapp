//! Redirect all HTTP requests to HTTPs.

use std::sync::OnceLock;

use rocket::http::Status;
use rocket::log::LogLevel;
use rocket::{route, Error, Request, Data, Route, Orbit, Rocket, Ignite, Config};
use rocket::fairing::{Fairing, Info, Kind};
use rocket::response::Redirect;

#[derive(Debug, Clone)]
pub struct Redirector {
    pub listen_port: u16,
    pub tls_port: OnceLock<u16>,
}

impl Redirector {
    pub fn on(port: u16) -> Self {
        Redirector { listen_port: port, tls_port: OnceLock::new() }
    }

    // Route function that gets called on every single request.
    fn redirect<'r>(req: &'r Request, _: Data<'r>) -> route::BoxFuture<'r> {
        // FIXME: Check the host against a whitelist!
        let redirector = req.rocket().state::<Self>().expect("managed Self");
        if let Some(host) = req.host() {
            let domain = host.domain();
            let https_uri = match redirector.tls_port.get() {
                Some(443) | None => format!("https://{domain}{}", req.uri()),
                Some(port) => format!("https://{domain}:{port}{}", req.uri()),
            };

            route::Outcome::from(req, Redirect::permanent(https_uri)).pin()
        } else {
            route::Outcome::from(req, Status::BadRequest).pin()
        }
    }

    // Launch an instance of Rocket than handles redirection on `self.port`.
    pub async fn try_launch(self, mut config: Config) -> Result<Rocket<Ignite>, Error> {
        use yansi::Paint;
        use rocket::http::Method::*;

        // Determine the port TLS is being served on.
        let tls_port = self.tls_port.get_or_init(|| config.port);

        // Adjust config for redirector: disable TLS, set port, disable logging.
        config.tls = None;
        config.port = self.listen_port;
        config.log_level = LogLevel::Critical;

        info!("{}{}", "🔒 ".mask(), "HTTP -> HTTPS Redirector:".magenta());
        info_!("redirecting on insecure port {} to TLS port {}",
            self.listen_port.yellow(), tls_port.green());

        // Build a vector of routes to `redirect` on `<path..>` for each method.
        let redirects = [Get, Put, Post, Delete, Options, Head, Trace, Connect, Patch]
            .into_iter()
            .map(|m| Route::new(m, "/<path..>", Self::redirect))
            .collect::<Vec<_>>();

        rocket::custom(config)
            .manage(self)
            .mount("/", redirects)
            .launch()
            .await
    }
}

#[rocket::async_trait]
impl Fairing for Redirector {
    fn info(&self) -> Info {
        Info {
            name: "HTTP -> HTTPS Redirector",
            kind: Kind::Liftoff | Kind::Singleton
        }
    }

    async fn on_liftoff(&self, rkt: &Rocket<Orbit>) {
        let (this, shutdown, config) = (self.clone(), rkt.shutdown(), rkt.config().clone());
        let _ = rocket::tokio::spawn(async move {
            if let Err(e) = this.try_launch(config).await {
                error!("Failed to start HTTP -> HTTPS redirector.");
                info_!("Error: {}", e);
                error_!("Shutting down main instance.");
                shutdown.notify();
            }
        });
    }
}
