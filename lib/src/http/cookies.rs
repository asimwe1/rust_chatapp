use http::Header;

use std::cell::RefMut;

pub use cookie::{Cookie, CookieJar, Iter, CookieBuilder, Delta};

#[derive(Debug)]
pub enum Cookies<'a> {
    Jarred(RefMut<'a, CookieJar>),
    Empty(CookieJar)
}

impl<'a> From<RefMut<'a, CookieJar>> for Cookies<'a> {
    fn from(jar: RefMut<'a, CookieJar>) -> Cookies<'a> {
        Cookies::Jarred(jar)
    }
}

impl<'a> Cookies<'a> {
    pub(crate) fn empty() -> Cookies<'static> {
        Cookies::Empty(CookieJar::new())
    }

    pub fn get(&self, name: &str) -> Option<&Cookie<'static>> {
        match *self {
            Cookies::Jarred(ref jar) => jar.get(name),
            Cookies::Empty(_) => None
        }
    }

    pub fn add(&mut self, cookie: Cookie<'static>) {
        if let Cookies::Jarred(ref mut jar) = *self {
            jar.add(cookie)
        }
    }

    pub fn remove(&mut self, cookie: Cookie<'static>) {
        if let Cookies::Jarred(ref mut jar) = *self {
            jar.remove(cookie)
        }
    }

    pub fn iter(&self) -> Iter {
        match *self {
            Cookies::Jarred(ref jar) => jar.iter(),
            Cookies::Empty(ref jar) => jar.iter()
        }
    }

    pub fn delta(&self) -> Delta {
        match *self {
            Cookies::Jarred(ref jar) => jar.delta(),
            Cookies::Empty(ref jar) => jar.delta()
        }
    }
}

impl<'a, 'c> From<&'a Cookie<'c>> for Header<'static> {
    fn from(cookie: &Cookie) -> Header<'static> {
        Header::new("Set-Cookie", cookie.encoded().to_string())
    }
}
