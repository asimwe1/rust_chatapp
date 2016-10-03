use super::*;
use response::{FreshHyperResponse, Outcome};
use request::HyperRequest;
use catcher;
use config::RocketConfig;

use std::collections::HashMap;
use std::str::from_utf8_unchecked;
use std::cmp::min;
use std::process;

use term_painter::Color::*;
use term_painter::ToStyle;

use hyper::server::Server as HyperServer;
use hyper::server::Handler as HyperHandler;
use hyper::header::SetCookie;

pub struct Rocket {
    address: String,
    port: usize,
    router: Router,
    catchers: HashMap<u16, Catcher>,
    log_set: bool,
}

impl HyperHandler for Rocket {
    fn handle<'h, 'k>(&self,
                      req: HyperRequest<'h, 'k>,
                      mut res: FreshHyperResponse<'h>) {
        res.headers_mut().set(response::header::Server("rocket".to_string()));
        self.dispatch(req, res)
    }
}

impl Rocket {
    fn dispatch<'h, 'k>(&self,
                        hyp_req: HyperRequest<'h, 'k>,
                        mut res: FreshHyperResponse<'h>) {
        // Get a copy of the URI for later use.
        let uri = hyp_req.uri.to_string();

        // Try to create a Rocket request from the hyper request.
        let request = match Request::from_hyp(hyp_req) {
            Ok(mut req) => {
                self.preprocess_request(&mut req);
                req
            }
            Err(ref reason) => {
                let mock_request = Request::mock(Method::Get, uri.as_str());
                debug_!("Bad request: {}", reason);
                return self.handle_internal_error(&mock_request, res);
            }
        };

        info!("{}:", request);
        let matches = self.router.route(&request);
        for route in matches {
            // Retrieve and set the requests parameters.
            info_!("Matched: {}", route);
            request.set_params(route);

            // Dispatch the request to the handler and update the cookies.
            let mut responder = (route.handler)(&request);
            let cookie_delta = request.cookies().delta();
            if cookie_delta.len() > 0 {
                res.headers_mut().set(SetCookie(cookie_delta));
            }

            // Get the response.
            let outcome = responder.respond(res);
            info_!("{} {}", White.paint("Outcome:"), outcome);

            // Get the result if we failed forward so we can try again.
            res = match outcome {
                Outcome::Complete | Outcome::FailStop => return,
                Outcome::FailForward(r) => r,
                Outcome::Bad(r) => return self.handle_internal_error(&request, r),
            };
        }

        error_!("No matching routes.");
        self.handle_not_found(&request, res);
    }

    /// Preprocess the request for Rocket-specific things. At this time, we're
    /// only checking for _method in forms.
    fn preprocess_request(&self, req: &mut Request) {
        // Check if this is a form and if the form contains the special _method
        // field which we use to reinterpret the request's method.
        let data_len = req.data.len();
        let (min_len, max_len) = ("_method=get".len(), "_method=delete".len());
        if req.content_type().is_form() && data_len >= min_len {
            let form = unsafe {
                from_utf8_unchecked(&req.data.as_slice()[..min(data_len, max_len)])
            };

            let mut form_items = form::FormItems(form);
            if let Some(("_method", value)) = form_items.next() {
                if let Ok(method) = value.parse() {
                    req.method = method;
                }
            }
        }
    }

    // Call on internal server error.
    fn handle_internal_error<'r>(&self,
                                 request: &'r Request<'r>,
                                 response: FreshHyperResponse) {
        error_!("Internal server error.");
        let catcher = self.catchers.get(&500).unwrap();
        catcher.handle(Error::Internal, request).respond(response);
    }

    // Call when no route was found.
    fn handle_not_found<'r>(&self,
                            request: &'r Request<'r>,
                            response: FreshHyperResponse) {
        error_!("{} dispatch failed: 404.", request);
        let catcher = self.catchers.get(&404).unwrap();
        catcher.handle(Error::NoRoute, request).respond(response);
    }

    pub fn new<S: ToString>(address: S, port: usize) -> Rocket {
        Rocket {
            address: address.to_string(),
            port: port,
            router: Router::new(),
            catchers: catcher::defaults::get(),
            log_set: false,
        }
    }

    pub fn mount(&mut self, base: &'static str, routes: Vec<Route>) -> &mut Self {
        self.enable_normal_logging_if_disabled();
        info!("🛰  {} '{}':", Magenta.paint("Mounting"), base);
        for mut route in routes {
            let path = format!("{}/{}", base, route.path.as_str());
            route.set_path(path);

            info_!("{}", route);
            self.router.add(route);
        }

        self
    }

    pub fn catch(&mut self, catchers: Vec<Catcher>) -> &mut Self {
        self.enable_normal_logging_if_disabled();
        info!("👾  {}:", Magenta.paint("Catchers"));
        for c in catchers {
            if self.catchers.contains_key(&c.code) &&
                    !self.catchers.get(&c.code).unwrap().is_default() {
                let msg = format!("warning: overrides {} catcher!", c.code);
                warn!("{} ({})", c, Yellow.paint(msg.as_str()));
            } else {
                info_!("{}", c);
            }

            self.catchers.insert(c.code, c);
        }

        self
    }

    fn enable_normal_logging_if_disabled(&mut self) {
        if !self.log_set {
            logger::init(LoggingLevel::Normal);
            self.log_set = true;
        }
    }

    pub fn log(&mut self, level: LoggingLevel) {
        if self.log_set {
            warn!("Log level already set! Not overriding.");
        } else {
            logger::init(level);
            self.log_set = true;
        }
    }

    /// Retrieves the configuration parameter named `name` for the current
    /// environment. Returns Some(value) if the paremeter exists. Otherwise,
    /// returns None.
    pub fn config<S: AsRef<str>>(_name: S) -> Option<&'static str> {
        // TODO: Implement me.
        None
    }

    pub fn launch(mut self) {
        self.enable_normal_logging_if_disabled();
        if self.router.has_collisions() {
            warn!("Route collisions detected!");
        }

        let full_addr = format!("{}:{}", self.address, self.port);
        let server = match HyperServer::http(full_addr.as_str()) {
            Ok(hyper_server) => hyper_server,
            Err(e) => {
                error!("failed to start server.");
                error_!("{}", e);
                process::exit(1);
            }
        };

        info!("🚀  {} {}...",
              White.paint("Rocket has launched from"),
              White.bold().paint(&full_addr));

        server.handle(self).unwrap();
    }

    pub fn mount_and_launch(mut self, base: &'static str, routes: Vec<Route>) {
        self.mount(base, routes);
        self.launch();
    }

    pub fn ignite() -> Rocket {
        use config::ConfigError::*;
        let config = match RocketConfig::read() {
            Ok(config) => config,
            Err(e@ParseError(..)) | Err(e@BadEntry(..)) |
            Err(e@BadEnv(..)) | Err(e@BadType(..))  => {
                logger::init(LoggingLevel::Debug);
                e.pretty_print();
                process::exit(1)
            }
            Err(IOError) | Err(BadCWD) => {
                warn!("error reading Rocket config file; using defaults.");
                RocketConfig::default()
            }
            Err(NotFound) => RocketConfig::default()
        };

        logger::init(config.active().log_level);
        info!("🔧  Configured for {}.", config.active_env);
        info_!("listening: {}:{}",
               White.paint(&config.active().address),
               White.paint(&config.active().port));
        info_!("logging: {:?}", White.paint(config.active().log_level));
        info_!("session key: {}",
               White.paint(config.active().session_key.is_some()));

        Rocket {
            address: config.active().address.clone(),
            port: config.active().port,
            router: Router::new(),
            catchers: catcher::defaults::get(),
            log_set: true,
        }
    }
}
