use ::{Method, Handler, ErrorHandler};
use content_type::ContentType;

pub struct StaticRouteInfo {
    pub method: Method,
    pub path: &'static str,
    pub accept: Option<ContentType>,
    pub handler: Handler,
    pub rank: Option<isize>,
}

pub struct StaticCatchInfo {
    pub code: u16,
    pub handler: ErrorHandler
}

