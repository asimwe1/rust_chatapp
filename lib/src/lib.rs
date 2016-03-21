#![feature(str_char)]

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
pub use response::{Response, HypResponse, HypFresh, Responder};
pub use request::Request;
pub use param::FromParam;
pub use router::Router;

use std::fmt;
use term_painter::ToStyle;
use term_painter::Color::*;
use hyper::uri::RequestUri;
use hyper::server::Handler as HypHandler;
use hyper::server::Request as HypRequest;
use hyper::Server;

pub type Handler<'a> = fn(Request) -> Response<'a>;

#[allow(dead_code)]
#[derive(Clone)]
pub struct Route<'a> {
    pub method: Method,
    pub path: &'static str,
    pub handler: Handler<'a>
}

impl<'a> fmt::Display for Route<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {:?}", Green.paint(&self.method), Blue.paint(&self.path))
    }
}

#[allow(dead_code)]
pub struct Rocket {
    address: &'static str,
    port: isize,
    router: Router
}

impl HypHandler for Rocket {
    fn handle<'a, 'k>(&'a self, req: HypRequest<'a, 'k>,
            res: HypResponse<'a, HypFresh>) {
        if let RequestUri::AbsolutePath(uri_string) = req.uri {
            if let Some(method) = Method::from_hyp(req.method) {
                println!("Request: {:?}", uri_string);
                self.router.route(method, uri_string.as_str());
                res.send(b"Hello, world!").unwrap();
                return;
            }
        }

        Response::not_found().body.respond(res);
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

    pub fn mount(&mut self, base: &'static str, routes: &[&Route<'static>])
            -> &mut Self {
        println!("🛰  {} '{}':", Magenta.paint("Mounting"), Blue.paint(base));
        for route in routes {
            println!("\t* {}", route);
            self.router.add_route(route.method.clone(), base, route.path);
        }

        self
    }

    pub fn mount_and_launch(mut self, base: &'static str,
                            routes: &[&Route<'static>]) {
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
