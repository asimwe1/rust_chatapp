use super::*;
use response::{FreshHyperResponse, Outcome};
use request::HyperRequest;
use catcher;

use std::collections::HashMap;

use term_painter::Color::*;
use term_painter::ToStyle;

use hyper::server::Server as HyperServer;
use hyper::server::Handler as HyperHandler;
use hyper::header::SetCookie;

pub struct Rocket {
    address: String,
    port: isize,
    router: Router,
    catchers: HashMap<u16, Catcher>,
    log_set: bool,
}

impl HyperHandler for Rocket {
    fn handle<'h, 'k>(&self, req: HyperRequest<'h, 'k>,
            mut res: FreshHyperResponse<'h>) {
        res.headers_mut().set(response::header::Server("rocket".to_string()));
        self.dispatch(req, res)
    }
}

impl Rocket {
    fn dispatch<'h, 'k>(&self, hyp_req: HyperRequest<'h, 'k>,
                        mut res: FreshHyperResponse<'h>) {
        // Get a copy of the URI for later use.
        let uri = hyp_req.uri.to_string();

        // Try to create a Rocket request from the hyper request.
        let request = match Request::from_hyp(hyp_req) {
            Ok(req) => req,
            Err(reason) => {
                let mock_request = Request::mock(Method::Get, uri.as_str());
                return self.handle_internal_error(reason, &mock_request, res);
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

            // Get the result if we failed so we can try again.
            res = match outcome {
                Outcome::FailForward(r) => r,
                Outcome::Complete | Outcome::FailStop => return,
            };
        }

        error_!("No matching routes.");
        self.handle_not_found(&request, res);
    }

    // Call on internal server error.
    fn handle_internal_error<'r>(&self, reason: String, request: &'r Request<'r>,
                            response: FreshHyperResponse) {
        error!("Internal server error.");
        debug!("{}", reason);
        let catcher = self.catchers.get(&500).unwrap();
        catcher.handle(Error::Internal, request).respond(response);
    }

    // Call when no route was found.
    fn handle_not_found<'r>(&self, request: &'r Request<'r>,
                            response: FreshHyperResponse) {
        error_!("{} dispatch failed: 404.", request);
        let catcher = self.catchers.get(&404).unwrap();
        catcher.handle(Error::NoRoute, request).respond(response);
    }

    pub fn new<S: ToString>(address: S, port: isize) -> Rocket {
        Rocket {
            address: address.to_string(),
            port: port,
            router: Router::new(),
            catchers: catcher::defaults::get(),
            log_set: false,
        }
    }

    pub fn mount(&mut self, base: &'static str, routes: Vec<Route>)
            -> &mut Self {
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

    pub fn launch(mut self) {
        self.enable_normal_logging_if_disabled();
        if self.router.has_collisions() {
            warn!("Route collisions detected!");
        }

        let full_addr = format!("{}:{}", self.address, self.port);
        info!("🚀  {} {}...", White.paint("Rocket has launched from"),
            White.bold().paint(&full_addr));
        let _ = HyperServer::http(full_addr.as_str()).unwrap().handle(self);
    }

    pub fn mount_and_launch(mut self, base: &'static str, routes: Vec<Route>) {
        self.mount(base, routes);
        self.launch();
    }
}
