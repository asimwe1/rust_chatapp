use std::str::FromStr;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddrV4, SocketAddrV6, SocketAddr};

use error::Error;

pub trait FromParam<'a>: Sized {
    fn from_param(param: &'a str) -> Result<Self, Error>;
}

impl<'a> FromParam<'a> for &'a str {
    fn from_param(param: &'a str) -> Result<&'a str, Error> {
        Ok(param)
    }
}

macro_rules! impl_with_fromstr {
    ($($T:ident),+) => ($(
        impl<'a> FromParam<'a> for $T {
            fn from_param(param: &'a str) -> Result<Self, Error> {
                $T::from_str(param).map_err(|_| Error::BadParse)
            }
        }
    )+)
}

impl_with_fromstr!(f32, f64, isize, i8, i16, i32, i64, usize, u8, u16, u32, u64,
       bool, String, IpAddr, Ipv4Addr, Ipv6Addr, SocketAddrV4, SocketAddrV6,
       SocketAddr);
