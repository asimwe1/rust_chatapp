pub use response::mime::{Mime, TopLevel, SubLevel};
use response::mime::Param;
use std::default::Default;

use std::str::FromStr;
use std::borrow::Borrow;
use std::fmt;

use router::Collider;

/// Rocket's representation of HTTP Content-Types.
///
/// This type wraps raw HTTP `Content-Type`s in a type-safe manner. It provides
/// methods to create and test against common HTTP content-types. It also
/// provides methods to parse HTTP Content-Type values
/// ([from_str](#method.from_str)) and to return the ContentType associated with
/// a file extension ([from_ext](#method.from_extension)).
#[derive(Debug, Clone, PartialEq)]
pub struct ContentType(pub TopLevel, pub SubLevel, pub Option<Vec<Param>>);

macro_rules! is_some {
    ($ct:ident, $name:ident: $top:ident/$sub:ident) => {
        /// Returns a new ContentType that matches the MIME for this method's
        /// name.
        pub fn $ct() -> ContentType {
            ContentType::of(TopLevel::$top, SubLevel::$sub)
        }

        is_some!($name: $top/$sub);
    };

    ($name:ident: $top:ident/$sub:ident) => {
        /// Returns true if `self` is the content type matching the method's
        /// name.
        pub fn $name(&self) -> bool {
            self.0 == TopLevel::$top && self.1 == SubLevel::$sub
        }
    };
}

impl ContentType {
    #[doc(hidden)]
    #[inline(always)]
    pub fn new(t: TopLevel, s: SubLevel, params: Option<Vec<Param>>) -> ContentType {
        ContentType(t, s, params)
    }

    /// Constructs a new content type of the given top level and sub level
    /// types.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rocket::ContentType;
    /// use rocket::response::mime::{TopLevel, SubLevel};
    ///
    /// let html = ContentType::of(TopLevel::Application, SubLevel::Html);
    /// assert!(html.is_html());
    /// ```
    #[inline(always)]
    pub fn of(t: TopLevel, s: SubLevel) -> ContentType {
        ContentType(t, s, None)
    }

    /// Returns a new ContentType for `*/*`, i.e., any.
    #[inline(always)]
    pub fn any() -> ContentType {
        ContentType::of(TopLevel::Star, SubLevel::Star)
    }

    /// Returns true if this content type is not one of the standard content
    /// types, that if, if it is an "extended" content type.
    pub fn is_ext(&self) -> bool {
        if let TopLevel::Ext(_) = self.0 {
            true
        } else if let SubLevel::Ext(_) = self.1 {
            true
        } else {
            false
        }
    }

    /// Returns true if the content type is JSON, i.e: `application/json`.
    is_some!(json, is_json: Application/Json);

    /// Returns true if the content type is XML, i.e: `application/xml`.
    is_some!(xml, is_xml: Application/Xml);

    /// Returns true if the content type is any, i.e.: `*/*`.
    is_some!(is_any: Star/Star);

    /// Returns true if the content type is HTML, i.e.: `application/html`.
    is_some!(html, is_html: Application/Html);

    /// Returns true if the content type is that for non-data HTTP forms, i.e.:
    /// `application/x-www-form-urlencoded`.
    is_some!(is_form: Application/WwwFormUrlEncoded);

    /// Returns true if the content type is that for data HTTP forms, i.e.:
    /// `multipart/form-data`.
    is_some!(is_data: Multipart/FormData);

    /// Returns the Content-Type associated with the extension `ext`. Not all
    /// extensions are recognized. If an extensions is not recognized, then this
    /// method returns a ContentType of `any`.
    ///
    /// # Example
    ///
    /// A recognized content type:
    ///
    /// ```rust
    /// use rocket::ContentType;
    ///
    /// let xml = ContentType::from_extension("xml");
    /// assert!(xml.is_xml());
    /// ```
    ///
    /// An unrecognized content type:
    ///
    /// ```rust
    /// use rocket::ContentType;
    ///
    /// let foo = ContentType::from_extension("foo");
    /// assert!(foo.is_any());
    /// ```
    pub fn from_extension(ext: &str) -> ContentType {
        let (top_level, sub_level) = match ext {
            "txt" => (TopLevel::Text, SubLevel::Plain),
            "html" => (TopLevel::Text, SubLevel::Html),
            "xml" => (TopLevel::Application, SubLevel::Xml),
            "js" => (TopLevel::Application, SubLevel::Javascript),
            "css" => (TopLevel::Text, SubLevel::Css),
            "json" => (TopLevel::Application, SubLevel::Json),
            "png" => (TopLevel::Image, SubLevel::Png),
            "gif" => (TopLevel::Image, SubLevel::Gif),
            "bmp" => (TopLevel::Image, SubLevel::Bmp),
            "jpeg" => (TopLevel::Image, SubLevel::Jpeg),
            "jpg" => (TopLevel::Image, SubLevel::Jpeg),
            "pdf" => (TopLevel::Application, SubLevel::Ext("pdf".into())),
            _ => (TopLevel::Star, SubLevel::Star),
        };

        ContentType::of(top_level, sub_level)
    }
}

impl Default for ContentType {
    /// Returns a ContentType of `any`, or `*/*`.
    #[inline(always)]
    fn default() -> ContentType {
        ContentType::any()
    }
}

#[doc(hidden)]
impl Into<Mime> for ContentType {
    fn into(self) -> Mime {
        Mime(self.0, self.1, self.2.unwrap_or_default())
    }
}

#[doc(hidden)]
impl<T: Borrow<Mime>> From<T> for ContentType {
    default fn from(mime: T) -> ContentType {
        let mime: Mime = mime.borrow().clone();
        ContentType::from(mime)
    }
}

#[doc(hidden)]
impl From<Mime> for ContentType {
    fn from(mime: Mime) -> ContentType {
        let params = match mime.2.len() {
            0 => None,
            _ => Some(mime.2),
        };

        ContentType(mime.0, mime.1, params)
    }
}

fn is_valid_first_char(c: char) -> bool {
    match c {
        'a'...'z' | 'A'...'Z' | '0'...'9' | '*' => true,
        _ => false,
    }
}

fn is_valid_char(c: char) -> bool {
    is_valid_first_char(c) || match c {
        '!' | '#' | '$' | '&' | '-' | '^' | '.' | '+' | '_' => true,
        _ => false,
    }
}

impl FromStr for ContentType {
    type Err = &'static str;

    /// Parses a ContentType from a given Content-Type header value.
    ///
    /// # Examples
    ///
    /// Parsing an `application/json`:
    ///
    /// ```rust
    /// use rocket::ContentType;
    /// use std::str::FromStr;
    ///
    /// let json = ContentType::from_str("application/json");
    /// assert_eq!(json, Ok(ContentType::json()));
    /// ```
    ///
    /// Parsing a content-type extension:
    ///
    /// ```rust
    /// use rocket::ContentType;
    /// use std::str::FromStr;
    /// use rocket::response::mime::{TopLevel, SubLevel};
    ///
    /// let custom = ContentType::from_str("application/x-custom").unwrap();
    /// assert!(custom.is_ext());
    /// assert_eq!(custom.0, TopLevel::Application);
    /// assert_eq!(custom.1, SubLevel::Ext("x-custom".into()));
    /// ```
    ///
    /// Parsing an invalid Content-Type value:
    ///
    /// ```rust
    /// use rocket::ContentType;
    /// use std::str::FromStr;
    ///
    /// let custom = ContentType::from_str("application//x-custom");
    /// assert!(custom.is_err());
    /// ```
    fn from_str(raw: &str) -> Result<ContentType, &'static str> {
        let slash = match raw.find('/') {
            Some(i) => i,
            None => return Err("Missing / in MIME type."),
        };

        let top_s = &raw[..slash];
        let (sub_s, _rest) = match raw.find(';') {
            Some(j) => (&raw[(slash + 1)..j], Some(&raw[(j + 1)..])),
            None => (&raw[(slash + 1)..], None),
        };

        if top_s.len() < 1 || sub_s.len() < 1 {
            return Err("Empty string.");
        }

        if !is_valid_first_char(top_s.chars().next().unwrap())
                || !is_valid_first_char(sub_s.chars().next().unwrap()) {
            return Err("Invalid first char.");
        }

        if top_s.contains(|c| !is_valid_char(c))
                || sub_s.contains(|c| !is_valid_char(c)) {
            return Err("Invalid character in string.");
        }

        let (top_s, sub_s) = (&*top_s.to_lowercase(), &*sub_s.to_lowercase());
        let top_level = TopLevel::from_str(top_s).map_err(|_| "Bad TopLevel")?;
        let sub_level = SubLevel::from_str(sub_s).map_err(|_| "Bad SubLevel")?;

        // FIXME: Use `rest` to find params.
        Ok(ContentType::new(top_level, sub_level, None))
    }
}

impl fmt::Display for ContentType {
    /// Formats the ContentType as an HTTP Content-Type value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket::ContentType;
    ///
    /// let http_ct = format!("{}", ContentType::xml());
    /// assert_eq!(http_ct, "application/xml".to_string());
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}/{}", self.0.as_str(), self.1.as_str())?;

        self.2.as_ref().map_or(Ok(()), |params| {
            for param in params.iter() {
                let (ref attr, ref value) = *param;
                write!(f, "; {}={}", attr, value)?;
            }

            Ok(())
        })
    }
}

impl Collider for ContentType {
    fn collides_with(&self, other: &ContentType) -> bool {
        self.0.collides_with(&other.0) && self.1.collides_with(&other.1)
    }
}

impl Collider for TopLevel {
    fn collides_with(&self, other: &TopLevel) -> bool {
        *self == TopLevel::Star || *other == TopLevel::Star || *self == *other
    }
}

impl Collider for SubLevel {
    fn collides_with(&self, other: &SubLevel) -> bool {
        *self == SubLevel::Star || *other == SubLevel::Star || *self == *other
    }
}

#[cfg(test)]
mod test {
    use super::ContentType;
    use hyper::mime::{TopLevel, SubLevel};
    use std::str::FromStr;


    macro_rules! assert_no_parse {
        ($string:expr) => ({
            let result = ContentType::from_str($string);
            if !result.is_err() {
                println!("{} parsed!", $string);
            }

            assert!(result.is_err());
        });
    }

    macro_rules! assert_parse {
        ($string:expr) => ({
            let result = ContentType::from_str($string);
            assert!(result.is_ok());
            result.unwrap()
        });
        ($string:expr, $top:tt/$sub:tt) => ({
            let c = assert_parse!($string);
            assert_eq!(c.0, TopLevel::$top);
            assert_eq!(c.1, SubLevel::$sub);
            c
        })
    }

    #[test]
    fn test_simple() {
        assert_parse!("application/json", Application/Json);
        assert_parse!("*/json", Star/Json);
        assert_parse!("text/html", Text/Html);
        assert_parse!("TEXT/html", Text/Html);
        assert_parse!("*/*", Star/Star);
        assert_parse!("application/*", Application/Star);
    }

    #[test]
    fn test_params() {
        assert_parse!("application/json; charset=utf8", Application/Json);
        assert_parse!("application/*;charset=utf8;else=1", Application/Star);
        assert_parse!("*/*;charset=utf8;else=1", Star/Star);
    }

    #[test]
    fn test_bad_parses() {
        assert_no_parse!("application//json");
        assert_no_parse!("application///json");
        assert_no_parse!("/json");
        assert_no_parse!("text/");
        assert_no_parse!("text//");
        assert_no_parse!("/");
        assert_no_parse!("*/");
        assert_no_parse!("/*");
        assert_no_parse!("///");
    }
}
