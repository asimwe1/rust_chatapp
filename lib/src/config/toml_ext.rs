use std::fmt;
use std::collections::BTreeMap;

use config::Value;

use pear::{ParseResult, ParseError};
use pear::parsers::*;
use pear::combinators::*;

#[inline(always)]
pub fn is_whitespace(byte: char) -> bool {
    byte == ' ' || byte == '\t'
}

#[inline(always)]
pub fn is_number_token(byte: char) -> bool {
    match byte {
        '0'...'9' | '.' | '-' => true,
        _ => false
    }
}

// FIXME: Silence warning for `digits.parse`.
#[parser]
fn number<'a>(input: &mut &'a str) -> ParseResult<&'a str, Value> {
    let digits = take_some_while(is_number_token);
    if let Ok(int) = digits.parse::<i64>() {
        Value::Integer(int)
    } else {
        let v = from!(digits.parse::<f64>());
        Value::Float(v)
    }
}

#[parser]
fn array<'a>(input: &mut &'a str) -> ParseResult<&'a str, Value> {
    let array = (eat('['), collect!(value(), eat(',')), eat(']')).1;
    Value::Array(array)
}

// FIXME: Be more permissive here?
#[parser]
fn key<'a>(input: &mut &'a str) -> ParseResult<&'a str, String> {
    take_some_while(|c| match c {
        '0'...'9' | 'A'...'Z' | 'a'...'z' | '_' | '-' => true,
        _ => false
    }).to_string()
}

#[parser]
fn table<'a>(input: &mut &'a str) -> ParseResult<&'a str, Value> {
    eat('{');

    let mut values = BTreeMap::new();
    try_repeat_while!(eat(','), {
        let key = surrounded(key, is_whitespace);
        (eat('='), skip_while(is_whitespace));
        values.insert(key, value())
    });

    eat('}');
    Value::Table(values)
}

#[parser]
fn value<'a>(input: &mut &'a str) -> ParseResult<&'a str, Value> {
    skip_while(is_whitespace);
    let val = switch! {
        eat_slice("true") => Value::Boolean(true),
        eat_slice("false") => Value::Boolean(false),
        peek('{') => table(),
        peek('[') => array(),
        peek_if(is_number_token) => number(),
        peek('"') => Value::String(delimited('"', |_| true, '"').to_string()),
        _ => Value::String(take_some_while(|c| c != ',' && c != '}' && c != ']').to_string())
    };

    skip_while(is_whitespace);
    val
}

pub fn parse_simple_toml_value(mut input: &str) -> Result<Value, String> {
    let result: Result<Value, ParseError<&str>> = parse!(&mut input, (value(), eof()).0).into();
    result.map_err(|e| e.to_string())
}

/// A simple wrapper over a `Value` reference with a custom implementation of
/// `Display`. This is used to log config values at initialization.
pub(crate) struct LoggedValue<'a>(pub &'a Value);

impl<'a> fmt::Display for LoggedValue<'a> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use config::Value::*;
        match *self.0 {
            String(_) | Integer(_) | Float(_) | Boolean(_) | Datetime(_) | Array(_) => {
                self.0.fmt(f)
            }
            Table(ref map) => {
                write!(f, "{{ ")?;
                for (i, (key, val)) in map.iter().enumerate() {
                    write!(f, "{} = {}", key, LoggedValue(val))?;
                    if i != map.len() - 1 { write!(f, ", ")?; }
                }

                write!(f, " }}")
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;
    use super::parse_simple_toml_value;
    use super::Value::{self, *};

    macro_rules! assert_parse {
        ($string:expr, $value:expr) => (
            match parse_simple_toml_value($string) {
                Ok(value) => assert_eq!(value, $value),
                Err(e) => panic!("{:?} failed to parse: {:?}", $string, e)
            };
        )
    }

    #[test]
    fn parse_toml_values() {
        assert_parse!("1", Integer(1));
        assert_parse!("1.32", Float(1.32));
        assert_parse!("true", Boolean(true));
        assert_parse!("false", Boolean(false));
        assert_parse!("\"hello, WORLD!\"", String("hello, WORLD!".into()));
        assert_parse!("hi", String("hi".into()));
        assert_parse!("\"hi\"", String("hi".into()));

        assert_parse!("[]", Array(Vec::new()));
        assert_parse!("[1]", vec![1].into());
        assert_parse!("[1, 2, 3]", vec![1, 2, 3].into());
        assert_parse!("[1.32, 2]", Array(vec![1.32.into(), 2.into()]));

        assert_parse!("{}", Table(BTreeMap::new()));

        assert_parse!("{a=b}", Table({
            let mut map = BTreeMap::new();
            map.insert("a".into(), "b".into());
            map
        }));

        assert_parse!("{v=1, on=true,pi=3.14}", Table({
            let mut map = BTreeMap::new();
            map.insert("v".into(), 1.into());
            map.insert("on".into(), true.into());
            map.insert("pi".into(), 3.14.into());
            map
        }));

        assert_parse!("{v=[1, 2, 3], v2=[a, \"b\"], on=true,pi=3.14}", Table({
            let mut map = BTreeMap::new();
            map.insert("v".into(), vec![1, 2, 3].into());
            map.insert("v2".into(), vec!["a", "b"].into());
            map.insert("on".into(), true.into());
            map.insert("pi".into(), 3.14.into());
            map
        }));

        assert_parse!("{v=[[1], [2, 3], [4,5]]}", Table({
            let mut map = BTreeMap::new();
            let first: Value = vec![1].into();
            let second: Value = vec![2, 3].into();
            let third: Value = vec![4, 5].into();
            map.insert("v".into(), vec![first, second, third].into());
            map
        }));
    }
}
