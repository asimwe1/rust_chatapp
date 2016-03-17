use error::Error;
use param::FromParam;

pub struct Request;

impl Request {
    pub fn empty() -> Request {
        Request
    }

    pub fn get_param_str<'a>(&self, name: &'a str) -> Result<&'a str, Error> {
        Err(Error::NoKey)
    }

    pub fn get_param<'b, T: FromParam<'b>>(&self, name: &'b str)
            -> Result<T, Error> {
        self.get_param_str(name).and_then(T::from_param)
    }
}
