use std::borrow::Cow;
use std::fmt::{self, Display};

use crate::ext::IntoOwned;
use crate::parse::{Extent, IndexedStr};
use crate::uri::{Authority, Origin, Error, as_utf8_unchecked};

/// A URI with a scheme, authority, path, and query:
/// `http://user:pass@domain.com:4444/path?query`.
///
/// # Structure
///
/// The following diagram illustrates the syntactic structure of an absolute
/// URI with all optional parts:
///
/// ```text
///  http://user:pass@domain.com:4444/path?query
///  |--|   |-----------------------||---------|
/// scheme          authority          origin
/// ```
///
/// The scheme part of the absolute URI and at least one of authority or origin
/// are required.
#[derive(Debug, Clone)]
pub struct Absolute<'a> {
    source: Option<Cow<'a, str>>,
    scheme: IndexedStr<'a>,
    authority: Option<Authority<'a>>,
    origin: Option<Origin<'a>>,
}

impl IntoOwned for Absolute<'_> {
    type Owned = Absolute<'static>;

    fn into_owned(self) -> Self::Owned {
        Absolute {
            source: self.source.into_owned(),
            scheme: self.scheme.into_owned(),
            authority: self.authority.into_owned(),
            origin: self.origin.into_owned(),
        }
    }
}

impl<'a> Absolute<'a> {
    #[inline]
    pub(crate) unsafe fn raw(
        source: Cow<'a, [u8]>,
        scheme: Extent<&'a [u8]>,
        authority: Option<Authority<'a>>,
        origin: Option<Origin<'a>>,
    ) -> Absolute<'a> {
        Absolute {
            authority, origin,
            source: Some(as_utf8_unchecked(source)),
            scheme: scheme.into(),
        }
    }

    #[cfg(test)]
    pub(crate) fn new(
        scheme: &'a str,
        authority: Option<Authority<'a>>,
        origin: Option<Origin<'a>>
    ) -> Absolute<'a> {
        Absolute {
            authority, origin,
            source: None,
            scheme: Cow::Borrowed(scheme).into(),
        }
    }

    /// Parses the string `string` into an `Absolute`. Parsing will never
    /// allocate. Returns an `Error` if `string` is not a valid absolute URI.
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate rocket;
    /// use rocket::http::uri::Absolute;
    ///
    /// // Parse a valid authority URI.
    /// let uri = Absolute::parse("http://google.com").expect("valid URI");
    /// assert_eq!(uri.scheme(), "http");
    /// assert_eq!(uri.authority().unwrap().host(), "google.com");
    /// assert_eq!(uri.origin(), None);
    /// ```
    pub fn parse(string: &'a str) -> Result<Absolute<'a>, Error<'a>> {
        crate::parse::uri::absolute_from_str(string)
    }

    /// Returns the scheme part of the absolute URI.
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate rocket;
    /// use rocket::http::uri::Absolute;
    ///
    /// let uri = Absolute::parse("ftp://127.0.0.1").expect("valid URI");
    /// assert_eq!(uri.scheme(), "ftp");
    /// ```
    #[inline(always)]
    pub fn scheme(&self) -> &str {
        self.scheme.from_cow_source(&self.source)
    }

    /// Returns the authority part of the absolute URI, if there is one.
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate rocket;
    /// use rocket::http::uri::Absolute;
    ///
    /// let uri = Absolute::parse("https://rocket.rs:80").expect("valid URI");
    /// assert_eq!(uri.scheme(), "https");
    /// let authority = uri.authority().unwrap();
    /// assert_eq!(authority.host(), "rocket.rs");
    /// assert_eq!(authority.port(), Some(80));
    ///
    /// let uri = Absolute::parse("file:/web/home").expect("valid URI");
    /// assert_eq!(uri.authority(), None);
    /// ```
    #[inline(always)]
    pub fn authority(&self) -> Option<&Authority<'a>> {
        self.authority.as_ref()
    }

    /// Returns the origin part of the absolute URI, if there is one.
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate rocket;
    /// use rocket::http::uri::Absolute;
    ///
    /// let uri = Absolute::parse("file:/web/home.html?new").expect("valid URI");
    /// assert_eq!(uri.scheme(), "file");
    /// let origin = uri.origin().unwrap();
    /// assert_eq!(origin.path(), "/web/home.html");
    /// assert_eq!(origin.query().unwrap(), "new");
    ///
    /// let uri = Absolute::parse("https://rocket.rs").expect("valid URI");
    /// assert_eq!(uri.origin(), None);
    /// ```
    #[inline(always)]
    pub fn origin(&self) -> Option<&Origin<'a>> {
        self.origin.as_ref()
    }

    /// Sets the authority in `self` to `authority` and returns `self`.
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate rocket;
    /// use rocket::http::uri::{Absolute, Authority};
    ///
    /// let uri = Absolute::parse("https://rocket.rs:80").expect("valid URI");
    /// let authority = uri.authority().unwrap();
    /// assert_eq!(authority.host(), "rocket.rs");
    /// assert_eq!(authority.port(), Some(80));
    ///
    /// let new_authority = Authority::parse("google.com").unwrap();
    /// let uri = uri.with_authority(new_authority);
    /// let authority = uri.authority().unwrap();
    /// assert_eq!(authority.host(), "google.com");
    /// assert_eq!(authority.port(), None);
    /// ```
    #[inline(always)]
    pub fn with_authority(mut self, authority: Authority<'a>) -> Self {
        self.set_authority(authority);
        self
    }

    /// Sets the authority in `self` to `authority`.
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate rocket;
    /// use rocket::http::uri::{Absolute, Authority};
    ///
    /// let mut uri = Absolute::parse("https://rocket.rs:80").expect("valid URI");
    /// let authority = uri.authority().unwrap();
    /// assert_eq!(authority.host(), "rocket.rs");
    /// assert_eq!(authority.port(), Some(80));
    ///
    /// let new_authority = Authority::parse("google.com:443").unwrap();
    /// uri.set_authority(new_authority);
    /// let authority = uri.authority().unwrap();
    /// assert_eq!(authority.host(), "google.com");
    /// assert_eq!(authority.port(), Some(443));
    /// ```
    #[inline(always)]
    pub fn set_authority(&mut self, authority: Authority<'a>) {
        self.authority = Some(authority);
    }

    /// Sets the origin in `self` to `origin` and returns `self`.
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate rocket;
    /// use rocket::http::uri::{Absolute, Origin};
    ///
    /// let mut uri = Absolute::parse("http://rocket.rs/web/?new").unwrap();
    /// let origin = uri.origin().unwrap();
    /// assert_eq!(origin.path(), "/web/");
    /// assert_eq!(origin.query().unwrap(), "new");
    ///
    /// let new_origin = Origin::parse("/launch").unwrap();
    /// let uri = uri.with_origin(new_origin);
    /// let origin = uri.origin().unwrap();
    /// assert_eq!(origin.path(), "/launch");
    /// assert_eq!(origin.query(), None);
    /// ```
    #[inline(always)]
    pub fn with_origin(mut self, origin: Origin<'a>) -> Self {
        self.set_origin(origin);
        self
    }

    /// Sets the origin in `self` to `origin`.
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate rocket;
    /// use rocket::http::uri::{Absolute, Origin};
    ///
    /// let mut uri = Absolute::parse("http://rocket.rs/web/?new").unwrap();
    /// let origin = uri.origin().unwrap();
    /// assert_eq!(origin.path(), "/web/");
    /// assert_eq!(origin.query().unwrap(), "new");
    ///
    /// let new_origin = Origin::parse("/launch?when=now").unwrap();
    /// uri.set_origin(new_origin);
    /// let origin = uri.origin().unwrap();
    /// assert_eq!(origin.path(), "/launch");
    /// assert_eq!(origin.query().unwrap(), "when=now");
    /// ```
    #[inline(always)]
    pub fn set_origin(&mut self, origin: Origin<'a>) {
        self.origin = Some(origin);
    }
}

impl<'a, 'b> PartialEq<Absolute<'b>> for Absolute<'a> {
    fn eq(&self, other: &Absolute<'b>) -> bool {
        self.scheme() == other.scheme()
            && self.authority() == other.authority()
            && self.origin() == other.origin()
    }
}

impl Display for Absolute<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.scheme())?;
        match self.authority {
            Some(ref authority) => write!(f, "://{}", authority)?,
            None => write!(f, ":")?
        }

        if let Some(ref origin) = self.origin {
            write!(f, "{}", origin)?;
        }

        Ok(())
    }
}
