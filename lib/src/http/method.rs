use std::fmt;
use std::str::FromStr;

use error::Error;
use http::hyper::HyperMethod;
use self::Method::*;

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
    pub fn from_hyp(method: &HyperMethod) -> Option<Method> {
        match *method {
            HyperMethod::Get => Some(Get),
            HyperMethod::Put => Some(Put),
            HyperMethod::Post => Some(Post),
            HyperMethod::Delete => Some(Delete),
            HyperMethod::Options => Some(Options),
            HyperMethod::Head => Some(Head),
            HyperMethod::Trace => Some(Trace),
            HyperMethod::Connect => Some(Connect),
            HyperMethod::Patch => Some(Patch),
            HyperMethod::Extension(_) => None,
        }
    }

    #[doc(hidden)]
    #[inline(always)]
    pub fn to_hyp(&self) -> HyperMethod {
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
