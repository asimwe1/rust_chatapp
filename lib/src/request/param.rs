use std::str::FromStr;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddrV4, SocketAddrV6, SocketAddr};
use std::path::PathBuf;

use router::Segments;
use url;

pub trait FromParam<'a>: Sized {
    type Error;
    fn from_param(param: &'a str) -> Result<Self, Self::Error>;
}

impl<'a> FromParam<'a> for &'a str {
    type Error = ();
    fn from_param(param: &'a str) -> Result<&'a str, Self::Error> {
        Ok(param)
    }
}

impl<'a> FromParam<'a> for String {
    type Error = &'a str;
    fn from_param(p: &'a str) -> Result<String, Self::Error> {
        let decoder = url::percent_encoding::percent_decode(p.as_bytes());
        decoder.decode_utf8().map_err(|_| p).map(|s| s.into_owned())
    }
}

macro_rules! impl_with_fromstr {
    ($($T:ident),+) => ($(
        impl<'a> FromParam<'a> for $T {
            type Error = &'a str;
            fn from_param(param: &'a str) -> Result<Self, Self::Error> {
                $T::from_str(param).map_err(|_| param)
            }
        }
    )+)
}

impl_with_fromstr!(f32, f64, isize, i8, i16, i32, i64, usize, u8, u16, u32, u64,
       bool, IpAddr, Ipv4Addr, Ipv6Addr, SocketAddrV4, SocketAddrV6,
       SocketAddr);

pub trait FromSegments<'a>: Sized {
    type Error;
    fn from_segments(segments: Segments<'a>) -> Result<Self, Self::Error>;
}

impl<'a> FromSegments<'a> for Segments<'a> {
    type Error = ();
    fn from_segments(segments: Segments<'a>) -> Result<Segments<'a>, ()> {
        Ok(segments)
    }
}

impl<'a> FromSegments<'a> for PathBuf {
    type Error = ();
    fn from_segments(segments: Segments<'a>) -> Result<PathBuf, ()> {
        Ok(segments.collect())
    }
}
