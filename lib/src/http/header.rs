use std::collections::HashMap;
use std::borrow::{Borrow, Cow};
use std::fmt;

use http::hyper::header as hyper;

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

impl<T> From<T> for Header<'static> where T: hyper::Header + hyper::HeaderFormat {
    fn from(hyper_header: T) -> Header<'static> {
        let formatter = hyper::HeaderFormatter(&hyper_header);
        Header::new(T::header_name(), format!("{}", formatter))
    }
}

pub struct HeaderMap<'h> {
    headers: HashMap<Cow<'h, str>, Vec<Cow<'h, str>>>
}

impl<'h> HeaderMap<'h> {
    #[inline(always)]
    pub fn new() -> HeaderMap<'h> {
        HeaderMap { headers: HashMap::new() }
    }

    #[inline]
    pub fn contains(&self, name: &str) -> bool {
        self.headers.get(name).is_some()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.headers.iter().flat_map(|(_, values)| values.iter()).count()
    }

    #[inline]
    pub fn get<'a>(&'a self, name: &str) -> impl Iterator<Item=&'a str> {
        self.headers.get(name).into_iter().flat_map(|values| {
            values.iter().map(|val| val.borrow())
        })
    }

    #[inline]
    pub fn get_one<'a>(&'a self, name: &str) -> Option<&'a str> {
        self.headers.get(name).and_then(|values| {
            if values.len() >= 1 { Some(values[0].borrow()) }
            else { None }
        })
    }

    #[inline(always)]
    pub fn replace<'p: 'h, H: Into<Header<'p>>>(&mut self, header: H) -> bool {
        let header = header.into();
        self.headers.insert(header.name, vec![header.value]).is_some()
    }

    #[inline(always)]
    pub fn replace_raw<'a: 'h, 'b: 'h, N, V>(&mut self, name: N, value: V) -> bool
        where N: Into<Cow<'a, str>>, V: Into<Cow<'b, str>>
    {
        self.replace(Header::new(name, value))
    }

    #[inline(always)]
    pub fn replace_all<'n, 'v: 'h, H>(&mut self, name: H, values: Vec<Cow<'v, str>>)
        where 'n: 'h, H: Into<Cow<'n, str>>
    {
        self.headers.insert(name.into(), values);
    }

    #[inline(always)]
    pub fn add<'p: 'h, H: Into<Header<'p>>>(&mut self, header: H) {
        let header = header.into();
        self.headers.entry(header.name).or_insert(vec![]).push(header.value);
    }

    #[inline(always)]
    pub fn add_raw<'a: 'h, 'b: 'h, N, V>(&mut self, name: N, value: V)
        where N: Into<Cow<'a, str>>, V: Into<Cow<'b, str>>
    {
        self.add(Header::new(name, value))
    }

    #[inline(always)]
    pub fn add_all<'n, H>(&mut self, name: H, values: &mut Vec<Cow<'h, str>>)
        where 'n:'h, H: Into<Cow<'n, str>>
    {
        self.headers.entry(name.into()).or_insert(vec![]).append(values)
    }

    #[inline(always)]
    pub fn remove(&mut self, name: &str) {
        self.headers.remove(name);
    }

    #[inline(always)]
    pub fn remove_all(&mut self) -> Vec<Header<'h>> {
        let old_map = ::std::mem::replace(self, HeaderMap::new());
        old_map.into_iter().collect()
    }

    pub fn iter<'s>(&'s self) -> impl Iterator<Item=Header<'s>> {
        self.headers.iter().flat_map(|(key, values)| {
            values.iter().map(move |val| {
                Header::new(key.borrow(), val.borrow())
            })
        })
    }

    #[inline(always)]
    pub fn into_iter(self) -> impl Iterator<Item=Header<'h>> {
        self.headers.into_iter().flat_map(|(name, value)| {
            value.into_iter().map(move |value| {
                Header {
                    name: name.clone(),
                    value: value
                }
            })
        })
    }

    #[doc(hidden)]
    #[inline(always)]
    pub fn into_iter_raw<'s>(self)
            -> impl Iterator<Item=(Cow<'h, str>, Vec<Cow<'h, str>>)> {
        self.headers.into_iter()
    }
}
