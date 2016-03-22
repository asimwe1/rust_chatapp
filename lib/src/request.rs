use error::Error;
use param::FromParam;

pub struct Request<'a> {
    params: Vec<&'a str>,
    uri: &'a str,
}

impl<'a> Request<'a> {
    pub fn empty() -> Request<'static> {
        Request::new(vec![], "")
    }

    pub fn new(params: Vec<&'a str>, uri: &'a str) -> Request<'a> {
        Request {
            params: params,
            uri: uri
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
