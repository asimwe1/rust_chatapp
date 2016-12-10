use std::fmt;
use std::borrow::Cow;

use rocket::request::FromParam;
use rand::{self, Rng};

const BASE62: &'static [u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

fn valid_id(id: &str) -> bool {
    id.chars().all(|c| {
        (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || (c >= '0' && c <= '9')
    })
}

pub struct PasteID<'a>(Cow<'a, str>);

impl<'a> PasteID<'a> {
    /// Here's how this works: get `size` random 8-bit integers, convert them
    /// into base-62 characters, and concat them to get the ID.
    pub fn new(size: usize) -> PasteID<'static> {
        let mut id = String::with_capacity(size);
        let mut rng = rand::thread_rng();
        for _ in 0..size {
            id.push(BASE62[rng.gen::<usize>() % 62] as char);
        }

        PasteID(Cow::Owned(id))
    }
}

impl<'a> FromParam<'a> for PasteID<'a> {
    type Error = &'a str;

    fn from_param(param: &'a str) -> Result<PasteID<'a>, &'a str> {
        match valid_id(param) {
            true => Ok(PasteID(Cow::Borrowed(param))),
            false => Err(param)
        }
    }
}

impl<'a> fmt::Display for PasteID<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

