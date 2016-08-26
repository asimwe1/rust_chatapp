use ::{Method, Handler, ErrorHandler};
use content_type::ContentType;

pub struct StaticRouteInfo {
    pub method: Method,
    pub path: &'static str,
    pub content_type: ContentType,
    pub handler: Handler,
}

pub struct StaticCatchInfo {
    pub code: u16,
    pub handler: ErrorHandler
}

