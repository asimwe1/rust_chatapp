extern crate hyper;

pub mod method;
pub mod error;
pub mod response;
pub mod request;
pub mod param;

use std::io::Write;

pub use method::Method;
pub use error::Error;
pub use response::{Response, HypResponse, HypFresh, Responder};
pub use request::Request;
pub use param::FromParam;

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

#[allow(dead_code)]
pub struct Rocket {
    address: &'static str,
    port: isize,
    handler: Option<Route<'static>>, // just for testing
    paths: Vec<&'static str> // for now, to check for collisions
    // mounts: HashMap<&'static str, Route<'a>>
}

impl HypHandler for Rocket {
    fn handle<'a, 'k>(&'a self, req: HypRequest<'a, 'k>,
                      mut res: HypResponse<'a, HypFresh>) {
        println!("Request: {:?}", req.uri);
        if self.handler.is_some() {
            let handler = self.handler.as_ref();
            let mut response = (handler.unwrap().handler)(Request::empty());
            response.body.respond(res);
        }
    }
}

impl Rocket {
    pub fn new(address: &'static str, port: isize) -> Rocket {
        Rocket {
            address: address,
            port: port,
            handler: None
        }
    }

    pub fn mount(&mut self, base: &str, routes: &[&Route<'static>]) -> &mut Self {
        println!("🛰 Mounting '{}':", base);
        for route in routes {
            if self.handler.is_none() {
                println!("\t* INSTALLED: {} '{}'", route.method, route.path);
                self.handler = Some((*route).clone());
            }
            println!("\t* {} '{}'", route.method, route.path);
        }

        self
    }

    pub fn mount_and_launch(mut self, base: &str, routes: &[&Route<'static>]) {
        self.mount(base, routes);
        self.launch();
    }

    pub fn launch(self) {
        let full_addr = format!("{}:{}", self.address, self.port);
        println!("🚀  Rocket has launched from {}...", full_addr);
        let _ = Server::http(full_addr.as_str()).unwrap().handle(self);
    }
}
