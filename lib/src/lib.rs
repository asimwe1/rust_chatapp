extern crate hyper;

mod method;
mod error;
mod response;
mod request;

pub use method::Method;
pub use error::Error;
pub use response::Response;
pub use request::Request;

use hyper::server::Handler as HypHandler;
use hyper::server::Request as HypRequest;
use hyper::server::Response as HypResponse;
use hyper::net::Fresh as HypFresh;
use hyper::Server;

pub type Handler = fn(Request) -> Response;

#[allow(dead_code)]
pub struct Route<'a> {
    pub method: Method,
    pub path: &'a str,
    pub handler: Handler
}

#[allow(dead_code)]
pub struct Rocket {
    address: &'static str,
    port: isize,
    // mounts: HashMap<&'static str, Route<'a>>
}

impl HypHandler for Rocket {
    fn handle<'a, 'k>(&'a self, req: HypRequest<'a, 'k>,
                      res: HypResponse<'a, HypFresh>) {
        println!("Request: {:?}", req.uri);
        res.send(b"Hello World!").unwrap();
    }
}

impl Rocket {
    pub fn new(address: &'static str, port: isize) -> Rocket {
        Rocket {
            address: address,
            port: port
        }
    }

    pub fn mount(&mut self, base: &str, routes: &[&Route]) -> &mut Self {
        println!("🛰  Mounting '{}':", base);
        for route in routes {
            println!("\t* {} '{}'", route.method, route.path);
        }

        self
    }

    pub fn mount_and_launch(mut self, base: &str, routes: &[&Route]) {
        self.mount(base, routes);
        self.launch();
    }

    pub fn launch(self) {
        let full_addr = format!("{}:{}", self.address, self.port);
        println!("🚀  Rocket has launched from {}...", full_addr);
        let _ = Server::http(full_addr.as_str()).unwrap().handle(self);
    }
}
