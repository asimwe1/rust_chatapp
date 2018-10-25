use std::fmt;

use smallvec::SmallVec;

use uri::UriDisplay;

/// A struct used to format strings for [`UriDisplay`].
///
/// A mutable version of this struct is passed to [`UriDisplay::fmt()`]. This
/// struct properly formats series of named values for use in URIs. In
/// particular, this struct applies the following transformations:
///
///   * When **mutliple values** are written, they are separated by `&`.
///
///   * When a **named value** is written with [`write_named_value()`], the name
///     is written out, followed by a `=`, followed by the value.
///
///   * When **nested named values** are written, typically by passing a value
///     to [`write_named_value()`] whose implementation of `UriDisplay` also
///     calls `write_named_vlaue()`, the nested names are joined by a `.`,
///     written out followed by a `=`, followed by the value.
///
/// [`UriDisplay`]: uri::UriDisplay
/// [`UriDisplay::fmt()`]: uri::UriDisplay::fmt()
/// [`write_named_value()`]: uri::Formatter::write_named_value()
///
/// # Usage
///
/// Usage is fairly straightforward:
///
///   * For every _named value_ you wish to emit, call [`write_named_value()`].
///   * For every _unnamed value_ you wish to emit, call [`write_value()`].
///   * To write a string directly, call [`write_raw()`].
///
/// The `write_named_value` method automatically prefixes the `name` to the
/// written value and, along with `write_value` and `write_raw`, handles nested
/// calls to `write_named_value` automatically, prefixing names when necessary.
/// Unlike the other methods, `write_raw` does _not_ prefix any nested names
/// every time it is called. Instead, it only prefixes names the _first_ time it
/// is called, after a call to `write_named_value` or `write_value`, or after a
/// call to [`refresh()`].
///
/// [`refresh()`]: uri::Formatter::refresh()
///
/// # Example
///
/// The following example uses all of the `write` methods in a varied order to
/// display the semantics of `Formatter`. Note that `UriDisplay` should rarely
/// be implemented manually, preferring to use the derive, and that this
/// implementation is purely demonstrative.
///
/// ```rust
/// # extern crate rocket;
/// use std::fmt;
///
/// use rocket::http::uri::{Formatter, UriDisplay};
///
/// struct Outer {
///     value: Inner,
///     another: usize,
///     extra: usize
/// }
///
/// struct Inner {
///     value: usize,
///     extra: usize
/// }
///
/// impl UriDisplay for Outer {
///     fn fmt(&self, f: &mut Formatter) -> fmt::Result {
///         f.write_named_value("outer_field", &self.value)?;
///         f.write_named_value("another", &self.another)?;
///         f.write_raw("out")?;
///         f.write_raw("side")?;
///         f.write_value(&self.extra)
///     }
/// }
///
/// impl UriDisplay for Inner {
///     fn fmt(&self, f: &mut Formatter) -> fmt::Result {
///         f.write_named_value("inner_field", &self.value)?;
///         f.write_value(&self.extra)?;
///         f.write_raw("inside")
///     }
/// }
///
/// let inner = Inner { value: 0, extra: 1 };
/// let outer = Outer { value: inner, another: 2, extra: 3 };
/// let uri_string = format!("{}", &outer as &UriDisplay);
/// assert_eq!(uri_string, "outer_field.inner_field=0&\
///                         outer_field=1&\
///                         outer_field=inside&\
///                         another=2&\
///                         outside&\
///                         3");
/// ```
///
/// Note that you can also use the `write!` macro to write directly to the
/// formatter as long as the [`std::fmt::Write`] trait is in scope. Internally,
/// the `write!` macro calls [`write_raw()`], so care must be taken to ensure
/// that the written string is URI-safe.
///
/// ```rust
/// # #[macro_use] extern crate rocket;
/// use std::fmt::{self, Write};
///
/// use rocket::http::uri::{Formatter, UriDisplay};
///
/// pub struct Complex(u8, u8);
///
/// impl UriDisplay for Complex {
///     fn fmt(&self, f: &mut Formatter) -> fmt::Result {
///         write!(f, "{}+{}", self.0, self.1)
///     }
/// }
///
/// #[derive(UriDisplay)]
/// struct Message {
///     number: Complex,
/// }
///
/// let message = Message { number: Complex(42, 231) };
/// let uri_string = format!("{}", &message as &UriDisplay);
/// assert_eq!(uri_string, "number=42+231");
/// ```
///
/// [`write_value()`]: uri::Formatter::write_value()
/// [`write_raw()`]: uri::Formatter::write_raw()
pub struct Formatter<'i, 'f: 'i> {
    crate prefixes: SmallVec<[&'static str; 3]>,
    crate inner: &'i mut fmt::Formatter<'f>,
    crate previous: bool,
    crate fresh: bool
}

impl<'i, 'f: 'i> Formatter<'i, 'f> {
    crate fn new(formatter: &'i mut fmt::Formatter<'f>) -> Self {
        Formatter {
            prefixes: SmallVec::new(),
            inner: formatter,
            previous: false,
            fresh: true,
        }
    }

    fn with_prefix<F>(&mut self, prefix: &str, f: F) -> fmt::Result
        where F: FnOnce(&mut Self) -> fmt::Result
    {
        // TODO: PROOF OF CORRECTNESS.
        let prefix: &'static str = unsafe { ::std::mem::transmute(prefix) };

        self.prefixes.push(prefix);
        let result = f(self);
        self.prefixes.pop();

        result
    }

    #[inline(always)]
    fn refreshed<F: FnOnce(&mut Self) -> fmt::Result>(&mut self, f: F) -> fmt::Result {
        self.refresh();
        let result = f(self);
        self.refresh();
        result
    }

    /// Writes `string` to `self`.
    ///
    /// If `self` is _fresh_ (after a call to other `write_` methods or
    /// [`refresh()`]), prefixes any names as necessary.
    ///
    /// This method is called by the `write!` macro.
    ///
    /// [`refresh()`]: Formatter::refresh()
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate rocket;
    /// use std::fmt;
    ///
    /// use rocket::http::uri::{Formatter, UriDisplay};
    ///
    /// struct Foo;
    ///
    /// impl UriDisplay for Foo {
    ///     fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    ///         f.write_raw("f")?;
    ///         f.write_raw("o")?;
    ///         f.write_raw("o")
    ///     }
    /// }
    ///
    /// let foo = Foo;
    /// let uri_string = format!("{}", &foo as &UriDisplay);
    /// assert_eq!(uri_string, "foo");
    /// ```
    pub fn write_raw<S: AsRef<str>>(&mut self, string: S) -> fmt::Result {
        let s = string.as_ref();
        if self.fresh {
            if self.previous {
                self.inner.write_str("&")?;
            }

            if !self.prefixes.is_empty() {
                for (i, prefix) in self.prefixes.iter().enumerate() {
                    self.inner.write_str(prefix)?;
                    if i < self.prefixes.len() - 1 {
                        self.inner.write_str(".")?;
                    }
                }

                self.inner.write_str("=")?;
            }
        }

        self.fresh = false;
        self.previous = true;
        self.inner.write_str(s)
    }

    /// Writes the named value `value` by prefixing `name` followed by `=` to
    /// the value. Any nested names are also prefixed as necessary.
    ///
    /// Refreshes `self` before the name is written and after the value is
    /// written.
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate rocket;
    /// use std::fmt;
    ///
    /// use rocket::http::uri::{Formatter, UriDisplay};
    ///
    /// struct Foo {
    ///     name: usize
    /// }
    ///
    /// impl UriDisplay for Foo {
    ///     fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    ///         f.write_named_value("name", &self.name)
    ///     }
    /// }
    ///
    /// let foo = Foo { name: 123 };
    /// let uri_string = format!("{}", &foo as &UriDisplay);
    /// assert_eq!(uri_string, "name=123");
    /// ```
    #[inline]
    pub fn write_named_value<T: UriDisplay>(&mut self, name: &str, value: T) -> fmt::Result {
        self.refreshed(|f| f.with_prefix(name, |f| f.write_value(value)))
    }

    /// Writes the unnamed value `value`. Any nested names are prefixed as
    /// necessary.
    ///
    /// Refreshes `self` before and after the value is written.
    ///
    /// # Example
    ///
    /// ```rust
    /// # extern crate rocket;
    /// use std::fmt;
    ///
    /// use rocket::http::uri::{Formatter, UriDisplay};
    ///
    /// struct Foo(usize);
    ///
    /// impl UriDisplay for Foo {
    ///     fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    ///         f.write_value(&self.0)
    ///     }
    /// }
    ///
    /// let foo = Foo(123);
    /// let uri_string = format!("{}", &foo as &UriDisplay);
    /// assert_eq!(uri_string, "123");
    /// ```
    #[inline]
    pub fn write_value<T: UriDisplay>(&mut self, value: T) -> fmt::Result {
        self.refreshed(|f| UriDisplay::fmt(&value, f))
    }

    /// Refreshes the formatter.
    ///
    /// After refreshing, [`write_raw()`] will prefix any nested names as well
    /// as insert an `&` separator.
    ///
    /// [`write_raw()`]: Formatter::write_raw()
    ///
    /// # Example
    ///
    /// ```rust
    /// # #[macro_use] extern crate rocket;
    /// use std::fmt;
    ///
    /// use rocket::http::uri::{Formatter, UriDisplay};
    ///
    /// struct Foo;
    ///
    /// impl UriDisplay for Foo {
    ///     fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    ///         f.write_raw("a")?;
    ///         f.write_raw("raw")?;
    ///         f.refresh();
    ///         f.write_raw("format")
    ///     }
    /// }
    ///
    /// #[derive(UriDisplay)]
    /// struct Message {
    ///     inner: Foo,
    /// }
    ///
    /// let msg = Message { inner: Foo };
    /// let uri_string = format!("{}", &msg as &UriDisplay);
    /// assert_eq!(uri_string, "inner=araw&inner=format");
    /// ```
    #[inline(always)]
    pub fn refresh(&mut self) {
        self.fresh = true;
    }
}

impl<'f, 'i: 'f> fmt::Write for Formatter<'f, 'i> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_raw(s)
    }
}
