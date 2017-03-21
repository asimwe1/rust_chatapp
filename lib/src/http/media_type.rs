use std::borrow::Cow;
use std::str::FromStr;
use std::fmt;
use std::hash::{Hash, Hasher};

use http::ascii::{uncased_eq, UncasedAsciiRef};
use http::parse::{IndexedStr, parse_media_type};

use smallvec::SmallVec;

#[derive(Debug, Clone)]
struct MediaParam {
    key: IndexedStr,
    value: IndexedStr,
}

// FIXME: `Static` is needed for `const` items. Need `const SmallVec::new`.
#[derive(Debug, Clone)]
pub enum MediaParams {
    Empty,
    Static(&'static [(IndexedStr, IndexedStr)]),
    Dynamic(SmallVec<[(IndexedStr, IndexedStr); 2]>)
}

// TODO: impl PartialEq, Hash for `MediaType`.
#[derive(Debug, Clone)]
pub struct MediaType {
    /// Storage for the entire media type string. This will be `Some` when the
    /// media type was parsed from a string and `None` when it was created
    /// manually.
    #[doc(hidden)]
    pub source: Option<Cow<'static, str>>,
    /// The top-level type.
    #[doc(hidden)]
    pub top: IndexedStr,
    /// The subtype.
    #[doc(hidden)]
    pub sub: IndexedStr,
    /// The parameters, if any.
    #[doc(hidden)]
    pub params: MediaParams
}

macro_rules! media_str {
    ($string:expr) => (IndexedStr::Concrete(Cow::Borrowed($string)))
}

macro_rules! media_types {
    ($($name:ident ($check:ident): $str:expr, $t:expr,
        $s:expr $(; $k:expr => $v:expr)*),+) => {
        $(
            #[doc="[MediaType](struct.MediaType.html) for <b>"]
            #[doc=$str]
            #[doc="</b>: <i>"] #[doc=$t] #[doc="/"] #[doc=$s] #[doc="</i>"]
            #[allow(non_upper_case_globals)]
            pub const $name: MediaType = MediaType {
                source: None,
                top: media_str!($t),
                sub: media_str!($s),
                params: MediaParams::Static(&[$((media_str!($k), media_str!($v))),*])
            };

            #[inline(always)]
            pub fn $check(&self) -> bool {
                *self == MediaType::$name
            }
         )+

        /// Returns `true` if this MediaType is known to Rocket, that is,
        /// there is an associated constant for `self`.
        pub fn is_known(&self) -> bool {
            $(if self.$check() { return true })+
            false
        }
    };
}

macro_rules! from_extension {
    ($($ext:expr => $name:ident),*) => (
        pub fn from_extension(ext: &str) -> Option<MediaType> {
            match ext {
                $(x if uncased_eq(x, $ext) => Some(MediaType::$name)),*,
                _ => None
            }
        }
    )
}

impl MediaType {
    #[inline]
    pub fn new<T, S>(top: T, sub: S) -> MediaType
        where T: Into<Cow<'static, str>>, S: Into<Cow<'static, str>>
    {
        MediaType {
            source: None,
            top: IndexedStr::Concrete(top.into()),
            sub: IndexedStr::Concrete(sub.into()),
            params: MediaParams::Empty,
        }
    }

    #[inline]
    pub fn with_params<T, S, K, V, P>(top: T, sub: S, ps: P) -> MediaType
        where T: Into<Cow<'static, str>>, S: Into<Cow<'static, str>>,
              K: Into<Cow<'static, str>>, V: Into<Cow<'static, str>>,
              P: IntoIterator<Item=(K, V)>
    {
        let mut params = SmallVec::new();
        for (key, val) in ps {
            params.push((
                IndexedStr::Concrete(key.into()),
                IndexedStr::Concrete(val.into())
            ))
        }

        MediaType {
            source: None,
            top: IndexedStr::Concrete(top.into()),
            sub: IndexedStr::Concrete(sub.into()),
            params: MediaParams::Dynamic(params)
        }
    }

    known_extensions!(from_extension);

    #[inline]
    pub fn top(&self) -> &UncasedAsciiRef {
        self.top.to_str(self.source.as_ref()).into()
    }

    #[inline]
    pub fn sub(&self) -> &UncasedAsciiRef {
        self.sub.to_str(self.source.as_ref()).into()
    }

    #[inline]
    pub fn params<'a>(&'a self) -> impl Iterator<Item=(&'a str, &'a str)> + 'a {
        let param_slice = match self.params {
            MediaParams::Static(slice) => slice,
            MediaParams::Dynamic(ref vec) => &vec[..],
            MediaParams::Empty => &[]
        };

        param_slice.iter()
            .map(move |&(ref key, ref val)| {
                let source_str = self.source.as_ref();
                (key.to_str(source_str), val.to_str(source_str))
            })
    }

    #[inline(always)]
    pub fn into_owned(self) -> MediaType {
        MediaType {
            source: self.source.map(|c| c.into_owned().into()),
            top: self.top,
            sub: self.sub,
            params: self.params
        }
    }

    known_media_types!(media_types);
}

impl FromStr for MediaType {
    // Ideally we'd return a `ParseError`, but that required a lifetime.
    type Err = String;

    #[inline]
    fn from_str(raw: &str) -> Result<MediaType, String> {
        parse_media_type(raw)
            .map(|mt| mt.into_owned())
            .map_err(|e| e.to_string())
    }
}

impl PartialEq for MediaType {
    fn eq(&self, other: &MediaType) -> bool {
        self.top() == other.top() && self.sub() == other.sub()
    }
}

impl Hash for MediaType {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.top().hash(state);
        self.sub().hash(state);

        for (key, val) in self.params() {
            key.hash(state);
            val.hash(state);
        }
    }
}

impl fmt::Display for MediaType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}/{}", self.top(), self.sub())?;
        for (key, val) in self.params() {
            write!(f, "; {}={}", key, val)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;
    use super::MediaType;

    macro_rules! assert_no_parse {
        ($string:expr) => ({
            let result = MediaType::from_str($string);
            if result.is_ok() {
                panic!("{:?} parsed unexpectedly.", $string)
            }
        });
    }

    macro_rules! assert_parse {
        ($string:expr) => ({
            let result = MediaType::from_str($string);
            match result {
                Ok(media_type) => media_type,
                Err(e) => panic!("{:?} failed to parse: {}", $string, e)
            }
        });
    }

    macro_rules! assert_parse_eq {
        (@full $string:expr, $result:expr, $(($k:expr, $v:expr)),*) => ({
            let result = assert_parse!($string);
            assert_eq!(result, $result);

            let result = assert_parse!($string);
            assert_eq!(result, $result);

            let expected_params: Vec<(&str, &str)> = vec![$(($k, $v)),*];
            if expected_params.len() > 0 {
                assert_eq!(result.params().count(), expected_params.len());
                let all_params = result.params().zip(expected_params.iter());
                for ((key, val), &(ekey, eval)) in all_params {
                    assert_eq!(key, ekey);
                    assert_eq!(val, eval);
                }
            }
        });

        (from: $string:expr, into: $result:expr)
            => (assert_parse_eq!(@full $string, $result, ));
        (from: $string:expr, into: $result:expr, params: $(($key:expr, $val:expr)),*)
            => (assert_parse_eq!(@full $string, $result, $(($key, $val)),*));
    }

    #[test]
    fn check_does_parse() {
        assert_parse!("text/html");
        assert_parse!("a/b");
        assert_parse!("*/*");
    }

    #[test]
    fn check_parse_eq() {
        assert_parse_eq!(from: "text/html", into: MediaType::HTML);
        assert_parse_eq!(from: "text/html; charset=utf-8", into: MediaType::HTML);
        assert_parse_eq!(from: "text/html", into: MediaType::new("text", "html"));

        assert_parse_eq!(from: "a/b", into: MediaType::new("a", "b"));
        assert_parse_eq!(from: "*/*", into: MediaType::Any);
        assert_parse_eq!(from: "application/pdf", into: MediaType::PDF);
        assert_parse_eq!(from: "application/json", into: MediaType::JSON);
        assert_parse_eq!(from: "image/svg+xml", into: MediaType::SVG);

        assert_parse_eq!(from: "*/json", into: MediaType::new("*", "json"));
        assert_parse_eq! {
            from: "application/*; param=1",
            into: MediaType::new("application", "*")
        };
    }

    #[test]
    fn check_param_eq() {
        assert_parse_eq! {
            from: "text/html; a=b; b=c; c=d",
            into: MediaType::new("text", "html"),
            params: ("a", "b"), ("b", "c"), ("c", "d")
        };

        assert_parse_eq! {
            from: "text/html;a=b;b=c;     c=d; d=e",
            into: MediaType::new("text", "html"),
            params: ("a", "b"), ("b", "c"), ("c", "d"), ("d", "e")
        };

        assert_parse_eq! {
            from: "text/html; charset=utf-8",
            into: MediaType::new("text", "html"),
            params: ("charset", "utf-8")
        };

        assert_parse_eq! {
            from: "application/*; param=1",
            into: MediaType::new("application", "*"),
            params: ("param", "1")
        };

        assert_parse_eq! {
            from: "*/*;q=0.5;b=c;c=d",
            into: MediaType::Any,
            params: ("q", "0.5"), ("b", "c"), ("c", "d")
        };

        assert_parse_eq! {
            from: "multipart/form-data; boundary=----WebKitFormBoundarypRshfItmvaC3aEuq",
            into: MediaType::FormData,
            params: ("boundary", "----WebKitFormBoundarypRshfItmvaC3aEuq")
        };

        assert_parse_eq! {
            from: r#"*/*; a="hello, world!@#$%^&*();;hi""#,
            into: MediaType::Any,
            params: ("a", "hello, world!@#$%^&*();;hi")
        };

        assert_parse_eq! {
            from: r#"application/json; a=";,;""#,
            into: MediaType::JSON,
            params: ("a", ";,;")
        };

        assert_parse_eq! {
            from: r#"application/json; a=";,;"; b=c"#,
            into: MediaType::JSON,
            params: ("a", ";,;"), ("b", "c")
        };

        assert_parse_eq! {
            from: r#"application/json; b=c; a=";.,.;""#,
            into: MediaType::JSON,
            params: ("b", "c"), ("a", ";.,.;")
        };

        assert_parse_eq! {
            from: r#"*/*; a="a"; b="b"; a=a; b=b; c=c"#,
            into: MediaType::Any,
            params: ("a", "a"), ("b", "b"), ("a", "a"), ("b", "b"), ("c", "c")
        };
    }

    #[test]
    fn check_params_do_parse() {
        assert_parse!("*/*; q=1; q=2");
        assert_parse!("*/*; q=1;q=2;q=3;a=v;c=1;da=1;sdlkldsadasd=uhisdcb89");
        assert_parse!("*/*; q=1; q=2");
        assert_parse!("*/*; q=1; q=2; a=b;c=d;    e=f; a=s;a=e");
        assert_parse!("*/*; q=1; q=2 ; a=b");
        assert_parse!("*/*; q=1; q=2; hello=\"world !\"");
    }

    #[test]
    fn test_bad_parses() {
        assert_no_parse!("application//json");
        assert_no_parse!("application///json");
        assert_no_parse!("a/b;");
        assert_no_parse!("*/*; a=b;;");
        assert_no_parse!("*/*; a=b;a");
        assert_no_parse!("*/*; a=b; ");
        assert_no_parse!("*/*; a=b;");
        assert_no_parse!("*/*; a = b");
        assert_no_parse!("*/*; a= b");
        assert_no_parse!("*/*; a =b");
        assert_no_parse!(r#"*/*; a="b"#);
        assert_no_parse!(r#"*/*; a="b; c=d"#);
        assert_no_parse!(r#"*/*; a="b; c=d"#);
    }
}
