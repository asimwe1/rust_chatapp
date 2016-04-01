use super::*;
use response::{HyperResponse, HyperFresh};
use request::HyperRequest;

use std::io::Read;
use term_painter::Color::*;
use term_painter::ToStyle;

use hyper::uri::RequestUri as HyperRequestUri;
use hyper::server::Server as HyperServer;
use hyper::server::Handler as HyperHandler;

pub struct Rocket {
    address: &'static str,
    port: isize,
    router: Router
}

impl HyperHandler for Rocket {
    fn handle<'a, 'k>(&'a self, mut req: HyperRequest<'a, 'k>,
                                    res: HyperResponse<'a, HyperFresh>) {
        println!("{} {:?} {:?}", White.paint("Incoming:"),
            Green.paint(&req.method), Blue.paint(&req.uri));

        let mut buf = vec![];
        req.read_to_end(&mut buf); // FIXME: Simple DOS attack here.
        if let HyperRequestUri::AbsolutePath(uri_string) = req.uri {
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
        let _ = HyperServer::http(full_addr.as_str()).unwrap().handle(self);
    }
}
