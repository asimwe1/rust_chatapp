use serde::Serialize;
use indexmap::{IndexMap, IndexSet};

use crate::form::prelude::*;
use crate::http::Status;

/// An infallible form guard that records form fields while parsing any form
/// type.
#[derive(Debug)]
pub struct Contextual<'v, T> {
    pub value: Option<T>,
    pub context: Context<'v>
}

/// A form context containing received fields, values, and encountered errors.
///
/// # Serialization
///
/// When a value of this type is serialized, a `struct` or map with the
/// following fields is emitted:
///
/// | field         | type              | description                                    |
/// |---------------|-------------------|------------------------------------------------|
/// | `errors`      | &str => &[Error]  | map from a field name to errors it encountered |
/// | `values`      | &str => &[&str]   | map from a field name to its submitted values  |
/// | `data_values` | &[&str]           | field names of all data fields received        |
/// | `form_errors` | &[Error]          | errors not corresponding to specific fields    |
///
/// See [`Error`] for details on how an `Error` is serialized.
#[derive(Debug, Default, Serialize)]
pub struct Context<'v> {
    errors: IndexMap<NameBuf<'v>, Errors<'v>>,
    values: IndexMap<&'v Name, Vec<&'v str>>,
    data_values: IndexSet<&'v Name>,
    form_errors: Errors<'v>,
    #[serde(skip)]
    status: Status,
}

impl<'v> Context<'v> {
    pub fn value<N: AsRef<Name>>(&self, name: N) -> Option<&'v str> {
        self.values.get(name.as_ref())?.get(0).cloned()
    }

    pub fn values<'a, N>(&'a self, name: N) -> impl Iterator<Item = &'v str> + 'a
        where N: AsRef<Name>
    {
        self.values
            .get(name.as_ref())
            .map(|e| e.iter().cloned())
            .into_iter()
            .flatten()
    }

    pub fn has_error<N: AsRef<Name>>(&self, name: &N) -> bool {
        self.errors(name).next().is_some()
    }

    pub fn errors<'a, N>(&'a self, name: &'a N) -> impl Iterator<Item = &Error<'v>> + 'a
        where N: AsRef<Name> + ?Sized
    {
        let name = name.as_ref();
        name.prefixes()
            .filter_map(move |name| self.errors.get(name))
            .map(|e| e.iter())
            .flatten()
    }

    pub fn all_errors(&self) -> impl Iterator<Item = &Error<'v>> {
        self.errors.values()
            .map(|e| e.iter())
            .flatten()
            .chain(self.form_errors.iter())
    }

    pub fn status(&self) -> Status {
        self.status
    }

    pub(crate) fn push_error(&mut self, e: Error<'v>) {
        self.status = std::cmp::max(self.status, e.status());
        match e.name {
            Some(ref name) => match self.errors.get_mut(name) {
                Some(errors) => errors.push(e),
                None => { self.errors.insert(name.clone(), e.into()); },
            }
            None => self.form_errors.push(e)
        }
    }

    pub(crate) fn push_errors(&mut self, errors: Errors<'v>) {
        errors.into_iter().for_each(|e| self.push_error(e))
    }
}

impl<'f> From<Errors<'f>> for Context<'f> {
    fn from(errors: Errors<'f>) -> Self {
        let mut context = Context::default();
        context.push_errors(errors);
        context
    }
}

// impl<'v, T> From<Errors<'v>> for Contextual<'v, T> {
//     fn from(e: Errors<'v>) -> Self {
//         Contextual { value: None, context: Context::from(e) }
//     }
// }

// #[crate::async_trait]
// impl<'r, T: FromForm<'r>> FromData<'r> for Contextual<'r, T> {
//     type Error = std::convert::Infallible;
//
//     async fn from_data(req: &'r Request<'_>, data: Data) -> Outcome<Self, Self::Error> {
//         match Form::<Contextual<'r, T>>::from_data(req, data).await {
//             Outcome::Success(form) => Outcome::Success(form.into_inner()),
//             Outcome::Failure((_, e)) => Outcome::Success(Contextual::from(e)),
//             Outcome::Forward(d) => Outcome::Forward(d)
//         }
//     }
// }

#[crate::async_trait]
impl<'v, T: FromForm<'v>> FromForm<'v> for Contextual<'v, T> {
    type Context = (<T as FromForm<'v>>::Context, Context<'v>);

    fn init(opts: Options) -> Self::Context {
        (T::init(opts), Context::default())
    }

    fn push_value((ref mut val_ctxt, ctxt): &mut Self::Context, field: ValueField<'v>) {
        ctxt.values.entry(field.name.source()).or_default().push(field.value);
        T::push_value(val_ctxt, field);
    }

    async fn push_data(
        (ref mut val_ctxt, ctxt): &mut Self::Context,
        field: DataField<'v, '_>
    ) {
        ctxt.data_values.insert(field.name.source());
        T::push_data(val_ctxt, field).await;
    }

    fn push_error((_, ref mut ctxt): &mut Self::Context, e: Error<'v>) {
        ctxt.push_error(e);
    }

    fn finalize((val_ctxt, mut context): Self::Context) -> Result<'v, Self> {
        let value = match T::finalize(val_ctxt) {
            Ok(value) => Some(value),
            Err(errors) => {
                context.push_errors(errors);
                None
            }
        };

        Ok(Contextual { value, context })
    }
}
