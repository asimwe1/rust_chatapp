use super::*;
use response::FreshHyperResponse;
use request::HyperRequest;
use catcher;

use std::io::Read;
use std::collections::HashMap;
use term_painter::Color::*;
use term_painter::ToStyle;

use hyper::uri::RequestUri as HyperRequestUri;
use hyper::method::Method as HyperMethod;
use hyper::server::Server as HyperServer;
use hyper::server::Handler as HyperHandler;

pub struct Rocket {
    address: &'static str,
    port: isize,
    router: Router,
    catchers: HashMap<u16, Catcher>,
}

fn uri_is_absolute(uri: &HyperRequestUri) -> bool {
    match *uri {
        HyperRequestUri::AbsolutePath(_) => true,
        _ => false
    }
}

fn unwrap_absolute_path(uri: &HyperRequestUri) -> &str {
    match *uri {
        HyperRequestUri::AbsolutePath(ref s) => s.as_str(),
        _ => panic!("Can only accept absolute paths!")
    }
}

fn method_is_valid(method: &HyperMethod) -> bool {
    Method::from_hyp(method).is_some()
}

impl HyperHandler for Rocket {
    fn handle<'a, 'k>(&'a self, req: HyperRequest<'a, 'k>,
            res: FreshHyperResponse<'a>) {
        println!("{:?} '{}'", Green.paint(&req.method), Blue.paint(&req.uri));

        let finalize = |mut req: HyperRequest, _res: FreshHyperResponse| {
            let mut buf = vec![];
            // FIXME: Simple DOS attack here. Working around Hyper bug.
            let _ = req.read_to_end(&mut buf);
        };

        if !uri_is_absolute(&req.uri) {
            println!("{}", Red.paint("\t=> Internal failure. Bad URI."));
            println!("{} {:?}", Yellow.paint("\t=> Debug:"), req.uri);
            return finalize(req, res);
        }

        if !method_is_valid(&req.method) {
            println!("{}", Yellow.paint("\t=> Internal failure. Bad method."));
            println!("{} {:?}", Yellow.paint("\t=> Debug:"), req.method);
            return finalize(req, res);
        }

        self.dispatch(req, res)
    }
}

impl Rocket {
    fn dispatch<'h, 'k>(&self, mut req: HyperRequest<'h, 'k>,
                        res: FreshHyperResponse<'h>) {
        // We read all of the contents now because we have to do it at some
        // point thanks to Hyper. FIXME: Simple DOS attack here.
        let mut buf = vec![];
        let _ = req.read_to_end(&mut buf);

        // Extract the method, uri, and try to find a route.
        let method = Method::from_hyp(&req.method).unwrap();
        let uri = unwrap_absolute_path(&req.uri);
        let route = self.router.route(method, uri);

        // A closure which we call when we know there is no route.
        let handle_not_found = |response: FreshHyperResponse| {
            let request = Request::new(vec![], uri, &buf);
            let handler_404 = self.catchers.get(&404).unwrap().handler;
            println!("{}", Red.paint("\t<= Dispatch failed. Returning 404."));
            handler_404(request).respond(response);
        };

        // No route found. Handle the not_found error and return.
        if route.is_none() {
            println!("{}", Red.paint("\t=> No matching routes."));
            return handle_not_found(res);
        }

        // Okay, we've got a route. Unwrap it, generate a request, and try to
        // dispatch. TODO: keep trying lower ranked routes before dispatching a
        // not found error.
        println!("\t=> {}", Magenta.paint("Dispatching request."));
        let route = route.unwrap();
        let params = route.get_params(uri);
        let request = Request::new(params, uri, &buf);
        let outcome = (route.handler)(request).respond(res);

        println!("\t=> {} {}", White.paint("Outcome:"), outcome);
        outcome.map_forward(|res| {
            println!("{}", Red.paint("\t=> No further matching routes."));
            handle_not_found(res);
        });
    }

    pub fn new(address: &'static str, port: isize) -> Rocket {
        Rocket {
            address: address,
            port: port,
            router: Router::new(),
            catchers: catcher::defaults::get(),
        }
    }

    pub fn mount(&mut self, base: &'static str, routes: Vec<Route>)
            -> &mut Self {
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
