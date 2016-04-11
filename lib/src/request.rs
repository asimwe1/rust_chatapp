use error::Error;
use param::FromParam;

pub use hyper::server::Request as HyperRequest;

#[derive(Clone)]
pub struct Request<'a> {
    params: Vec<&'a str>,
    pub uri: &'a str,
    pub data: &'a [u8]
}

impl<'a> Request<'a> {
    pub fn new(params: Vec<&'a str>, uri: &'a str, data: &'a [u8]) -> Request<'a> {
        Request {
            params: params,
            uri: uri,
            data: data
        }
    }

    pub fn get_uri(&self) -> &'a str {
        self.uri
    }

    pub fn get_param<T: FromParam<'a>>(&'a self, n: usize) -> Result<T, Error> {
        if n >= self.params.len() {
            Err(Error::NoKey)
        } else {
            T::from_param(self.params[n])
        }
    }
}
