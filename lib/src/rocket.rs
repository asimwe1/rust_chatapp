use super::*;
use response::FreshHyperResponse;
use request::HyperRequest;
use catcher;

use std::collections::HashMap;

use term_painter::Color::*;
use term_painter::ToStyle;

use hyper::server::Server as HyperServer;
use hyper::server::Handler as HyperHandler;

pub struct Rocket {
    address: &'static str,
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
                        res: FreshHyperResponse<'h>) {
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
        let route = self.router.route(&request);
        if let Some(ref route) = route {
            // Retrieve and set the requests parameters.
            request.set_params(route);

            // Here's the magic: dispatch the request to the handler.
            let outcome = (route.handler)(&request).respond(res);
            info_!("{} {}", White.paint("Outcome:"), outcome);

            // TODO: keep trying lower ranked routes before dispatching a not
            // found error.
            outcome.map_forward(|res| {
                error_!("No further matching routes.");
                // TODO: Have some way to know why this was failed forward. Use that
                // instead of always using an unchained error.
                self.handle_not_found(&request, res);
            });
        } else {
            error_!("No matching routes.");
            self.handle_not_found(&request, res);
        }
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

    pub fn new(address: &'static str, port: isize) -> Rocket {
        Rocket {
            address: address,
            port: port,
            router: Router::new(),
            catchers: catcher::defaults::get(),
            log_set: false,
        }
    }

    pub fn mount(&mut self, base: &'static str, routes: Vec<Route>)
            -> &mut Self {
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
        info!("👾  {}:", Magenta.paint("Catchers"));
        for c in catchers {
            if self.catchers.contains_key(&c.code) &&
                    !self.catchers.get(&c.code).unwrap().is_default() {
                let msg = format!("warning: overrides {} catcher!", c.code);
                info_!("{} ({})", c, Yellow.paint(msg.as_str()));
            } else {
                info_!("{}", c);
            }

            self.catchers.insert(c.code, c);
        }

        self
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
        if self.router.has_collisions() {
            warn!("Route collisions detected!");
        }

        if !self.log_set {
            self.log(LoggingLevel::Normal)
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
