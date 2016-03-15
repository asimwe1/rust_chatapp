pub use hyper::status::StatusCode;
pub use hyper::header::{self, Headers};
use std::io::Read;

pub enum Body<'a> {
    Bytes(&'a [u8]),
    Str(&'a str),
    String(String),
    Stream(Box<Read>),
    Empty
}

pub struct Response<'a> {
    pub status: StatusCode,
    pub headers: Headers,
    pub body: Body<'a>
}

impl<'a> Response<'a> {
    pub fn empty() -> Response<'a> {
        Response {
            status: StatusCode::Ok,
            headers: Headers::new(),
            body: Body::Empty
        }
    }

    pub fn not_found() -> Response<'a> {
        Response {
            status: StatusCode::NotFound,
            headers: Headers::new(),
            body: Body::Empty
        }
    }

    pub fn server_error() -> Response<'a> {
        Response {
            status: StatusCode::InternalServerError,
            headers: Headers::new(),
            body: Body::Empty
        }
    }
}

impl<'a> From<&'a str> for Response<'a> {
    fn from(s: &'a str) -> Self {
        let mut headers = Headers::new();
        headers.set(header::ContentLength(s.len() as u64));
        Response {
            status: StatusCode::Ok,
            headers: headers,
            body: Body::Str(s)
        }
    }
}

impl<'a> From<String> for Response<'a> {
    fn from(s: String) -> Self {
        let mut headers = Headers::new();
        headers.set(header::ContentLength(s.len() as u64));
        Response {
            status: StatusCode::Ok,
            headers: headers,
            body: Body::String(s)
        }
    }
}
