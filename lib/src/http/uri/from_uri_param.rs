use std::path::{Path, PathBuf};

use http::RawStr;
use http::uri::UriDisplay;

pub trait FromUriParam<T>: UriDisplay {
    type Target: UriDisplay;
    fn from_uri_param(param: T) -> Self::Target;
}

impl<T: UriDisplay> FromUriParam<T> for T {
    type Target = T;
    #[inline(always)]
    fn from_uri_param(param: T) -> T { param }
}

impl<'a, T: UriDisplay> FromUriParam<&'a T> for T {
    type Target = &'a T;
    #[inline(always)]
    fn from_uri_param(param: &'a T) -> &'a T { param }
}

impl<'a, T: UriDisplay> FromUriParam<&'a mut T> for T {
    type Target = &'a mut T;
    #[inline(always)]
    fn from_uri_param(param: &'a mut T) -> &'a mut T { param }
}

impl<'a> FromUriParam<&'a str> for String {
    type Target = &'a str;
    #[inline(always)]
    fn from_uri_param(param: &'a str) -> &'a str { param }
}

impl<'a, 'b> FromUriParam<&'a str> for &'b RawStr {
    type Target = &'a str;
    #[inline(always)]
    fn from_uri_param(param: &'a str) -> &'a str { param }
}

impl<'a> FromUriParam<&'a Path> for PathBuf {
    type Target = &'a Path;
    #[inline(always)]
    fn from_uri_param(param: &'a Path) -> &'a Path { param }
}

impl<'a> FromUriParam<&'a str> for PathBuf {
    type Target = &'a Path;
    #[inline(always)]
    fn from_uri_param(param: &'a str) -> &'a Path {
        Path::new(param)
    }
}
