use std::{fmt, io};
use std::num::{ParseIntError, ParseFloatError};
use std::str::{Utf8Error, ParseBoolError};
use std::net::AddrParseError;
use std::borrow::Cow;

use serde::{Serialize, ser::{Serializer, SerializeStruct}};

use crate::http::Status;
use crate::form::name::{NameBuf, Name};
use crate::data::ByteUnit;

/// A collection of [`Error`]s.
#[derive(Default, Debug, PartialEq, Serialize)]
#[serde(transparent)]
pub struct Errors<'v>(Vec<Error<'v>>);

impl crate::http::ext::IntoOwned for Errors<'_> {
    type Owned = Errors<'static>;

    fn into_owned(self) -> Self::Owned {
        Errors(self.0.into_owned())
    }
}

/// A form error, potentially tied to a specific form field.
///
/// # Serialization
///
/// When a value of this type is serialized, a `struct` or map with the
/// following fields is emitted:
///
/// | field    | type           | description                                      |
/// |----------|----------------|--------------------------------------------------|
/// | `name`   | `Option<&str>` | the erroring field's name, if known              |
/// | `value`  | `Option<&str>` | the erroring field's value, if known             |
/// | `entity` | `&str`         | string representation of the erroring [`Entity`] |
/// | `msg`    | `&str`         | concise message of the error                     |
#[derive(Debug, PartialEq)]
pub struct Error<'v> {
    /// The name of the field, if it is known.
    pub name: Option<NameBuf<'v>>,
    /// The field's value, if it is known.
    pub value: Option<Cow<'v, str>>,
    /// The kind of error that occured.
    pub kind: ErrorKind<'v>,
    /// The entitiy that caused the error.
    pub entity: Entity,
}

impl<'v> Serialize for Error<'v> {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        let mut err = ser.serialize_struct("Error", 3)?;
        err.serialize_field("name", &self.name)?;
        err.serialize_field("value", &self.value)?;
        err.serialize_field("entity", &self.entity.to_string())?;
        err.serialize_field("msg", &self.to_string())?;
        err.end()
    }
}

impl crate::http::ext::IntoOwned for Error<'_> {
    type Owned = Error<'static>;

    fn into_owned(self) -> Self::Owned {
        Error {
            name: self.name.into_owned(),
            value: self.value.into_owned(),
            kind: self.kind.into_owned(),
            entity: self.entity,
        }
    }
}

#[derive(Debug)]
pub enum ErrorKind<'v> {
    InvalidLength {
        min: Option<u64>,
        max: Option<u64>,
    },
    InvalidChoice {
        choices: Cow<'v, [Cow<'v, str>]>,
    },
    OutOfRange {
        start: Option<isize>,
        end: Option<isize>,
    },
    Validation(Cow<'v, str>),
    Duplicate,
    Missing,
    Unexpected,
    Unknown,
    Custom(Box<dyn std::error::Error + Send>),
    Multipart(multer::Error),
    Utf8(Utf8Error),
    Int(ParseIntError),
    Bool(ParseBoolError),
    Float(ParseFloatError),
    Addr(AddrParseError),
    Io(io::Error),
}

impl crate::http::ext::IntoOwned for ErrorKind<'_> {
    type Owned = ErrorKind<'static>;

    fn into_owned(self) -> Self::Owned {
        use ErrorKind::*;

        match self {
            InvalidLength { min, max } => InvalidLength { min, max },
            OutOfRange { start, end } => OutOfRange { start, end },
            Validation(s) => Validation(s.into_owned().into()),
            Duplicate => Duplicate,
            Missing => Missing,
            Unexpected => Unexpected,
            Unknown => Unknown,
            Custom(e) => Custom(e),
            Multipart(e) => Multipart(e),
            Utf8(e) => Utf8(e),
            Int(e) => Int(e),
            Bool(e) => Bool(e),
            Float(e) => Float(e),
            Addr(e) => Addr(e),
            Io(e) => Io(e),
            InvalidChoice { choices } => InvalidChoice {
                choices: choices.iter()
                    .map(|s| Cow::Owned(s.to_string()))
                    .collect::<Vec<_>>()
                    .into()
            }
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Entity {
    Form,
    Field,
    ValueField,
    DataField,
    Name,
    Value,
    Key,
    Indices,
    Index(usize),
}

impl<'v> Errors<'v> {
    pub fn new() -> Self {
        Errors(vec![])
    }

    pub fn with_name<N: Into<NameBuf<'v>>>(mut self, name: N) -> Self {
        self.set_name(name);
        self
    }

    pub fn set_name<N: Into<NameBuf<'v>>>(&mut self, name: N) {
        let name = name.into();
        for error in self.iter_mut() {
            if error.name.is_none() {
                error.set_name(name.clone());
            }
        }
    }

    pub fn with_value(mut self, value: &'v str) -> Self {
        self.set_value(value);
        self
    }

    pub fn set_value(&mut self, value: &'v str) {
        self.iter_mut().for_each(|e| e.set_value(value));
    }

    pub fn status(&self) -> Status {
        match &*self.0 {
            &[] => Status::InternalServerError,
            &[ref error] => error.status(),
            &[ref e1, ref errors@..] => errors.iter()
                .map(|e| e.status())
                .max()
                .unwrap_or_else(|| e1.status()),
        }
    }
}

impl<'v> Error<'v> {
    pub fn custom<E>(error: E) -> Self
        where E: std::error::Error + Send + 'static
    {
        (Box::new(error) as Box<dyn std::error::Error + Send>).into()
    }

    pub fn validation<S: Into<Cow<'v, str>>>(msg: S) -> Self {
        ErrorKind::Validation(msg.into()).into()
    }

    pub fn with_entity(mut self, entity: Entity) -> Self {
        self.set_entity(entity);
        self
    }

    pub fn set_entity(&mut self, entity: Entity) {
        self.entity = entity;
    }

    pub fn with_name<N: Into<NameBuf<'v>>>(mut self, name: N) -> Self {
        self.set_name(name);
        self
    }

    pub fn set_name<N: Into<NameBuf<'v>>>(&mut self, name: N) {
        if self.name.is_none() {
            self.name = Some(name.into());
        }
    }

    pub fn with_value(mut self, value: &'v str) -> Self {
        self.set_value(value);
        self
    }

    pub fn set_value(&mut self, value: &'v str) {
        if self.value.is_none() {
            self.value = Some(value.into());
        }
    }

    pub fn is_for_exactly<N: AsRef<Name>>(&self, name: N) -> bool {
        self.name.as_ref()
            .map(|n| name.as_ref() == n)
            .unwrap_or(false)
    }

    pub fn is_for<N: AsRef<Name>>(&self, name: N) -> bool {
        self.name.as_ref().map(|e_name| {
            if e_name.is_empty() != name.as_ref().is_empty() {
                return false;
            }

            let mut e_keys = e_name.keys();
            let mut n_keys = name.as_ref().keys();
            loop {
                match (e_keys.next(), n_keys.next()) {
                    (Some(e), Some(n)) if e == n => continue,
                    (Some(_), Some(_)) => return false,
                    (Some(_), None) => return false,
                    (None, _) => break,
                }
            }

            true
        })
        .unwrap_or(false)
    }

    pub fn status(&self) -> Status {
        use ErrorKind::*;
        use multer::Error::*;

        match self.kind {
            InvalidLength { min: None, .. }
            | Multipart(FieldSizeExceeded { .. })
            | Multipart(StreamSizeExceeded { .. })
                => Status::PayloadTooLarge,
            Unknown => Status::InternalServerError,
            Io(_) | _ if self.entity == Entity::Form => Status::BadRequest,
            _ => Status::UnprocessableEntity
        }
    }
}

impl<'v> ErrorKind<'v> {
    pub fn default_entity(&self) -> Entity {
        match self {
            | ErrorKind::InvalidLength { .. }
            | ErrorKind::InvalidChoice { .. }
            | ErrorKind::OutOfRange {.. }
            | ErrorKind::Validation {.. }
            | ErrorKind::Utf8(_)
            | ErrorKind::Int(_)
            | ErrorKind::Float(_)
            | ErrorKind::Bool(_)
            | ErrorKind::Custom(_)
            | ErrorKind::Addr(_) => Entity::Value,

            | ErrorKind::Duplicate
            | ErrorKind::Missing
            | ErrorKind::Unknown
            | ErrorKind::Unexpected => Entity::Field,

            | ErrorKind::Multipart(_)
            | ErrorKind::Io(_) => Entity::Form,
        }
    }
}

impl fmt::Display for ErrorKind<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::InvalidLength { min, max } => {
                match (min, max) {
                    (None, None) => write!(f, "unexpected or incomplete")?,
                    (None, Some(k)) => write!(f, "length cannot exceed {}", k)?,
                    (Some(1), None) => write!(f, "value cannot be empty")?,
                    (Some(k), None) => write!(f, "length must be at least {}", k)?,
                    (Some(i), Some(j)) => write!(f, "length must be between {} and {}", i, j)?,
                }
            }
            ErrorKind::InvalidChoice { choices } => {
                match choices.as_ref() {
                    &[] => write!(f, "invalid choice")?,
                    &[ref choice] => write!(f, "expected {}", choice)?,
                    _ => {
                        write!(f, "expected one of ")?;
                        for (i, choice) in choices.iter().enumerate() {
                            if i != 0 { write!(f, ", ")?; }
                            write!(f, "`{}`", choice)?;
                        }
                    }
                }
            }
            ErrorKind::OutOfRange { start, end } => {
                match (start, end) {
                    (None, None) => write!(f, "out of range")?,
                    (None, Some(k)) => write!(f, "value cannot exceed {}", k)?,
                    (Some(k), None) => write!(f, "value must be at least {}", k)?,
                    (Some(i), Some(j)) => write!(f, "value must be between {} and {}", i, j)?,
                }
            }
            ErrorKind::Validation(msg) => msg.fmt(f)?,
            ErrorKind::Duplicate => "duplicate".fmt(f)?,
            ErrorKind::Missing => "missing".fmt(f)?,
            ErrorKind::Unexpected => "unexpected".fmt(f)?,
            ErrorKind::Unknown => "unknown internal error".fmt(f)?,
            ErrorKind::Custom(e) => e.fmt(f)?,
            ErrorKind::Multipart(e) => write!(f, "invalid multipart: {}", e)?,
            ErrorKind::Utf8(e) => write!(f, "invalid UTF-8: {}", e)?,
            ErrorKind::Int(e) => write!(f, "invalid integer: {}", e)?,
            ErrorKind::Bool(e) => write!(f, "invalid boolean: {}", e)?,
            ErrorKind::Float(e) => write!(f, "invalid float: {}", e)?,
            ErrorKind::Addr(e) => write!(f, "invalid address: {}", e)?,
            ErrorKind::Io(e) => write!(f, "i/o error: {}", e)?,
        }

        Ok(())
    }
}

impl fmt::Display for Error<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.kind.fmt(f)
    }
}

impl fmt::Display for Entity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = match self {
            Entity::Form => "form",
            Entity::Field => "field",
            Entity::ValueField => "value field",
            Entity::DataField => "data field",
            Entity::Name => "name",
            Entity::Value => "value",
            Entity::Key => "key",
            Entity::Indices => "indices",
            Entity::Index(k) => return write!(f, "index {}", k),
        };

        string.fmt(f)
    }
}

impl fmt::Display for Errors<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} errors:", self.len())?;
        for error in self.iter() {
            write!(f, "\n{}", error)?;
        }

        Ok(())
    }
}

impl<'a, 'b> PartialEq<ErrorKind<'b>> for ErrorKind<'a> {
    fn eq(&self, other: &ErrorKind<'b>) -> bool {
        use ErrorKind::*;
        match (self, other) {
            (InvalidLength { min: a, max: b }, InvalidLength { min, max }) => min == a && max == b,
            (InvalidChoice { choices: a }, InvalidChoice { choices }) => choices == a,
            (OutOfRange { start: a, end: b }, OutOfRange { start, end }) => start == a && end == b,
            (Validation(a), Validation(b)) => a == b,
            (Duplicate, Duplicate) => true,
            (Missing, Missing) => true,
            (Unexpected, Unexpected) => true,
            (Custom(_), Custom(_)) => true,
            (Multipart(a), Multipart(b)) => a == b,
            (Utf8(a), Utf8(b)) => a == b,
            (Int(a), Int(b)) => a == b,
            (Bool(a), Bool(b)) => a == b,
            (Float(a), Float(b)) => a == b,
            (Addr(a), Addr(b)) => a == b,
            (Io(a), Io(b)) => a.kind() == b.kind(),
            _ => false,
        }
    }
}

impl<'v> std::ops::Deref for Errors<'v> {
    type Target = Vec<Error<'v>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'v> std::ops::DerefMut for Errors<'v> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'v, T: Into<Error<'v>>> From<T> for Errors<'v> {
    #[inline(always)]
    fn from(e: T) -> Self {
        Errors(vec![e.into()])
    }
}

impl<'v> From<Vec<Error<'v>>> for Errors<'v> {
    #[inline(always)]
    fn from(v: Vec<Error<'v>>) -> Self {
        Errors(v)
    }
}

impl<'v, T: Into<ErrorKind<'v>>> From<T> for Error<'v> {
    #[inline(always)]
    fn from(k: T) -> Self {
        let kind = k.into();
        let entity = kind.default_entity();
        Error { name: None, value: None, kind, entity }
    }
}

impl<'v> IntoIterator for Errors<'v> {
    type Item = Error<'v>;

    type IntoIter = <Vec<Error<'v>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'v> std::ops::Deref for Error<'v> {
    type Target = ErrorKind<'v>;

    fn deref(&self) -> &Self::Target {
        &self.kind
    }
}

impl From<(Option<u64>, Option<u64>)> for ErrorKind<'_> {
    fn from((min, max): (Option<u64>, Option<u64>)) -> Self {
        ErrorKind::InvalidLength { min, max }
    }
}

impl<'a, 'v: 'a> From<&'static [Cow<'v, str>]> for ErrorKind<'a> {
    fn from(choices: &'static [Cow<'v, str>]) -> Self {
        ErrorKind::InvalidChoice { choices: choices.into() }
    }
}

impl<'a, 'v: 'a> From<Vec<Cow<'v, str>>> for ErrorKind<'a> {
    fn from(choices: Vec<Cow<'v, str>>) -> Self {
        ErrorKind::InvalidChoice { choices: choices.into() }
    }
}

impl From<(Option<isize>, Option<isize>)> for ErrorKind<'_> {
    fn from((start, end): (Option<isize>, Option<isize>)) -> Self {
        ErrorKind::OutOfRange { start, end }
    }
}

impl From<(Option<ByteUnit>, Option<ByteUnit>)> for ErrorKind<'_> {
    fn from((start, end): (Option<ByteUnit>, Option<ByteUnit>)) -> Self {
        use std::convert::TryFrom;

        let as_isize = |b: ByteUnit| isize::try_from(b.as_u64()).ok();
        ErrorKind::from((start.and_then(as_isize), end.and_then(as_isize)))
    }
}

macro_rules! impl_from_choices {
    ($($size:literal),*) => ($(
        impl<'a, 'v: 'a> From<&'static [Cow<'v, str>; $size]> for ErrorKind<'a> {
            fn from(choices: &'static [Cow<'v, str>; $size]) -> Self {
                let choices = &choices[..];
                ErrorKind::InvalidChoice { choices: choices.into() }
            }
        }
    )*)
}

impl_from_choices!(1, 2, 3, 4, 5, 6, 7, 8);

macro_rules! impl_from_for {
    (<$l:lifetime> $T:ty => $V:ty as $variant:ident) => (
        impl<$l> From<$T> for $V {
            fn from(value: $T) -> Self {
                <$V>::$variant(value)
            }
        }
    )
}

impl<'a> From<multer::Error> for Error<'a> {
    fn from(error: multer::Error) -> Self {
        use multer::Error::*;
        use self::ErrorKind::*;

        let incomplete = Error::from(InvalidLength { min: None, max: None });
        match error {
            UnknownField { field_name: Some(name) } => Error::from(Unexpected).with_name(name),
            UnknownField { field_name: None } => Error::from(Unexpected),
            FieldSizeExceeded { limit, field_name } => {
                let e = Error::from((None, Some(limit)));
                match field_name {
                    Some(name) => e.with_name(name),
                    None => e
                }
            },
            StreamSizeExceeded { limit } => {
                Error::from((None, Some(limit))).with_entity(Entity::Form)
            }
            IncompleteFieldData { field_name: Some(name) } => incomplete.with_name(name),
            IncompleteFieldData { field_name: None } => incomplete,
            IncompleteStream | IncompleteHeaders => incomplete.with_entity(Entity::Form),
            e => Error::from(ErrorKind::Multipart(e))
        }
    }
}

impl_from_for!(<'a> Utf8Error => ErrorKind<'a> as Utf8);
impl_from_for!(<'a> ParseIntError => ErrorKind<'a> as Int);
impl_from_for!(<'a> ParseFloatError => ErrorKind<'a> as Float);
impl_from_for!(<'a> ParseBoolError => ErrorKind<'a> as Bool);
impl_from_for!(<'a> AddrParseError => ErrorKind<'a> as Addr);
impl_from_for!(<'a> io::Error => ErrorKind<'a> as Io);
impl_from_for!(<'a> Box<dyn std::error::Error + Send> => ErrorKind<'a> as Custom);
