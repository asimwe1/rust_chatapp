use std::borrow::Cow;
use http::hyper::header::Header as HyperHeader;
use http::hyper::header::HeaderFormat as HyperHeaderFormat;
use http::hyper::header::HeaderFormatter as HyperHeaderFormatter;
use std::fmt;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Header<'h> {
    pub name: Cow<'h, str>,
    pub value: Cow<'h, str>,
}

impl<'h> Header<'h> {
    #[inline(always)]
    pub fn new<'a: 'h, 'b: 'h, N, V>(name: N, value: V) -> Header<'h>
        where N: Into<Cow<'a, str>>, V: Into<Cow<'b, str>>
    {
        Header {
            name: name.into(),
            value: value.into()
        }
    }
}

impl<'h> fmt::Display for Header<'h> {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.value)
    }
}

impl<T> From<T> for Header<'static> where T: HyperHeader + HyperHeaderFormat {
    fn from(hyper_header: T) -> Header<'static> {
        let formatter = HyperHeaderFormatter(&hyper_header);
        Header::new(T::header_name(), format!("{}", formatter))
    }
}
