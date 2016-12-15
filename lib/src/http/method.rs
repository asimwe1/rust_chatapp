use std::fmt;
use std::str::FromStr;

use error::Error;
use http::hyper;
use self::Method::*;

// TODO: Support non-standard methods, here and in codegen.

/// Representation of HTTP methods.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Method {
    Get,
    Put,
    Post,
    Delete,
    Options,
    Head,
    Trace,
    Connect,
    Patch
}

impl Method {
    #[doc(hidden)]
    pub fn from_hyp(method: &hyper::Method) -> Option<Method> {
        match *method {
            hyper::Method::Get => Some(Get),
            hyper::Method::Put => Some(Put),
            hyper::Method::Post => Some(Post),
            hyper::Method::Delete => Some(Delete),
            hyper::Method::Options => Some(Options),
            hyper::Method::Head => Some(Head),
            hyper::Method::Trace => Some(Trace),
            hyper::Method::Connect => Some(Connect),
            hyper::Method::Patch => Some(Patch),
            hyper::Method::Extension(_) => None,
        }
    }

    #[doc(hidden)]
    #[inline(always)]
    pub fn to_hyp(&self) -> hyper::Method {
        self.to_string().as_str().parse().unwrap()
    }

    /// Whether an HTTP request with the given method supports a payload.
    pub fn supports_payload(&self) -> bool {
        match *self {
            Put | Post | Delete | Patch => true,
            Get | Head | Connect | Trace | Options => false,
        }
    }
}

impl FromStr for Method {
    type Err = Error;

    fn from_str(s: &str) -> Result<Method, Error> {
        match s {
            "GET" | "get" => Ok(Get),
            "PUT" | "put" => Ok(Put),
            "POST" | "post" => Ok(Post),
            "DELETE" | "delete" => Ok(Delete),
            "OPTIONS" | "options" => Ok(Options),
            "HEAD" | "head" => Ok(Head),
            "TRACE" | "trace" => Ok(Trace),
            "CONNECT" | "connect" => Ok(Connect),
            "PATCH" | "patch" => Ok(Patch),
            _ => Err(Error::BadMethod),
        }
    }
}

impl fmt::Display for Method {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(match *self {
            Get => "GET",
            Put => "PUT",
            Post => "POST",
            Delete => "DELETE",
            Options => "OPTIONS",
            Head => "HEAD",
            Trace => "TRACE",
            Connect => "CONNECT",
            Patch => "PATCH",
        })
    }
}
