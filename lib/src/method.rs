use self::Method::*;
use std::str::FromStr;
use std::fmt::{self, Display};
use error::Error;
use hyper::method::Method as HypMethod;

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
    pub fn from_hyp(method: HypMethod) -> Option<Method> {
        match method {
            HypMethod::Get => Some(Get),
            HypMethod::Put => Some(Put),
            HypMethod::Post => Some(Post),
            HypMethod::Delete => Some(Delete),
            HypMethod::Options => Some(Options),
            HypMethod::Head => Some(Head),
            HypMethod::Trace => Some(Trace),
            HypMethod::Connect => Some(Connect),
            HypMethod::Patch => Some(Patch),
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

impl Display for Method {
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
