use method::Method;
use handler::Handler;

pub struct StaticRouteInfo {
    pub method: Method,
    pub path: &'static str,
    pub handler: Handler
}

