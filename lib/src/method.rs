use self::Method::{Options, Get, Post, Put, Delete, Head, Trace, Connect, Patch};
use std::str::FromStr;
use std::fmt::{self, Display};
use error::Error;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
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

impl FromStr for Method {
    type Err = Error;

    fn from_str(s: &str) -> Result<Method, Error> {
        match s {
            "OPTIONS" => Ok(Options),
            "GET" => Ok(Get),
            "POST" => Ok(Post),
            "PUT" => Ok(Put),
            "DELETE" => Ok(Delete),
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
            Options => "OPTIONS",
            Get => "GET",
            Post => "POST",
            Put => "PUT",
            Delete => "DELETE",
            Head => "HEAD",
            Trace => "TRACE",
            Connect => "CONNECT",
            Patch => "PATCH",
        })
    }
}
