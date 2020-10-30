use crate::form::name::NameView;
use crate::form::error::{Error, ErrorKind, Entity};
use crate::http::{ContentType, RawStr};
use crate::{Request, Data};

#[derive(Debug, Clone)]
pub struct ValueField<'r> {
    pub name: NameView<'r>,
    pub value: &'r str,
}

pub struct DataField<'r, 'i> {
    pub name: NameView<'r>,
    pub file_name: Option<&'r str>,
    pub content_type: ContentType,
    pub request: &'r Request<'i>,
    pub data: Data,
}

impl<'v> ValueField<'v> {
    /// `raw` must already be URL-decoded. This is weird.
    pub fn parse(field: &'v str) -> Self {
        // WHATWG URL Living Standard 5.1 steps 3.2, 3.3.
        let (name, val) = RawStr::new(field).split_at_byte(b'=');
        ValueField::from((name.as_str(), val.as_str()))
    }

    pub fn from_value(value: &'v str) -> Self {
        ValueField::from(("", value))
    }

    pub fn shift(mut self) -> Self {
        self.name.shift();
        self
    }

    pub fn unexpected(&self) -> Error<'v> {
        Error::from(ErrorKind::Unexpected)
            .with_name(NameView::new(self.name.source()))
            .with_value(self.value)
            .with_entity(Entity::ValueField)
    }

    pub fn missing(&self) -> Error<'v> {
        Error::from(ErrorKind::Missing)
            .with_name(NameView::new(self.name.source()))
            .with_value(self.value)
            .with_entity(Entity::ValueField)
    }
}

impl<'a> From<(&'a str, &'a str)> for ValueField<'a> {
    fn from((name, value): (&'a str, &'a str)) -> Self {
        ValueField { name: NameView::new(name), value }
    }
}

impl<'a, 'b> PartialEq<ValueField<'b>> for ValueField<'a> {
    fn eq(&self, other: &ValueField<'b>) -> bool {
        self.name == other.name && self.value == other.value
    }
}

impl<'v> DataField<'v, '_> {
    pub fn shift(mut self) -> Self {
        self.name.shift();
        self
    }

    pub fn unexpected(&self) -> Error<'v> {
        Error::from(ErrorKind::Unexpected)
            .with_name(self.name)
            .with_entity(Entity::DataField)
    }
}
