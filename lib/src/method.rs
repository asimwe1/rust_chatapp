use super::*;
use self::Method::*;

use std::fmt;
use std::str::FromStr;
use hyper::method::Method as HyperMethod;

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
            _ => None
        }
    }
}

impl FromStr for Method {
    type Err = Error;

    fn from_str(s: &str) -> Result<Method, Error> {
        match s {
            "GET" => Ok(Get),
            "PUT" => Ok(Put),
            "POST" => Ok(Post),
            "DELETE" => Ok(Delete),
            "OPTIONS" => Ok(Options),
            "HEAD" => Ok(Head),
            "TRACE" => Ok(Trace),
            "CONNECT" => Ok(Connect),
            "PATCH" => Ok(Patch),
            _ => Err(Error::BadMethod)
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
