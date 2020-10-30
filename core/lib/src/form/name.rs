//! Types for handling field names, name keys, and key indices.

use std::ops::Deref;
use std::borrow::Cow;

use ref_cast::RefCast;

use crate::http::RawStr;

/// A field name composed of keys.
///
/// A form field name is composed of _keys_, delimited by `.` or `[]`. Keys, in
/// turn, are composed of _indices_, delimited by `:`. The graphic below
/// illustrates this composition for a single field in `$name=$value` format:
///
/// ```text
///       food.bart[bar:foo].blam[0_0][1000]=some-value
/// name  |--------------------------------|
/// key   |--| |--| |-----|  |--| |-|  |--|
/// index |--| |--| |-| |-|  |--| |-|  |--|
/// ```
///
/// A `Name` is a wrapper around the field name string with methods to easily
/// access its sub-components.
///
/// # Serialization
///
/// A value of this type is serialized exactly as an `&str` consisting of the
/// entire field name.
#[repr(transparent)]
#[derive(RefCast)]
pub struct Name(str);

impl Name {
    /// Wraps a string as a `Name`. This is cost-free.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket::form::name::Name;
    ///
    /// let name = Name::new("a.b.c");
    /// assert_eq!(name.as_str(), "a.b.c");
    /// ```
    pub fn new<S: AsRef<str> + ?Sized>(string: &S) -> &Name {
        Name::ref_cast(string.as_ref())
    }

    /// Returns an iterator over the keys of `self`, including empty keys.
    ///
    /// See the [top-level docs](Self) for a description of "keys".
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket::form::name::Name;
    ///
    /// let name = Name::new("apple.b[foo:bar]zoo.[barb].bat");
    /// let keys: Vec<_> = name.keys().map(|k| k.as_str()).collect();
    /// assert_eq!(keys, &["apple", "b", "foo:bar", "zoo", "", "barb", "bat"]);
    /// ```
    pub fn keys(&self) -> impl Iterator<Item = &Key> {
        struct Keys<'v>(NameView<'v>);

        impl<'v> Iterator for Keys<'v> {
            type Item = &'v Key;

            fn next(&mut self) -> Option<Self::Item> {
                if self.0.is_terminal() {
                    return None;
                }

                let key = self.0.key_lossy();
                self.0.shift();
                Some(key)
            }
        }

        Keys(NameView::new(self))
    }

    /// Returns an iterator over overlapping name prefixes of `self`, each
    /// succeeding prefix containing one more key than the previous.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket::form::name::Name;
    ///
    /// let name = Name::new("apple.b[foo:bar]");
    /// let prefixes: Vec<_> = name.prefixes().map(|p| p.as_str()).collect();
    /// assert_eq!(prefixes, &["apple", "apple.b", "apple.b[foo:bar]"]);
    ///
    /// let name = Name::new("a.b.[foo]");
    /// let prefixes: Vec<_> = name.prefixes().map(|p| p.as_str()).collect();
    /// assert_eq!(prefixes, &["a", "a.b", "a.b.", "a.b.[foo]"]);
    /// ```
    pub fn prefixes(&self) -> impl Iterator<Item = &Name> {
        struct Prefixes<'v>(NameView<'v>);

        impl<'v> Iterator for Prefixes<'v> {
            type Item = &'v Name;

            fn next(&mut self) -> Option<Self::Item> {
                if self.0.is_terminal() {
                    return None;
                }

                let name = self.0.as_name();
                self.0.shift();
                Some(name)
            }
        }

        Prefixes(NameView::new(self))
    }

    /// Borrows the underlying string.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket::form::name::Name;
    ///
    /// let name = Name::new("a.b.c");
    /// assert_eq!(name.as_str(), "a.b.c");
    /// ```
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl serde::Serialize for Name {
    fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer
    {
        self.0.serialize(ser)
    }
}

impl<'de: 'a, 'a> serde::Deserialize<'de> for &'a Name {
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
        where D: serde::Deserializer<'de>
    {
        <&'a str as serde::Deserialize<'de>>::deserialize(de).map(Name::new)
    }
}

impl<'a, S: AsRef<str> + ?Sized> From<&'a S> for &'a Name {
    #[inline]
    fn from(string: &'a S) -> Self {
        Name::new(string)
    }
}

impl Deref for Name {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<I: core::slice::SliceIndex<str, Output=str>> core::ops::Index<I> for Name {
    type Output = Name;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        self.0[index].into()
    }
}

impl PartialEq for Name {
    fn eq(&self, other: &Self) -> bool {
        self.keys().eq(other.keys())
    }
}

impl PartialEq<str> for Name {
    fn eq(&self, other: &str) -> bool {
        self == Name::new(other)
    }
}

impl PartialEq<Name> for str {
    fn eq(&self, other: &Name) -> bool {
        Name::new(self) == other
    }
}

impl PartialEq<&str> for Name {
    fn eq(&self, other: &&str) -> bool {
        self == Name::new(other)
    }
}

impl PartialEq<Name> for &str {
    fn eq(&self, other: &Name) -> bool {
        Name::new(self) == other
    }
}

impl AsRef<Name> for str {
    fn as_ref(&self) -> &Name {
        Name::new(self)
    }
}

impl AsRef<Name> for RawStr {
    fn as_ref(&self) -> &Name {
        Name::new(self)
    }
}

impl Eq for Name { }

impl std::hash::Hash for Name {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.keys().for_each(|k| k.0.hash(state))
    }
}

impl std::fmt::Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::fmt::Debug for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// A field name key composed of indices.
///
/// A form field name key is composed of _indices_, delimited by `:`. The
/// graphic below illustrates this composition for a single field in
/// `$name=$value` format:
///
/// ```text
///       food.bart[bar:foo:baz]=some-value
/// name  |--------------------|
/// key   |--| |--| |---------|
/// index |--| |--| |-| |-| |-|
/// ```
///
/// A `Key` is a wrapper around a given key string with methods to easily access
/// its indices.
///
/// # Serialization
///
/// A value of this type is serialized exactly as an `&str` consisting of the
/// entire key.
#[repr(transparent)]
#[derive(RefCast, Debug, PartialEq, Eq, Hash)]
pub struct Key(str);

impl Key {
    /// Wraps a string as a `Key`. This is cost-free.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket::form::name::Key;
    ///
    /// let key = Key::new("a:b:c");
    /// assert_eq!(key.as_str(), "a:b:c");
    /// ```
    pub fn new<S: AsRef<str> + ?Sized>(string: &S) -> &Key {
        Key::ref_cast(string.as_ref())
    }

    /// Returns an iterator over the indices of `self`, including empty indices.
    ///
    /// See the [top-level docs](Self) for a description of "indices".
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket::form::name::Key;
    ///
    /// let key = Key::new("foo:bar::baz:a.b.c");
    /// let indices: Vec<_> = key.indices().collect();
    /// assert_eq!(indices, &["foo", "bar", "", "baz", "a.b.c"]);
    /// ```
    pub fn indices(&self) -> impl Iterator<Item = &str> {
        self.split(':')
    }

    /// Borrows the underlying string.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket::form::name::Key;
    ///
    /// let key = Key::new("a:b:c");
    /// assert_eq!(key.as_str(), "a:b:c");
    /// ```
    pub fn as_str(&self) -> &str {
        &*self
    }
}

impl Deref for Key {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl serde::Serialize for Key {
    fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer
    {
        self.0.serialize(ser)
    }
}

impl<'de: 'a, 'a> serde::Deserialize<'de> for &'a Key {
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
        where D: serde::Deserializer<'de>
    {
        <&'a str as serde::Deserialize<'de>>::deserialize(de).map(Key::new)
    }
}

impl<I: core::slice::SliceIndex<str, Output=str>> core::ops::Index<I> for Key {
    type Output = Key;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        self.0[index].into()
    }
}

impl PartialEq<str> for Key {
    fn eq(&self, other: &str) -> bool {
        self == Key::new(other)
    }
}

impl PartialEq<Key> for str {
    fn eq(&self, other: &Key) -> bool {
        Key::new(self) == other
    }
}

impl<'a, S: AsRef<str> + ?Sized> From<&'a S> for &'a Key {
    #[inline]
    fn from(string: &'a S) -> Self {
        Key::new(string)
    }
}

impl AsRef<Key> for str {
    fn as_ref(&self) -> &Key {
        Key::new(self)
    }
}

impl AsRef<Key> for RawStr {
    fn as_ref(&self) -> &Key {
        Key::new(self)
    }
}

impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// A sliding-prefix view into a [`Name`].
///
/// A [`NameView`] maintains a sliding key view into a [`Name`]. The current key
/// ([`key()`]) can be [`shift()`ed](NameView::shift()) one key to the right.
/// The `Name` prefix including the current key can be extracted via
/// [`as_name()`] and the prefix _not_ including the current key via
/// [`parent()`].
///
/// [`key()`]: NameView::key()
/// [`as_name()`]: NameView::as_name()
/// [`parent()`]: NameView::parent()
///
/// This is best illustrated via an example:
///
/// ```rust
/// use rocket::form::name::NameView;
///
/// // The view begins at the first key. Illustrated: `(a).b[c:d]` where
/// // parenthesis enclose the current key.
/// let mut view = NameView::new("a.b[c:d]");
/// assert_eq!(view.key().unwrap(), "a");
/// assert_eq!(view.as_name(), "a");
/// assert_eq!(view.parent(), None);
///
/// // Shifted once to the right views the second key: `a.(b)[c:d]`.
/// view.shift();
/// assert_eq!(view.key().unwrap(), "b");
/// assert_eq!(view.as_name(), "a.b");
/// assert_eq!(view.parent().unwrap(), "a");
///
/// // Shifting again now has predictable results: `a.b[(c:d)]`.
/// view.shift();
/// assert_eq!(view.key().unwrap(), "c:d");
/// assert_eq!(view.as_name(), "a.b[c:d]");
/// assert_eq!(view.parent().unwrap(), "a.b");
///
/// // Shifting past the end means we have no further keys.
/// view.shift();
/// assert_eq!(view.key(), None);
/// assert_eq!(view.key_lossy(), "");
/// assert_eq!(view.as_name(), "a.b[c:d]");
/// assert_eq!(view.parent().unwrap(), "a.b[c:d]");
///
/// view.shift();
/// assert_eq!(view.key(), None);
/// assert_eq!(view.as_name(), "a.b[c:d]");
/// assert_eq!(view.parent().unwrap(), "a.b[c:d]");
/// ```
///
/// # Equality
///
/// `PartialEq`, `Eq`, and `Hash` all operate on the name prefix including the
/// current key. Only key values are compared; delimiters are insignificant.
/// Again, illustrated via examples:
///
/// ```rust
/// use rocket::form::name::NameView;
///
/// let mut view = NameView::new("a.b[c:d]");
/// assert_eq!(view, "a");
///
/// // Shifted once to the right views the second key: `a.(b)[c:d]`.
/// view.shift();
/// assert_eq!(view.key().unwrap(), "b");
/// assert_eq!(view.as_name(), "a.b");
/// assert_eq!(view, "a.b");
/// assert_eq!(view, "a[b]");
///
/// // Shifting again now has predictable results: `a.b[(c:d)]`.
/// view.shift();
/// assert_eq!(view, "a.b[c:d]");
/// assert_eq!(view, "a.b.c:d");
/// assert_eq!(view, "a[b].c:d");
/// assert_eq!(view, "a[b]c:d");
/// ```
#[derive(Copy, Clone)]
pub struct NameView<'v> {
    name: &'v Name,
    start: usize,
    end: usize,
}

impl<'v> NameView<'v> {
    /// Initializes a new `NameView` at the first key of `name`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket::form::name::NameView;
    ///
    /// let mut view = NameView::new("a.b[c:d]");
    /// assert_eq!(view.key().unwrap(), "a");
    /// assert_eq!(view.as_name(), "a");
    /// assert_eq!(view.parent(), None);
    /// ```
    pub fn new<N: Into<&'v Name>>(name: N) -> Self {
        let mut view = NameView { name: name.into(), start: 0, end: 0 };
        view.shift();
        view
    }

    /// Shifts the current key once to the right.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket::form::name::NameView;
    ///
    /// let mut view = NameView::new("a.b[c:d]");
    /// assert_eq!(view.key().unwrap(), "a");
    ///
    /// view.shift();
    /// assert_eq!(view.key().unwrap(), "b");
    /// ```
    pub fn shift(&mut self) {
        const START_DELIMS: &'static [char] = &['.', '['];

        let string = &self.name[self.end..];
        let bytes = string.as_bytes();
        let shift = match bytes.get(0) {
            None | Some(b'=') => 0,
            Some(b'[') => match string[1..].find(&[']', '.'][..]) {
                Some(j) => match string[1..].as_bytes()[j] {
                    b']' => j + 2,
                    _ => j + 1,
                }
                None => bytes.len(),
            }
            Some(b'.') => match string[1..].find(START_DELIMS) {
                Some(j) => j + 1,
                None => bytes.len(),
            },
            _ => match string.find(START_DELIMS) {
                Some(j) => j,
                None => bytes.len()
            }
        };

        debug_assert!(self.end + shift <= self.name.len());
        *self = NameView {
            name: self.name,
            start: self.end,
            end: self.end + shift,
        };
    }

    /// Returns the key currently viewed by `self` if it is non-empty.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket::form::name::NameView;
    ///
    /// let mut view = NameView::new("a[b]");
    /// assert_eq!(view.key().unwrap(), "a");
    ///
    /// view.shift();
    /// assert_eq!(view.key().unwrap(), "b");
    ///
    /// view.shift();
    /// assert_eq!(view.key(), None);
    /// # view.shift(); assert_eq!(view.key(), None);
    /// # view.shift(); assert_eq!(view.key(), None);
    /// ```
    pub fn key(&self) -> Option<&'v Key> {
        let lossy_key = self.key_lossy();
        if lossy_key.is_empty() {
            return None;
        }

        Some(lossy_key)
    }

    /// Returns the key currently viewed by `self`, even if it is non-empty.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket::form::name::NameView;
    ///
    /// let mut view = NameView::new("a[b]");
    /// assert_eq!(view.key_lossy(), "a");
    ///
    /// view.shift();
    /// assert_eq!(view.key_lossy(), "b");
    ///
    /// view.shift();
    /// assert_eq!(view.key_lossy(), "");
    /// # view.shift(); assert_eq!(view.key_lossy(), "");
    /// # view.shift(); assert_eq!(view.key_lossy(), "");
    /// ```
    pub fn key_lossy(&self) -> &'v Key {
        let view = &self.name[self.start..self.end];
        let key = match view.as_bytes().get(0) {
            Some(b'.') => &view[1..],
            Some(b'[') if view.ends_with(']') => &view[1..view.len() - 1],
            _ => view
        };

        key.0.into()
    }

    /// Returns the `Name` _up to and including_ the current key.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket::form::name::NameView;
    ///
    /// let mut view = NameView::new("a[b]");
    /// assert_eq!(view.as_name(), "a");
    ///
    /// view.shift();
    /// assert_eq!(view.as_name(), "a[b]");
    /// # view.shift(); assert_eq!(view.as_name(), "a[b]");
    /// # view.shift(); assert_eq!(view.as_name(), "a[b]");
    /// ```
    pub fn as_name(&self) -> &'v Name {
        &self.name[..self.end]
    }

    /// Returns the `Name` _prior to_ the current key.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket::form::name::NameView;
    ///
    /// let mut view = NameView::new("a[b]");
    /// assert_eq!(view.parent(), None);
    ///
    /// view.shift();
    /// assert_eq!(view.parent().unwrap(), "a");
    ///
    /// view.shift();
    /// assert_eq!(view.parent().unwrap(), "a[b]");
    /// # view.shift(); assert_eq!(view.parent().unwrap(), "a[b]");
    /// # view.shift(); assert_eq!(view.parent().unwrap(), "a[b]");
    /// ```
    pub fn parent(&self) -> Option<&'v Name> {
        if self.start > 0 {
            Some(&self.name[..self.start])
        } else {
            None
        }
    }

    /// Returns the underlying `Name`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket::form::name::NameView;
    ///
    /// let mut view = NameView::new("a[b]");
    /// assert_eq!(view.source(), "a[b]");
    ///
    /// view.shift();
    /// assert_eq!(view.source(), "a[b]");
    ///
    /// view.shift();
    /// assert_eq!(view.source(), "a[b]");
    ///
    /// # view.shift(); assert_eq!(view.source(), "a[b]");
    /// # view.shift(); assert_eq!(view.source(), "a[b]");
    /// ```
    pub fn source(&self) -> &'v Name {
        self.name
    }

    fn is_terminal(&self) -> bool {
        self.start == self.name.len()
    }
}

impl std::fmt::Debug for NameView<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_name().fmt(f)
    }
}

impl std::fmt::Display for NameView<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_name().fmt(f)
    }
}

impl<'a, 'b> PartialEq<NameView<'b>> for NameView<'a> {
    fn eq(&self, other: &NameView<'b>) -> bool {
        self.as_name() == other.as_name()
    }
}

impl<B: PartialEq<Name>> PartialEq<B> for NameView<'_> {
    fn eq(&self, other: &B) -> bool {
        other == self.as_name()
    }
}

impl Eq for NameView<'_> {  }

impl std::hash::Hash for NameView<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_name().hash(state)
    }
}

impl std::borrow::Borrow<Name> for NameView<'_> {
    fn borrow(&self) -> &Name {
        self.as_name()
    }
}

/// A potentially owned [`Name`].
///
/// Constructible from a [`NameView`], [`Name`], `&str`, or `String`, a
/// `NameBuf` acts much like a [`Name`] but can be converted into an owned
/// version via [`IntoOwned`](crate::http::ext::IntoOwned).
///
/// ```rust
/// use rocket::form::name::NameBuf;
/// use rocket::http::ext::IntoOwned;
///
/// let alloc = String::from("a.b.c");
/// let name = NameBuf::from(alloc.as_str());
/// let owned: NameBuf<'static> = name.into_owned();
/// ```
#[derive(Clone)]
pub struct NameBuf<'v> {
    left: &'v Name,
    right: Cow<'v, str>,
}

impl<'v> NameBuf<'v> {
    #[inline]
    fn split(&self) -> (&Name, &Name) {
        (self.left, Name::new(&self.right))
    }

    /// Returns an iterator over the keys of `self`, including empty keys.
    ///
    /// See [`Name`] for a description of "keys".
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket::form::name::NameBuf;
    ///
    /// let name = NameBuf::from("apple.b[foo:bar]zoo.[barb].bat");
    /// let keys: Vec<_> = name.keys().map(|k| k.as_str()).collect();
    /// assert_eq!(keys, &["apple", "b", "foo:bar", "zoo", "", "barb", "bat"]);
    /// ```
    #[inline]
    pub fn keys(&self) -> impl Iterator<Item = &Key> {
        let (left, right) = self.split();
        left.keys().chain(right.keys())
    }

    /// Returns `true` if `self` is empty.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket::form::name::NameBuf;
    ///
    /// let name = NameBuf::from("apple.b[foo:bar]zoo.[barb].bat");
    /// assert!(!name.is_empty());
    ///
    /// let name = NameBuf::from("");
    /// assert!(name.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        let (left, right) = self.split();
        left.is_empty() && right.is_empty()
    }
}

impl crate::http::ext::IntoOwned for NameBuf<'_> {
    type Owned = NameBuf<'static>;

    fn into_owned(self) -> Self::Owned {
        let right = match (self.left, self.right) {
            (l, Cow::Owned(r)) if l.is_empty() => Cow::Owned(r),
            (l, r) if l.is_empty() => r.to_string().into(),
            (l, r) if r.is_empty() => l.to_string().into(),
            (l, r) => format!("{}.{}", l, r).into(),
        };

        NameBuf { left: "".into(), right }
    }
}

impl serde::Serialize for NameBuf<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'v> From<NameView<'v>> for NameBuf<'v> {
    fn from(nv: NameView<'v>) -> Self {
        NameBuf { left: nv.as_name(), right: Cow::Borrowed("") }
    }
}

impl<'v> From<&'v Name> for NameBuf<'v> {
    fn from(name: &'v Name) -> Self {
        NameBuf { left: name, right: Cow::Borrowed("") }
    }
}

impl<'v> From<&'v str> for NameBuf<'v> {
    fn from(name: &'v str) -> Self {
        NameBuf::from((None, Cow::Borrowed(name)))
    }
}

impl<'v> From<String> for NameBuf<'v> {
    fn from(name: String) -> Self {
        NameBuf::from((None, Cow::Owned(name)))
    }
}

#[doc(hidden)]
impl<'v> From<(Option<&'v Name>, Cow<'v, str>)> for NameBuf<'v> {
    fn from((prefix, right): (Option<&'v Name>, Cow<'v, str>)) -> Self {
        match prefix {
            Some(left) => NameBuf { left, right },
            None => NameBuf { left: "".into(), right }
        }
    }
}

#[doc(hidden)]
impl<'v> From<(Option<&'v Name>, &'v str)> for NameBuf<'v> {
    fn from((prefix, suffix): (Option<&'v Name>, &'v str)) -> Self {
        NameBuf::from((prefix, Cow::Borrowed(suffix)))
    }
}

#[doc(hidden)]
impl<'v> From<(&'v Name, &'v str)> for NameBuf<'v> {
    fn from((prefix, suffix): (&'v Name, &'v str)) -> Self {
        NameBuf::from((Some(prefix), Cow::Borrowed(suffix)))
    }
}

impl std::fmt::Debug for NameBuf<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"")?;

        let (left, right) = self.split();
        if !left.is_empty() { write!(f, "{}", left.escape_debug())? }
        if !right.is_empty() {
            if !left.is_empty() { f.write_str(".")?; }
            write!(f, "{}", right.escape_debug())?;
        }

        write!(f, "\"")
    }
}

impl std::fmt::Display for NameBuf<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (left, right) = self.split();
        if !left.is_empty() { left.fmt(f)?; }
        if !right.is_empty() {
            if !left.is_empty() { f.write_str(".")?; }
            right.fmt(f)?;
        }

        Ok(())
    }
}

impl PartialEq for NameBuf<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.keys().eq(other.keys())
    }
}

impl<N: AsRef<Name> + ?Sized> PartialEq<N> for NameBuf<'_> {
    fn eq(&self, other: &N) -> bool {
        self.keys().eq(other.as_ref().keys())
    }
}

impl PartialEq<Name> for NameBuf<'_> {
    fn eq(&self, other: &Name) -> bool {
        self.keys().eq(other.keys())
    }
}

impl PartialEq<NameBuf<'_>> for Name {
    fn eq(&self, other: &NameBuf<'_>) -> bool {
        self.keys().eq(other.keys())
    }
}

impl PartialEq<NameBuf<'_>> for str {
    fn eq(&self, other: &NameBuf<'_>) -> bool {
        Name::new(self) == other
    }
}

impl PartialEq<NameBuf<'_>> for &str {
    fn eq(&self, other: &NameBuf<'_>) -> bool {
        Name::new(self) == other
    }
}

impl Eq for NameBuf<'_> { }

impl std::hash::Hash for NameBuf<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.keys().for_each(|k| k.0.hash(state))
    }
}

impl indexmap::Equivalent<Name> for NameBuf<'_> {
    fn equivalent(&self, key: &Name) -> bool {
        self.keys().eq(key.keys())
    }
}

impl indexmap::Equivalent<NameBuf<'_>> for Name {
    fn equivalent(&self, key: &NameBuf<'_>) -> bool {
        self.keys().eq(key.keys())
    }
}
