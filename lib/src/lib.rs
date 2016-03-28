#![feature(str_char)]
#![feature(specialization)]

extern crate term_painter;
extern crate hyper;

pub mod method;
pub mod error;
pub mod response;
pub mod request;
pub mod param;
pub mod router;

pub use method::Method;
pub use error::Error;
pub use request::Request;
pub use param::FromParam;
pub use router::Router;
pub use response::{Response, HypResponse, Responder, HypFresh};

use std::fmt;
use term_painter::ToStyle;
use term_painter::Color::*;
use hyper::uri::RequestUri;
use hyper::server::Handler as HypHandler;
use hyper::server::Request as HypRequest;
use hyper::Server;

pub type Handler<'a> = fn(Request) -> Response<'a>;

// TODO: Figure out if using 'static for Handler is a good idea.
// TODO: Merge this `Route` and route::Route, somewhow.
pub struct Route {
    pub method: Method,
    pub path: &'static str,
    pub handler: Handler<'static>
}

impl Route {
    pub fn new(method: Method, path: &'static str, handler: Handler<'static>)
            -> Route {
        Route {
            method: method,
            path: path,
            handler: handler
        }
    }
}

impl<'a> fmt::Display for Route {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {:?}", Green.paint(&self.method), Blue.paint(&self.path))
    }
}

pub struct Rocket {
    address: &'static str,
    port: isize,
    router: Router
}

impl HypHandler for Rocket {
    fn handle<'a, 'k>(&'a self, req: HypRequest<'a, 'k>,
            res: HypResponse<'a, HypFresh>) {
        println!("{} {:?} {:?}", White.paint("Incoming:"),
            Green.paint(&req.method), Blue.paint(&req.uri));
        if let RequestUri::AbsolutePath(uri_string) = req.uri {
            if let Some(method) = Method::from_hyp(req.method) {
                let uri_str = uri_string.as_str();
                let route = self.router.route(method, uri_str);
                let mut response = route.map_or(Response::not_found(), |route| {
                    let params = route.get_params(uri_str);
                    let request = Request::new(params, uri_str);
                    (route.handler)(request)
                });

                println!("{}", Green.paint("\t=> Dispatched request."));
                return response.respond(res);
            }

            println!("{}", Yellow.paint("\t=> Debug: Method::from_hyp failed!"));
        }

        println!("{}", Red.paint("\t=> Dispatch failed. Returning 404."));
        Response::not_found().respond(res);
    }
}

impl Rocket {
    pub fn new(address: &'static str, port: isize) -> Rocket {
        Rocket {
            address: address,
            port: port,
            router: Router::new()
        }
    }

    pub fn mount(&mut self, base: &'static str, routes: &[&Route]) -> &mut Self {
        println!("🛰  {} '{}':", Magenta.paint("Mounting"), Blue.paint(base));
        for route in routes {
            println!("\t* {}", route);
            self.router.add_route(route.method, base, route.path, route.handler);
        }

        self
    }

    pub fn mount_and_launch(mut self, base: &'static str, routes: &[&Route]) {
        self.mount(base, routes);
        self.launch();
    }

    pub fn launch(self) {
        if self.router.has_collisions() {
            println!("{}", Yellow.paint("Warning: route collisions detected!"));
        }

        let full_addr = format!("{}:{}", self.address, self.port);
        println!("🚀  {} {}...", White.paint("Rocket has launched from"),
            White.bold().paint(&full_addr));
        let _ = Server::http(full_addr.as_str()).unwrap().handle(self);
    }
}
