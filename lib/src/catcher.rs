use handler::ErrorHandler;
use response::Response;
use codegen::StaticCatchInfo;
use error::Error;
use request::Request;

use std::fmt;
use term_painter::ToStyle;
use term_painter::Color::*;

pub struct Catcher {
    pub code: u16,
    handler: ErrorHandler,
    is_default: bool,
}

impl Catcher {
    pub fn new(code: u16, handler: ErrorHandler) -> Catcher {
        Catcher { code: code, handler: handler, is_default: false }
    }

    pub fn handle<'r>(&self, err: Error, request: &'r Request) -> Response<'r> {
        (self.handler)(err, request)
    }

    fn new_default(code: u16, handler: ErrorHandler) -> Catcher {
        Catcher { code: code, handler: handler, is_default: true, }
    }

    pub fn is_default(&self) -> bool {
        self.is_default
    }
}

impl<'a> From<&'a StaticCatchInfo> for Catcher {
    fn from(info: &'a StaticCatchInfo) -> Catcher {
        Catcher::new(info.code, info.handler)
    }
}

impl fmt::Display for Catcher {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", Blue.paint(&self.code))
    }
}

macro_rules! error_page_template {
    ($code:expr, $name:expr, $description:expr) => (
        concat!(r#"
            <!DOCTYPE html>
            <html>
            <head>
                <meta charset="utf-8">
                <title>"#, $code, " ", $name, r#"</title>
            </head>
            <body align="center">
                <div align="center">
                    <h1>"#, $code, ": ", $name, r#"</h1>
                    <p>"#, $description, r#"</p>
                    <hr />
                    <small>Rocket</small>
                </div>
            </body>
            </html>
        "#
        )
    )
}

macro_rules! default_errors {
    ($($code:expr, $name:expr, $description:expr, $fn_name:ident),+) => (
        let mut map = HashMap::new();

        $(
            fn $fn_name<'r>(_: Error, _r: &'r Request) -> Response<'r> {
                Response::with_raw_status($code,
                    content::HTML(error_page_template!($code, $name, $description))
                )
            }

            map.insert($code, Catcher::new_default($code, $fn_name));
        )+

        map
    )
}

pub mod defaults {
    use super::Catcher;

    use std::collections::HashMap;

    use request::Request;
    use response::{Response, content};
    use error::Error;

    pub fn get() -> HashMap<u16, Catcher> {
        default_errors! {
            400, "Bad Request", "The request could not be understood by the server due
                to malformed syntax.", handle_400,
            401, "Unauthorized", "The request requires user authentication.",
                handle_401,
            403, "Forbidden", "The request was forbidden by the server.
                Check authentication.", handle_403,
            402, "Payment Required", "The request could not be processed due to lack of
                payment.", handle_402,
            404, "Not Found", "The requested resource could not be found.", handle_404,
            405, "Method Not Allowed", "The request method is not supported for the
                requested resource.", handle_405,
            406, "Not Acceptable", "The requested resource is capable of generating
                only content not acceptable according to the Accept headers sent in the
                request.", handle_406,
            407, "Proxy Authentication Required", "Authentication with the proxy is
                required.", handle_407,
            408, "Request Timeout", "The server timed out waiting for the
                request.", handle_408,
            409, "Conflict", "The request could not be processed because of a conflict
                in the request.", handle_409,
            410, "Gone", "The resource requested is no longer available and will not be
                available again.", handle_410,
            411, "Length Required", "The request did not specify the length of its
                content, which is required by the requested resource.", handle_411,
            412, "Precondition Failed", "The server does not meet one of the
                preconditions specified in the request.", handle_412,
            413, "Payload Too Large", "The request is larger than the server is
                willing or able to process.", handle_413,
            414, "URI Too Long", "The URI provided was too long for the server to
                process.", handle_414,
            415, "Unsupported Media Type", "The request entity has a media type which
                the server or resource does not support.", handle_415,
            416, "Range Not Satisfiable", "The portion of the requested file cannot be
                supplied by the server.", handle_416,
            417, "Expectation Failed", "The server cannot meet the requirements of the
                Expect request-header field.", handle_417,
            418, "I'm a teapot", "I was requested to brew coffee, and I am a
                teapot.", handle_418,
            421, "Misdirected Request", "The server cannot produce a response for this
                request.", handle_421,
            426, "Upgrade Required", "Switching to the protocol in the Upgrade header
                field is required.", handle_426,
            428, "Precondition Required", "The server requires the request to be
               conditional.", handle_428,
            429, "Too Many Requests", "Too many requests have been received
                recently.", handle_429,
            431, "Request Header Fields Too Large", "The server is unwilling to process
                the request because either an individual header field, or all
                the header fields collectively, are too large.", handle_431,
            451, "Unavailable For Legal Reasons", "The requested resource is
                unavailable due to a legal demand to deny access to this
                resource.", handle_451,
            500, "Internal Server Error", "The server encountered an internal error
                while processing this request.", handle_500,
            501, "Not Implemented", "The server either does not recognize the request
                method, or it lacks the ability to fulfill the request.", handle_501,
            503, "Service Unavailable", "The server is currently unavailable.",
                handle_503,
            510, "Not Extended", "Further extensions to the request are required for
                the server to fulfill it.", handle_510
        }
    }
}

