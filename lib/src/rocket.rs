use super::*;
use response::{HyperResponse, HyperFresh};
use request::HyperRequest;
use catcher;

use std::io::Read;
use std::collections::HashMap;
use term_painter::Color::*;
use term_painter::ToStyle;

use hyper::uri::RequestUri as HyperRequestUri;
use hyper::server::Server as HyperServer;
use hyper::server::Handler as HyperHandler;

pub struct Rocket {
    address: &'static str,
    port: isize,
    router: Router,
    catchers: HashMap<u16, Catcher>,
}

impl HyperHandler for Rocket {
    fn handle<'a, 'k>(&'a self, mut req: HyperRequest<'a, 'k>,
                                    res: HyperResponse<'a, HyperFresh>) {
        println!("{:?} {:?}", Green.paint(&req.method), Blue.paint(&req.uri));

        let mut buf = vec![];
        req.read_to_end(&mut buf); // FIXME: Simple DOS attack here.
        if let HyperRequestUri::AbsolutePath(uri_string) = req.uri {
            if let Some(method) = Method::from_hyp(req.method) {
                let uri_str = uri_string.as_str();
                let route = self.router.route(method, uri_str);

                if route.is_some() {
                    let route = route.unwrap();
                    let params = route.get_params(uri_str);
                    let request = Request::new(params, uri_str, &buf);

                    println!("{}", Green.paint("\t=> Dispatching request."));
					// FIXME: Responder should be able to say it didn't work.
                    return (route.handler)(request).respond(res);
                } else {
                    // FIXME: Try next highest ranking route, not just 404.
                    let request = Request::new(vec![], uri_str, &buf);
					let handler_404 = self.catchers.get(&404).unwrap().handler;

					let msg = "\t=> Dispatch failed. Returning 404.";
					println!("{}", Red.paint(msg));
					return handler_404(request).respond(res);
                }
            }

            println!("{}", Yellow.paint("\t=> Debug: Method::from_hyp failed!"));
        }

		println!("{}", Red.paint("\t=> Internal failure. Bad method or path."));
        Response::server_error().respond(res);
    }
}

impl Rocket {
    pub fn new(address: &'static str, port: isize) -> Rocket {
        Rocket {
            address: address,
            port: port,
            router: Router::new(),
            catchers: catcher::defaults::get(),
        }
    }

    pub fn mount(&mut self, base: &'static str, routes: Vec<Route>) -> &mut Self {
        println!("🛰  {} '{}':", Magenta.paint("Mounting"), Blue.paint(base));
        for mut route in routes {
            let path = format!("{}/{}", base, route.path.as_str());
            route.set_path(path);

            println!("\t* {}", route);
            self.router.add(route);
        }

        self
    }

    pub fn catch(&mut self, catchers: Vec<Catcher>) -> &mut Self {
        println!("👾  {}:", Magenta.paint("Catchers"));
        for c in catchers {
            if self.catchers.contains_key(&c.code) &&
                    !self.catchers.get(&c.code).unwrap().is_default() {
                let msg = format!("warning: overrides {} catcher!", c.code);
                println!("\t* {} ({})", c, Yellow.paint(msg.as_str()));
            } else {
                println!("\t* {}", c);
            }

            self.catchers.insert(c.code, c);
        }

        self
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

    pub fn mount_and_launch(mut self, base: &'static str, routes: Vec<Route>) {
        self.mount(base, routes);
        self.launch();
    }
}
