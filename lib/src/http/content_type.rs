use std::borrow::{Borrow, Cow};
use std::str::FromStr;
use std::fmt;

use http::Header;
use http::mime::Mime;
use router::Collider;

/// Typed representation of HTTP Content-Types.
///
/// This type wraps raw HTTP `Content-Type`s in a type-safe manner. It provides
/// methods to create and test against common HTTP content-types. It also
/// provides methods to parse HTTP Content-Type values
/// ([from_str](#method.from_str)) and to return the ContentType associated with
/// a file extension ([from_ext](#method.from_extension)).
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct ContentType {
    pub ttype: Cow<'static, str>,
    pub subtype: Cow<'static, str>,
    pub params: Option<Cow<'static, str>>
}

macro_rules! ctr_params {
    () => (None);
    ($param:expr) => (Some(Cow::Borrowed($param)));
}

macro_rules! ctrs {
    ($($str:expr, $name:ident, $check_name:ident =>
            $top:expr, $sub:expr $(; $param:expr),*),+) => {
        $(
            #[doc="[ContentType](struct.ContentType.html) for <b>"]
            #[doc=$str]
            #[doc="</b>: <i>"]
            #[doc=$top]
            #[doc="/"]
            #[doc=$sub]
            $(#[doc="; "] #[doc=$param])*
            #[doc="</i>"]
            #[allow(non_upper_case_globals)]
            pub const $name: ContentType = ContentType {
                ttype: Cow::Borrowed($top),
                subtype: Cow::Borrowed($sub),
                params: ctr_params!($($param)*)
            };
         )+

            /// Returns `true` if this ContentType is known to Rocket.
            pub fn is_known(&self) -> bool {
                match (&*self.ttype, &*self.subtype) {
                    $(
                        ($top, $sub) => true,
                     )+
                     _ => false
                }
            }

        $(
            #[doc="Returns `true` if `self` is a <b>"]
            #[doc=$str]
            #[doc="</b> ContentType: <i>"]
            #[doc=$top]
            #[doc="</i>/<i>"]
            #[doc=$sub]
            #[doc="</i>."]
            /// Paramaters are not taken into account when doing that check.
            pub fn $check_name(&self) -> bool {
                self.ttype == $top && self.subtype == $sub
            }
         )+
    };
}

impl ContentType {
    #[inline(always)]
    pub fn new<T, S>(ttype: T, subtype: S) -> ContentType
        where T: Into<Cow<'static, str>>, S: Into<Cow<'static, str>>
    {
        ContentType {
            ttype: ttype.into(),
            subtype: subtype.into(),
            params: None
        }
    }

    #[inline(always)]
    pub fn with_params<T, S, P>(ttype: T, subtype: S, params: Option<P>) -> ContentType
        where T: Into<Cow<'static, str>>,
              S: Into<Cow<'static, str>>,
              P: Into<Cow<'static, str>>
    {
        ContentType {
            ttype: ttype.into(),
            subtype: subtype.into(),
            params: params.map(|p| p.into())
        }
    }


    /// Returns the Content-Type associated with the extension `ext`. Not all
    /// extensions are recognized. If an extensions is not recognized, then this
    /// method returns a ContentType of `any`. The currently recognized
    /// extensions are: txt, html, htm, xml, js, css, json, png, gif, bmp, jpeg,
    /// jpg, and pdf.
    ///
    /// # Example
    ///
    /// A recognized content type:
    ///
    /// ```rust
    /// use rocket::http::ContentType;
    ///
    /// let xml = ContentType::from_extension("xml");
    /// assert!(xml.is_xml());
    /// ```
    ///
    /// An unrecognized content type:
    ///
    /// ```rust
    /// use rocket::http::ContentType;
    ///
    /// let foo = ContentType::from_extension("foo");
    /// assert!(foo.is_any());
    /// ```
    pub fn from_extension(ext: &str) -> ContentType {
        match ext {
            "txt" => ContentType::Plain,
            "html" | "htm" => ContentType::HTML,
            "xml" => ContentType::XML,
            "js" => ContentType::JavaScript,
            "css" => ContentType::CSS,
            "json" => ContentType::JSON,
            "png" => ContentType::PNG,
            "gif" => ContentType::GIF,
            "bmp" => ContentType::BMP,
            "jpeg" | "jpg" => ContentType::JPEG,
            "pdf" => ContentType::PDF,
            _ => ContentType::Any
        }
    }

    ctrs! {
        "any", Any, is_any => "*", "*",
        "form", Form, is_form => "application", "x-www-form-urlencoded",
        "data form", DataForm, is_data_form => "multipart", "form-data",
        "JSON", JSON, is_json => "application", "json",
        "XML", XML, is_xml => "text", "xml" ; "charset=utf-8",
        "HTML", HTML, is_html => "text", "html" ; "charset=utf-8",
        "Plain", Plain, is_plain => "text", "plain" ; "charset=utf-8",
        "JavaScript", JavaScript, is_javascript => "application", "javascript",
        "CSS", CSS, is_css => "text", "css" ; "charset=utf-8",
        "PNG", PNG, is_png => "image", "png",
        "GIF", GIF, is_gif => "image", "gif",
        "BMP", BMP, is_bmp => "image", "bmp",
        "JPEG", JPEG, is_jpeg => "image", "jpeg",
        "PDF", PDF, is_pdf => "application", "pdf"
    }
}

impl Default for ContentType {
    /// Returns a ContentType of `any`, or `*/*`.
    #[inline(always)]
    fn default() -> ContentType {
        ContentType::Any
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
            _ => {
                Some(mime.2.into_iter()
                    .map(|(attr, value)| format!("{}={}", attr, value))
                    .collect::<Vec<_>>()
                    .join("; "))
            }
        };

        ContentType::with_params(mime.0.to_string(), mime.1.to_string(), params)
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

fn is_valid_token(string: &str) -> bool {
    if string.len() < 1 {
        return false;
    }

    string.chars().take(1).all(is_valid_first_char)
        && string.chars().skip(1).all(is_valid_char)
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
    /// use std::str::FromStr;
    /// use rocket::http::ContentType;
    ///
    /// let json = ContentType::from_str("application/json").unwrap();
    /// assert_eq!(json, ContentType::JSON);
    /// ```
    ///
    /// Parsing a content-type extension:
    ///
    /// ```rust
    /// use std::str::FromStr;
    /// use rocket::http::ContentType;
    /// use rocket::http::mime::{TopLevel, SubLevel};
    ///
    /// let custom = ContentType::from_str("application/x-custom").unwrap();
    /// assert!(!custom.is_known());
    /// assert_eq!(custom.ttype, "application");
    /// assert_eq!(custom.subtype, "x-custom");
    /// ```
    ///
    /// Parsing an invalid Content-Type value:
    ///
    /// ```rust
    /// use std::str::FromStr;
    /// use rocket::http::ContentType;
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
        let (sub_s, params) = match raw.find(';') {
            Some(j) => (raw[(slash + 1)..j].trim_right(), Some(raw[(j + 1)..].trim_left())),
            None => (&raw[(slash + 1)..], None),
        };

        if top_s.len() < 1 || sub_s.len() < 1 {
            return Err("Empty string.");
        }

        if !is_valid_token(top_s) || !is_valid_token(sub_s) {
            return Err("Invalid characters in type or subtype.");
        }

        let mut trimmed_params = vec![];
        for param in params.into_iter().flat_map(|p| p.split(";")) {
            let param = param.trim_left();
            for (i, split) in param.split("=").enumerate() {
                if split.trim() != split {
                    return Err("Whitespace not allowed around = character.");
                }

                match i {
                    0 => if !is_valid_token(split) {
                        return Err("Invalid parameter name.");
                    },
                    1 => if !((split.starts_with('"') && split.ends_with('"'))
                                || is_valid_token(split)) {
                        return Err("Invalid parameter value.");
                    },
                    _ => return Err("Malformed parameter.")
                }
            }

            trimmed_params.push(param);
        }

        let (ttype, subtype) = (top_s.to_lowercase(), sub_s.to_lowercase());
        let params = params.map(|_| trimmed_params.join(";"));
        Ok(ContentType::with_params(ttype, subtype, params))
    }
}

impl fmt::Display for ContentType {
    /// Formats the ContentType as an HTTP Content-Type value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket::http::ContentType;
    ///
    /// let ct = format!("{}", ContentType::JSON);
    /// assert_eq!(ct, "application/json");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}/{}", self.ttype, self.subtype)?;

        if let Some(ref params) = self.params {
            write!(f, "; {}", params)?;
        }

        Ok(())
    }
}

impl Into<Header<'static>> for ContentType {
    #[inline]
    fn into(self) -> Header<'static> {
        Header::new("Content-Type", self.to_string())
    }
}

impl Collider for ContentType {
    fn collides_with(&self, other: &ContentType) -> bool {
        (self.ttype == "*" || other.ttype == "*" || self.ttype == other.ttype) &&
        (self.subtype == "*" || other.subtype == "*" || self.subtype == other.subtype)
    }
}

#[cfg(test)]
mod test {
    use super::ContentType;
    use std::str::FromStr;

    macro_rules! assert_no_parse {
        ($string:expr) => ({
            let result = ContentType::from_str($string);
            if !result.is_err() {
                println!("{} parsed unexpectedly!", $string);
            }

            assert!(result.is_err());
        });
    }

    macro_rules! assert_parse {
        ($string:expr) => ({
            let result = ContentType::from_str($string);
            if let Err(e) = result {
                println!("{:?} failed to parse: {}", $string, e);
            }

            result.unwrap()
        });

        ($string:expr, $ct:expr) => ({
            let c = assert_parse!($string);
            assert_eq!(c.ttype, $ct.ttype);
            assert_eq!(c.subtype, $ct.subtype);
            assert_eq!(c.params, $ct.params);
            c
        })
    }

    #[test]
    fn test_simple() {
        assert_parse!("application/json", ContentType::JSON);
        assert_parse!("*/json", ContentType::new("*", "json"));
        assert_parse!("text/html;charset=utf-8", ContentType::HTML);
        assert_parse!("text/html ; charset=utf-8", ContentType::HTML);
        assert_parse!("text/html ;charset=utf-8", ContentType::HTML);
        assert_parse!("TEXT/html;charset=utf-8", ContentType::HTML);
        assert_parse!("*/*", ContentType::Any);
        assert_parse!("application/*", ContentType::new("application", "*"));
    }

    #[test]
    fn test_params() {
        assert_parse!("*/*;a=1;b=2;c=3;d=4",
            ContentType::with_params("*", "*", Some("a=1;b=2;c=3;d=4")));
        assert_parse!("*/*; a=1;   b=2; c=3;d=4",
            ContentType::with_params("*", "*", Some("a=1;b=2;c=3;d=4")));
        assert_parse!("application/*;else=1",
            ContentType::with_params("application", "*", Some("else=1")));
        assert_parse!("*/*;charset=utf-8;else=1",
            ContentType::with_params("*", "*", Some("charset=utf-8;else=1")));
        assert_parse!("*/*;    charset=utf-8;   else=1",
            ContentType::with_params("*", "*", Some("charset=utf-8;else=1")));
        assert_parse!("*/*;    charset=\"utf-8\";   else=1",
            ContentType::with_params("*", "*", Some("charset=\"utf-8\";else=1")));
    }

    #[test]
    fn test_bad_parses() {
        assert_no_parse!("application//json");
        assert_no_parse!("application///json");
        assert_no_parse!("*&_/*)()");
        assert_no_parse!("/json");
        assert_no_parse!("text/");
        assert_no_parse!("text//");
        assert_no_parse!("/");
        assert_no_parse!("*/");
        assert_no_parse!("/*");
        assert_no_parse!("///");
        assert_no_parse!("");
        assert_no_parse!("*/*;");
        assert_no_parse!("*/*;a=");
        assert_no_parse!("*/*;a= ");
        assert_no_parse!("*/*;a=@#$%^&*()");
        assert_no_parse!("*/*;;");
        assert_no_parse!("*/*;=;");
        assert_no_parse!("*/*=;");
        assert_no_parse!("*/*=;=");
        assert_no_parse!("*/*; a=b;");
        assert_no_parse!("*/*; a = b");
    }
}
