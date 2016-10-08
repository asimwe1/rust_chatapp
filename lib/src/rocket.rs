use std::collections::HashMap;
use std::str::from_utf8_unchecked;
use std::cmp::min;
use std::process;

use term_painter::Color::*;
use term_painter::ToStyle;

use config;
use logger;
use request::{Request, Data, FormItems};
use response::{Response};
use router::{Router, Route};
use catcher::{self, Catcher};
use outcome::Outcome;
use error::Error;

use http::{Method, StatusCode};
use http::hyper::{HyperRequest, FreshHyperResponse};
use http::hyper::{HyperServer, HyperHandler, HyperSetCookie, header};

pub struct Rocket {
    address: String,
    port: usize,
    router: Router,
    default_catchers: HashMap<u16, Catcher>,
    catchers: HashMap<u16, Catcher>,
}

impl HyperHandler for Rocket {
    fn handle<'h, 'k>(&self,
                      req: HyperRequest<'h, 'k>,
                      mut res: FreshHyperResponse<'h>) {
        res.headers_mut().set(header::Server("rocket".to_string()));
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
                return self.handle_error(StatusCode::InternalServerError,
                                         &mock_request, res);
            }
        };

        // Retrieve the data from the request.
        let mut data = Data::new();

        info!("{}:", request);
        let matches = self.router.route(&request);
        for route in matches {
            // Retrieve and set the requests parameters.
            info_!("Matched: {}", route);
            request.set_params(route);

            // Dispatch the request to the handler.
            let response = (route.handler)(&request, data);

            // Check if the request processing completed or if the request needs
            // to be forwarded. If it does, continue the loop to try again.
            info_!("{} {}", White.paint("Response:"), response);
            let mut responder = match response {
                Response::Complete(responder) => responder,
                Response::Forward(unused_data) => {
                    data = unused_data;
                    continue;
                }
            };

            // We have a responder. Update the cookies in the header.
            let cookie_delta = request.cookies().delta();
            if cookie_delta.len() > 0 {
                res.headers_mut().set(HyperSetCookie(cookie_delta));
            }

            // Actually process the response.
            let outcome = responder.respond(res);
            info_!("{} {}", White.paint("Outcome:"), outcome);

            // Check if the responder wants to forward to a catcher.
            match outcome {
                Outcome::Forward((c, r)) => return self.handle_error(c, &request, r),
                Outcome::Success | Outcome::Failure => return,
            };
        }

        error_!("No matching routes.");
        self.handle_error(StatusCode::NotFound, &request, res);
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

            let mut form_items = FormItems(form);
            if let Some(("_method", value)) = form_items.next() {
                if let Ok(method) = value.parse() {
                    req.method = method;
                }
            }
        }
    }

    // Call when no route was found.
    fn handle_error<'r>(&self,
                        code: StatusCode,
                        req: &'r Request,
                        response: FreshHyperResponse) {
        error_!("Dispatch failed: {}.", code);
        let catcher = self.catchers.get(&code.to_u16()).unwrap();

        if let Some(mut responder) = catcher.handle(Error::NoRoute, req).responder() {
            if responder.respond(response) != Outcome::Success {
                error_!("Catcher outcome was unsuccessul; aborting response.");
            } else {
                info_!("Responded with catcher.");
            }
        } else {
            error_!("Catcher returned an incomplete response.");
            warn_!("Using default error response.");
            let catcher = self.default_catchers.get(&code.to_u16()).unwrap();
            let responder = catcher.handle(Error::Internal, req).responder();
            responder.unwrap().respond(response).expect_success()
        }
    }

    pub fn mount(mut self, base: &'static str, routes: Vec<Route>) -> Self {
        info!("🛰  {} '{}':", Magenta.paint("Mounting"), base);
        for mut route in routes {
            let path = format!("{}/{}", base, route.path.as_str());
            route.set_path(path);

            info_!("{}", route);
            self.router.add(route);
        }

        self
    }

    pub fn catch(mut self, catchers: Vec<Catcher>) -> Self {
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

    pub fn launch(self) {
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

    /// Retrieves the configuration parameter named `name` for the current
    /// environment. Returns Some(value) if the paremeter exists. Otherwise,
    /// returns None.
    pub fn config<S: AsRef<str>>(_name: S) -> Option<&'static str> {
        // TODO: Implement me.
        None
    }

    pub fn ignite() -> Rocket {
        // Note: read_or_default will exit the process under errors.
        let config = config::read_or_default();

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
            default_catchers: catcher::defaults::get(),
            catchers: catcher::defaults::get(),
        }
    }
}
