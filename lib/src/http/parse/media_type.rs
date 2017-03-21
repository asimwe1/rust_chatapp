use std::borrow::Cow;

use pear::ParseResult;
use pear::parsers::*;
use pear::combinators::*;
use smallvec::SmallVec;

use http::{MediaType, MediaParams};
use http::parse::checkers::{is_whitespace, is_valid_token};
use http::parse::IndexedStr;

#[parser]
fn quoted_string<'a>(input: &mut &'a str) -> ParseResult<&'a str, &'a str> {
    eat('"');

    let mut is_escaped = false;
    let inner = take_while(|c| {
        if is_escaped { is_escaped = false; return true; }
        if c == '\\' { is_escaped = true; return true; }
        c != '"'
    });

    eat('"');
    inner
}

macro_rules! switch_repeat {
    ($input:expr, $($cases:tt)*) => (repeat!($input, switch!($($cases)*)))
}

#[parser]
fn media_type<'a>(input: &mut &'a str,
                  source: &'a str) -> ParseResult<&'a str, MediaType> {
    let top = take_some_while(|c| is_valid_token(c) && c != '/');
    eat('/');
    let sub = take_some_while(is_valid_token);

    let mut params = SmallVec::new();
    switch_repeat! {
        surrounded(|i| eat(i, ';'), is_whitespace) => {
            skip_while(is_whitespace);
            let key = take_some_while(|c| is_valid_token(c) && c != '=');
            eat('=');

            let value = switch! {
                peek('"') => quoted_string(),
                _ => take_some_while(|c| is_valid_token(c) && c != ';')
            };

            let indexed_key = IndexedStr::from(key, source).expect("key");
            let indexed_val = IndexedStr::from(value, source).expect("val");
            params.push((indexed_key, indexed_val))
        },
        _ => break
    }

    MediaType {
        source: Some(Cow::Owned(source.to_string())),
        top: IndexedStr::from(top, source).expect("top in source"),
        sub: IndexedStr::from(sub, source).expect("sub in source"),
        params: MediaParams::Dynamic(params)
    }
}

pub fn parse_media_type(mut input: &str) -> ParseResult<&str, MediaType> {
    parse!(&mut input, (media_type(input), eof()).0)
}
