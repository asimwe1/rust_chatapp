use {Request, Data};
use handler::{Outcome, ErrorHandler};
use http::{Method, MediaType};

pub type StaticHandler = for<'r> fn(&'r Request, Data) -> Outcome<'r>;

pub struct StaticRouteInfo {
    pub name: &'static str,
    pub method: Method,
    pub path: &'static str,
    pub format: Option<MediaType>,
    pub handler: StaticHandler,
    pub rank: Option<isize>,
}

pub struct StaticCatchInfo {
    pub code: u16,
    pub handler: ErrorHandler,
}
