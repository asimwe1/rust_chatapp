use std::str::FromStr;
use error::Error;

pub struct Request;

impl Request {
    pub fn empty() -> Request {
        Request
    }

    pub fn get_param_str(&self, name: &str) -> Result<&str, Error> {
        Err(Error::NoKey)
    }

    pub fn get_param<T: FromStr>(&self, name: &str) -> Result<T, Error> {
        self.get_param_str(name).and_then(|s| {
            T::from_str(s).map_err(|_| Error::BadParse)
        })
    }
}
