use error::Error;
use param::FromParam;
use method::Method;

#[derive(Clone, Debug)]
pub struct Request<'a> {
    params: Option<Vec<&'a str>>,
    pub method: Method,
    pub uri: &'a str,
    pub data: &'a [u8]
}

impl<'a> Request<'a> {
    pub fn new(method: Method, uri: &'a str, params: Option<Vec<&'a str>>,
               data: &'a [u8]) -> Request<'a> {
        Request {
            method: method,
            params: params,
            uri: uri,
            data: data
        }
    }

    pub fn get_uri(&self) -> &'a str {
        self.uri
    }

    pub fn get_param<T: FromParam<'a>>(&'a self, n: usize) -> Result<T, Error> {
        if self.params.is_none() || n >= self.params.as_ref().unwrap().len() {
            Err(Error::NoKey)
        } else {
            T::from_param(self.params.as_ref().unwrap()[n])
        }
    }
}
