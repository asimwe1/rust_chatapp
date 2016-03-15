extern crate hyper;

mod method;
mod error;
mod response;
mod request;

use std::io::Write;

pub use method::Method;
pub use error::Error;
pub use response::{Body, Response};
pub use request::Request;

use hyper::server::Handler as HypHandler;
use hyper::server::Request as HypRequest;
use hyper::server::Response as HypResponse;
use hyper::net::Fresh as HypFresh;
use hyper::Server;

pub type Handler<'a> = fn(Request) -> Response<'a>;

#[allow(dead_code)]
#[derive(Clone)]
pub struct Route<'a, 'b> {
    pub method: Method,
    pub path: &'a str,
    pub handler: Handler<'b>
}

#[allow(dead_code)]
pub struct Rocket {
    address: &'static str,
    port: isize,
    handler: Option<Route<'static, 'static>> // just for testing
    // mounts: HashMap<&'static str, Route<'a>>
}

impl HypHandler for Rocket {
    fn handle<'a, 'k>(&'a self, req: HypRequest<'a, 'k>,
                      mut res: HypResponse<'a, HypFresh>) {
        println!("Request: {:?}", req.uri);
        if self.handler.is_some() {
            let response = (self.handler.as_ref().unwrap().handler)(Request::empty());
            *(res.headers_mut()) = response.headers;
            *(res.status_mut()) = response.status;
            match response.body {
                Body::Str(string) => {
                    let mut stream = res.start().unwrap();
                    stream.write_all(string.as_bytes()).unwrap();
                    stream.end();
                }
                _ => println!("UNIMPLEMENTED")
            }
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

    pub fn mount(&mut self, base: &str, routes: &[&Route<'static, 'static>]) -> &mut Self {
        println!("🛰  Mounting '{}':", base);
        for route in routes {
            if self.handler.is_none() {
                println!("\t* INSTALLED: {} '{}'", route.method, route.path);
                self.handler = Some((*route).clone());
            }
            println!("\t* {} '{}'", route.method, route.path);
        }

        self
    }

    pub fn mount_and_launch(mut self, base: &str, routes: &[&Route<'static, 'static>]) {
        self.mount(base, routes);
        self.launch();
    }

    pub fn launch(self) {
        let full_addr = format!("{}:{}", self.address, self.port);
        println!("🚀  Rocket has launched from {}...", full_addr);
        let _ = Server::http(full_addr.as_str()).unwrap().handle(self);
    }
}
