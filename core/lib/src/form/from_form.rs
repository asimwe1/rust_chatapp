use std::borrow::Cow;
use std::collections::{HashMap, BTreeMap};
use std::hash::Hash;

use either::Either;
use indexmap::IndexMap;

use crate::form::prelude::*;
use crate::http::uncased::AsUncased;

/// Trait for implementing form guards: types parseable from HTTP form fields.
///
/// Only form guards that are _collections_, that is, collect more than one form
/// field while parsing, should implement `FromForm`. All other types should
/// implement [`FromFormField`] instead, which offers a simplified interface to
/// parsing a single form field.
///
/// For a gentle introduction to forms in Rocket, see the [forms guide].
///
/// # Form Guards
///
/// A form guard is a guard that operates on form fields, typically those with a
/// particular name prefix. Form guards validate and parse form field data via
/// implementations of `FromForm`. In other words, a type is a form guard _iff_
/// it implements `FromFrom`.
///
/// Form guards are used as the inner type of the [`Form`] data guard:
///
/// ```rust
/// # use rocket::post;
/// use rocket::form::Form;
///
/// # type FormGuard = String;
/// #[post("/submit", data = "<var>")]
/// fn submit(var: Form<FormGuard>) { /* ... */ }
/// ```
///
/// # Deriving
///
/// This trait can, and largely _should_, be automatically derived. When
/// deriving `FromForm`, every field in the structure must implement
/// [`FromForm`]. Form fields with the struct field's name are [shifted] and
/// then pushed to the struct field's `FromForm` parser.
///
/// ```rust
/// use rocket::form::FromForm;
///
/// #[derive(FromForm)]
/// struct TodoTask<'r> {
///     #[field(validate = len(1..))]
///     description: &'r str,
///     #[field(name = "done")]
///     completed: bool
/// }
/// ```
///
/// For full details on deriving `FromForm`, see the [`FromForm` derive].
///
/// [`Form`]: crate::form::Form
/// [`FromForm`]: crate::form::FromForm
/// [`FromForm` derive]: ../derive.FromForm.html
/// [FromFormField]: crate::form::FromFormField
/// [`shift()`ed]: NameView::shift()
/// [`key()`]: NameView::key()
/// [forms guide]: https://rocket.rs/master/guide/requests/#forms
///
/// # Provided Implementations
///
/// Rocket implements `FromForm` for several types. Their behavior is documented
/// here.
///
///   * **`T` where `T: FromFormField`**
///
///     This includes types like `&str`, `usize`, and [`Date`](time::Date). See
///     [`FromFormField`] for details.
///
///   * **`Vec<T>` where `T: FromForm`**
///
///     Parses a sequence of `T`'s. A new `T` is created whenever the field
///     name's key changes or is empty; the previous `T` is finalized and errors
///     are stored. While the key remains the same and non-empty, form values
///     are pushed to the current `T` after being shifted. All collected errors
///     are returned at finalization, if any, or the successfully created vector
///     is returned.
///
///   * **`HashMap<K, V>` where `K: FromForm + Eq + Hash`, `V: FromForm`**
///
///     **`BTreeMap<K, V>` where `K: FromForm + Ord`, `V: FromForm`**
///
///     Parses a sequence of `(K, V)`'s. A new pair is created for every unique
///     first index of the key.
///
///     If the key has only one index (`map[index]=value`), the index itself is
///     pushed to `K`'s parser and the remaining shifted field is pushed to
///     `V`'s parser.
///
///     If the key has two indices (`map[index:k]=value` or
///     `map[index:v]=value`), the second index must start with `k` or `v`. If
///     the second index starts with `k`, the shifted field is pushed to `K`'s
///     parser. If the second index starts with `v`, the shifted field is pushed
///     to `V`'s parser. If the second index is anything else, an error is
///     created for the offending form field.
///
///     Errors are collected as they occur. Finalization finalizes all pairs and
///     returns errors, if any, or the map.
///
///   * **`Option<T>` where `T: FromForm`**
///
///     _This form guard always succeeds._
///
///     Forwards all pushes to `T` without shifting. Finalizes successfully as
///     `Some(T)` if `T` finalizes without error or `None` otherwise.
///
///   * **`Result<T, Errors<'r>>` where `T: FromForm`**
///
///     _This form guard always succeeds._
///
///     Forwards all pushes to `T` without shifting. Finalizes successfully as
///     `Some(T)` if `T` finalizes without error or `Err(Errors)` with the
///     errors from `T` otherwise.
///
/// # Push Parsing
///
/// `FromForm` describes a 3-stage push-based interface to form parsing. After
/// preprocessing (see [the top-level docs](crate::form#parsing)), the three
/// stages are:
///
///   1. **Initialization.** The type sets up a context for later `push`es.
///
///      ```rust
///      # use rocket::form::prelude::*;
///      # struct Foo;
///      use rocket::form::Options;
///
///      # #[rocket::async_trait]
///      # impl<'r> FromForm<'r> for Foo {
///          # type Context = std::convert::Infallible;
///      fn init(opts: Options) -> Self::Context {
///          todo!("return a context for storing parse state")
///      }
///          # fn push_value(ctxt: &mut Self::Context, field: ValueField<'r>) { todo!() }
///          # async fn push_data(ctxt: &mut Self::Context, field: DataField<'r, '_>) { todo!() }
///          # fn finalize(ctxt: Self::Context) -> Result<'r, Self> { todo!() }
///      # }
///      ```
///
///   2. **Push.** The structure is repeatedly pushed form fields; the latest
///      context is provided with each `push`. If the structure contains
///      children, it uses the first [`key()`] to identify a child to which it
///      then `push`es the remaining `field` to, likely with a [`shift()`ed]
///      name. Otherwise, the structure parses the `value` itself. The context
///      is updated as needed.
///
///      ```rust
///      # use rocket::form::prelude::*;
///      # struct Foo;
///      use rocket::form::{ValueField, DataField};
///
///      # #[rocket::async_trait]
///      # impl<'r> FromForm<'r> for Foo {
///          # type Context = std::convert::Infallible;
///          # fn init(opts: Options) -> Self::Context { todo!() }
///      fn push_value(ctxt: &mut Self::Context, field: ValueField<'r>) {
///          todo!("modify context as necessary for `field`")
///      }
///
///      async fn push_data(ctxt: &mut Self::Context, field: DataField<'r, '_>) {
///          todo!("modify context as necessary for `field`")
///      }
///          # fn finalize(ctxt: Self::Context) -> Result<'r, Self> { todo!() }
///      # }
///      ```
///
///   3. **Finalization.** The structure is informed that there are no further
///      fields. It systemizes the effects of previous `push`es via its context
///      to return a parsed structure or generate [`Errors`].
///
///      ```rust
///      # use rocket::form::prelude::*;
///      # struct Foo;
///      use rocket::form::Result;
///
///      # #[rocket::async_trait]
///      # impl<'r> FromForm<'r> for Foo {
///          # type Context = std::convert::Infallible;
///          # fn init(opts: Options) -> Self::Context { todo!() }
///          # fn push_value(ctxt: &mut Self::Context, field: ValueField<'r>) { todo!() }
///          # async fn push_data(ctxt: &mut Self::Context, field: DataField<'r, '_>) { todo!() }
///      fn finalize(ctxt: Self::Context) -> Result<'r, Self> {
///          todo!("inspect context to generate `Self` or `Errors`")
///      }
///      # }
///      ```
///
/// These three stages make up the entirety of the `FromForm` trait.
///
/// ## Nesting and [`NameView`]
///
/// Each field name key typically identifies a unique child of a structure. As
/// such, when processed left-to-right, the keys of a field jointly identify a
/// unique leaf of a structure. The value of the field typically represents the
/// desired value of the leaf.
///
/// A [`NameView`] captures and simplifies this "left-to-right" processing of a
/// field's name by exposing a sliding-prefix view into a name. A [`shift()`]
/// shifts the view one key to the right. Thus, a `Name` of `a.b.c` when viewed
/// through a new [`NameView`] is `a`. Shifted once, the view is `a.b`.
/// [`key()`] returns the last (or "current") key in the view. A nested
/// structure can thus handle a field with a `NameView`, operate on the
/// [`key()`], [`shift()`] the `NameView`, and pass the field with the shifted
/// `NameView` to the next processor which handles `b` and so on.
///
/// [`shift()`]: NameView::shift()
/// [`key()`]: NameView::key()
///
/// # Implementing
///
/// Implementing `FromForm` should be a rare occurrence. Prefer instead to use
/// Rocket's built-in derivation or, for custom types, implementing
/// [`FromFormField`].
///
/// An implementation of `FromForm` consists of implementing the three stages
/// outlined above. `FromForm` is an async trait, so implementations must be
/// decorated with an attribute of `#[rocket::async_trait]`:
///
/// ```rust
/// # #[macro_use] extern crate rocket;
/// # struct MyType;
/// # struct MyContext;
/// use rocket::form::{self, FromForm, DataField, ValueField};
///
/// #[rocket::async_trait]
/// impl<'r> FromForm<'r> for MyType {
///     type Context = MyContext;
///
///     fn init(opts: form::Options) -> Self::Context {
///         todo!()
///     }
///
///     fn push_value(ctxt: &mut Self::Context, field: ValueField<'r>) {
///         todo!()
///     }
///
///     async fn push_data(ctxt: &mut Self::Context, field: DataField<'r, '_>) {
///         todo!()
///     }
///
///     fn finalize(this: Self::Context) -> form::Result<'r, Self> {
///         todo!()
///     }
/// }
/// ```
///
/// ## Lifetime
///
/// The lifetime `'r` correponds to the lifetime of the request.
///
/// ## Example
///
/// We illustrate implementation of `FromForm` through an example. The example
/// implements `FromForm` for a `Pair(A, B)` type where `A: FromForm` and `B:
/// FromForm`, parseable from forms with at least two fields, one with a key of
/// `0` and the other with a key of `1`. The field with key `0` is parsed as an
/// `A` while the field with key `1` is parsed as a `B`. Specifically, to parse
/// a `Pair(A, B)` from a field with prefix `pair`, a form with the following
/// fields must be submitted:
///
///   * `pair[0]` - type A
///   * `pair[1]` - type B
///
/// Examples include:
///
///   * `pair[0]=id&pair[1]=100` as `Pair(&str, usize)`
///   * `pair[0]=id&pair[1]=100` as `Pair(&str, &str)`
///   * `pair[0]=2012-10-12&pair[1]=100` as `Pair(time::Date, &str)`
///   * `pair.0=2012-10-12&pair.1=100` as `Pair(time::Date, usize)`
///
/// ```rust
/// use rocket::form::{self, FromForm, ValueField, DataField, Error, Errors};
/// use either::Either;
///
/// /// A form guard parseable from fields `.0` and `.1`.
/// struct Pair<A, B>(A, B);
///
/// // The parsing context. We'll be pushing fields with key `.0` to `left`
/// // and fields with `.1` to `right`. We'll collect errors along the way.
/// struct PairContext<'v, A: FromForm<'v>, B: FromForm<'v>> {
///     left: A::Context,
///     right: B::Context,
///     errors: Errors<'v>,
/// }
///
/// #[rocket::async_trait]
/// impl<'v, A: FromForm<'v>, B: FromForm<'v>> FromForm<'v> for Pair<A, B> {
///     type Context = PairContext<'v, A, B>;
///
///     // We initialize the `PairContext` as expected.
///     fn init(opts: form::Options) -> Self::Context {
///         PairContext {
///             left: A::init(opts),
///             right: B::init(opts),
///             errors: Errors::new()
///         }
///     }
///
///     // For each value, we determine if the key is `.0` (left) or `.1`
///     // (right) and push to the appropriate parser. If it was neither, we
///     // store the error for emission on finalization. The parsers for `A` and
///     // `B` will handle duplicate values and so on.
///     fn push_value(ctxt: &mut Self::Context, field: ValueField<'v>) {
///         match ctxt.context(field.name) {
///             Ok(Either::Left(ctxt)) => A::push_value(ctxt, field.shift()),
///             Ok(Either::Right(ctxt)) => B::push_value(ctxt, field.shift()),
///             Err(e) => ctxt.errors.push(e),
///         }
///     }
///
///     // This is identical to `push_value` but for data fields.
///     async fn push_data(ctxt: &mut Self::Context, field: DataField<'v, '_>) {
///         match ctxt.context(field.name) {
///             Ok(Either::Left(ctxt)) => A::push_data(ctxt, field.shift()).await,
///             Ok(Either::Right(ctxt)) => B::push_data(ctxt, field.shift()).await,
///             Err(e) => ctxt.errors.push(e),
///         }
///     }
///
///     // Finally, we finalize `A` and `B`. If both returned `Ok` and we
///     // encountered no errors during the push phase, we return our pair. If
///     // there were errors, we return them. If `A` and/or `B` failed, we
///     // return the commulative errors.
///     fn finalize(mut ctxt: Self::Context) -> form::Result<'v, Self> {
///         match (A::finalize(ctxt.left), B::finalize(ctxt.right)) {
///             (Ok(l), Ok(r)) if ctxt.errors.is_empty() => Ok(Pair(l, r)),
///             (Ok(_), Ok(_)) => Err(ctxt.errors),
///             (left, right) => {
///                 if let Err(e) = left { ctxt.errors.extend(e); }
///                 if let Err(e) = right { ctxt.errors.extend(e); }
///                 Err(ctxt.errors)
///             }
///         }
///     }
/// }
///
/// impl<'v, A: FromForm<'v>, B: FromForm<'v>> PairContext<'v, A, B> {
///     // Helper method used by `push_{value, data}`. Determines which context
///     // we should push to based on the field name's key. If the key is
///     // neither `0` nor `1`, we return an error.
///     fn context(
///         &mut self,
///         name: form::name::NameView<'v>
///     ) -> Result<Either<&mut A::Context, &mut B::Context>, Error<'v>> {
///         use std::borrow::Cow;
///
///         match name.key().map(|k| k.as_str()) {
///             Some("0") => Ok(Either::Left(&mut self.left)),
///             Some("1") => Ok(Either::Right(&mut self.right)),
///             _ => Err(Error::from(&[Cow::Borrowed("0"), Cow::Borrowed("1")])
///                 .with_entity(form::error::Entity::Index(0))
///                 .with_name(name)),
///         }
///     }
/// }
/// ```
#[crate::async_trait]
pub trait FromForm<'r>: Send + Sized {
    /// The form guard's parsing context.
    type Context: Send;

    /// Initializes and returns the parsing context for `Self`.
    fn init(opts: Options) -> Self::Context;

    /// Processes the value field `field`.
    fn push_value(ctxt: &mut Self::Context, field: ValueField<'r>);

    /// Processes the data field `field`.
    async fn push_data(ctxt: &mut Self::Context, field: DataField<'r, '_>);

    /// Processes the extern form or field error `_error`.
    ///
    /// The default implementation does nothing, which is always correct.
    fn push_error(_ctxt: &mut Self::Context, _error: Error<'r>) { }

    /// Finalizes parsing. Returns the parsed value when successful or
    /// collection of [`Errors`] otherwise.
    fn finalize(ctxt: Self::Context) -> Result<'r, Self>;

    /// Returns a default value, if any, to use when a value is desired and
    /// parsing fails.
    ///
    /// The default implementation initializes `Self` with lenient options and
    /// finalizes immediately, returning the value if finalization succeeds.
    fn default() -> Option<Self> {
        Self::finalize(Self::init(Options::Lenient)).ok()
    }
}

#[doc(hidden)]
pub struct VecContext<'v, T: FromForm<'v>> {
    opts: Options,
    last_key: Option<&'v Key>,
    current: Option<T::Context>,
    errors: Errors<'v>,
    items: Vec<T>
}

impl<'v, T: FromForm<'v>> VecContext<'v, T> {
    fn shift(&mut self) {
        if let Some(current) = self.current.take() {
            match T::finalize(current) {
                Ok(v) => self.items.push(v),
                Err(e) => self.errors.extend(e)
            }
        }
    }

    fn context(&mut self, name: &NameView<'v>) -> &mut T::Context {
        let this_key = name.key();
        let keys_match = match (self.last_key, this_key) {
            (Some(k1), Some(k2)) if k1 == k2 => true,
            _ => false
        };

        if !keys_match {
            self.shift();
            self.current = Some(T::init(self.opts));
        }

        self.last_key = name.key();
        self.current.as_mut().expect("must have current if last == index")
    }
}

#[crate::async_trait]
impl<'v, T: FromForm<'v> + 'v> FromForm<'v> for Vec<T> {
    type Context = VecContext<'v, T>;

    fn init(opts: Options) -> Self::Context {
        VecContext {
            opts,
            last_key: None,
            current: None,
            items: vec![],
            errors: Errors::new(),
        }
    }

    fn push_value(this: &mut Self::Context, field: ValueField<'v>) {
        T::push_value(this.context(&field.name), field.shift());
    }

    async fn push_data(ctxt: &mut Self::Context, field: DataField<'v, '_>) {
        T::push_data(ctxt.context(&field.name), field.shift()).await
    }

    fn finalize(mut this: Self::Context) -> Result<'v, Self> {
        this.shift();
        match this.errors.is_empty() {
            true => Ok(this.items),
            false => Err(this.errors)?,
        }
    }
}

#[doc(hidden)]
pub struct MapContext<'v, K, V> where K: FromForm<'v>, V: FromForm<'v> {
    opts: Options,
    /// Maps from the string key to the index in `map`.
    key_map: IndexMap<&'v str, (usize, NameView<'v>)>,
    keys: Vec<K::Context>,
    values: Vec<V::Context>,
    errors: Errors<'v>,
}

impl<'v, K, V> MapContext<'v, K, V>
    where K: FromForm<'v>, V: FromForm<'v>
{
    fn new(opts: Options) -> Self {
        MapContext {
            opts,
            key_map: IndexMap::new(),
            keys: vec![],
            values: vec![],
            errors: Errors::new(),
        }
    }

    fn ctxt(&mut self, key: &'v str, name: NameView<'v>) -> (&mut K::Context, &mut V::Context) {
        match self.key_map.get(key) {
            Some(&(i, _)) => (&mut self.keys[i], &mut self.values[i]),
            None => {
                debug_assert_eq!(self.keys.len(), self.values.len());
                let map_index = self.keys.len();
                self.keys.push(K::init(self.opts));
                self.values.push(V::init(self.opts));
                self.key_map.insert(key, (map_index, name));
                (self.keys.last_mut().unwrap(), self.values.last_mut().unwrap())
            }
        }
    }

    fn push(
        &mut self,
        name: NameView<'v>
    ) -> Option<Either<&mut K::Context, &mut V::Context>> {
        let index_pair = name.key()
            .map(|k| k.indices())
            .map(|mut i| (i.next(), i.next()))
            .unwrap_or_default();

        match index_pair {
            (Some(key), None) => {
                let is_new_key = !self.key_map.contains_key(key);
                let (key_ctxt, val_ctxt) = self.ctxt(key, name);
                if is_new_key {
                    K::push_value(key_ctxt, ValueField::from_value(key));
                }

                return Some(Either::Right(val_ctxt));
            },
            (Some(kind), Some(key)) => {
                if kind.as_uncased().starts_with("k") {
                    return Some(Either::Left(self.ctxt(key, name).0));
                } else if kind.as_uncased().starts_with("v") {
                    return Some(Either::Right(self.ctxt(key, name).1));
                } else {
                    let error = Error::from(&[Cow::Borrowed("k"), Cow::Borrowed("v")])
                        .with_entity(Entity::Index(0))
                        .with_name(name);

                    self.errors.push(error);
                }
            }
            _ => {
                let error = Error::from(ErrorKind::Missing)
                    .with_entity(Entity::Key)
                    .with_name(name);

                self.errors.push(error);
            }
        };

        None
    }

    fn push_value(&mut self, field: ValueField<'v>) {
        match self.push(field.name) {
            Some(Either::Left(ctxt)) => K::push_value(ctxt, field.shift()),
            Some(Either::Right(ctxt)) => V::push_value(ctxt, field.shift()),
            _ => {}
        }
    }

    async fn push_data(&mut self, field: DataField<'v, '_>) {
        match self.push(field.name) {
            Some(Either::Left(ctxt)) => K::push_data(ctxt, field.shift()).await,
            Some(Either::Right(ctxt)) => V::push_data(ctxt, field.shift()).await,
            _ => {}
        }
    }

    fn finalize<T: std::iter::FromIterator<(K, V)>>(self) -> Result<'v, T> {
        let (keys, values, key_map) = (self.keys, self.values, self.key_map);
        let errors = std::cell::RefCell::new(self.errors);

        let keys = keys.into_iter()
            .zip(key_map.values().map(|(_, name)| name))
            .filter_map(|(ctxt, name)| match K::finalize(ctxt) {
                Ok(value) => Some(value),
                Err(e) => { errors.borrow_mut().extend(e.with_name(*name)); None },
            });

        let values = values.into_iter()
            .zip(key_map.values().map(|(_, name)| name))
            .filter_map(|(ctxt, name)| match V::finalize(ctxt) {
                Ok(value) => Some(value),
                Err(e) => { errors.borrow_mut().extend(e.with_name(*name)); None },
            });

        let map: T = keys.zip(values).collect();
        let no_errors = errors.borrow().is_empty();
        match no_errors {
            true => Ok(map),
            false => Err(errors.into_inner())
        }
    }
}

#[crate::async_trait]
impl<'v, K, V> FromForm<'v> for HashMap<K, V>
    where K: FromForm<'v> + Eq + Hash, V: FromForm<'v>
{
    type Context = MapContext<'v, K, V>;

    fn init(opts: Options) -> Self::Context {
        MapContext::new(opts)
    }

    fn push_value(ctxt: &mut Self::Context, field: ValueField<'v>) {
        ctxt.push_value(field);
    }

    async fn push_data(ctxt: &mut Self::Context, field: DataField<'v, '_>) {
        ctxt.push_data(field).await;
    }

    fn finalize(this: Self::Context) -> Result<'v, Self> {
        this.finalize()
    }
}

#[crate::async_trait]
impl<'v, K, V> FromForm<'v> for BTreeMap<K, V>
    where K: FromForm<'v> + Ord, V: FromForm<'v>
{
    type Context = MapContext<'v, K, V>;

    fn init(opts: Options) -> Self::Context {
        MapContext::new(opts)
    }

    fn push_value(ctxt: &mut Self::Context, field: ValueField<'v>) {
        ctxt.push_value(field);
    }

    async fn push_data(ctxt: &mut Self::Context, field: DataField<'v, '_>) {
        ctxt.push_data(field).await;
    }

    fn finalize(this: Self::Context) -> Result<'v, Self> {
        this.finalize()
    }
}

#[crate::async_trait]
impl<'v, T: FromForm<'v>> FromForm<'v> for Option<T> {
    type Context = <T as FromForm<'v>>::Context;

    fn init(opts: Options) -> Self::Context {
        T::init(opts)
    }

    fn push_value(ctxt: &mut Self::Context, field: ValueField<'v>) {
        T::push_value(ctxt, field)
    }

    async fn push_data(ctxt: &mut Self::Context, field: DataField<'v, '_>) {
        T::push_data(ctxt, field).await
    }

    fn finalize(this: Self::Context) -> Result<'v, Self> {
        match T::finalize(this) {
            Ok(v) => Ok(Some(v)),
            Err(_) => Ok(None)
        }
    }
}

#[crate::async_trait]
impl<'v, T: FromForm<'v>> FromForm<'v> for Result<'v, T> {
    type Context = <T as FromForm<'v>>::Context;

    fn init(opts: Options) -> Self::Context {
        T::init(opts)
    }

    fn push_value(ctxt: &mut Self::Context, field: ValueField<'v>) {
        T::push_value(ctxt, field)
    }

    async fn push_data(ctxt: &mut Self::Context, field: DataField<'v, '_>) {
        T::push_data(ctxt, field).await
    }

    fn finalize(this: Self::Context) -> Result<'v, Self> {
        match T::finalize(this) {
            Ok(v) => Ok(Ok(v)),
            Err(e) => Ok(Err(e))
        }
    }
}

#[doc(hidden)]
pub struct PairContext<'v, A: FromForm<'v>, B: FromForm<'v>> {
    left: A::Context,
    right: B::Context,
    errors: Errors<'v>,
}

impl<'v, A: FromForm<'v>, B: FromForm<'v>> PairContext<'v, A, B> {
    fn context(
        &mut self,
        name: NameView<'v>
    ) -> std::result::Result<Either<&mut A::Context, &mut B::Context>, Error<'v>> {
        match name.key().map(|k| k.as_str()) {
            Some("0") => Ok(Either::Left(&mut self.left)),
            Some("1") => Ok(Either::Right(&mut self.right)),
            _ => Err(Error::from(&[Cow::Borrowed("0"), Cow::Borrowed("1")])
                .with_entity(Entity::Index(0))
                .with_name(name)),
        }
    }
}

#[crate::async_trait]
impl<'v, A: FromForm<'v>, B: FromForm<'v>> FromForm<'v> for (A, B) {
    type Context = PairContext<'v, A, B>;

    fn init(opts: Options) -> Self::Context {
        PairContext {
            left: A::init(opts),
            right: B::init(opts),
            errors: Errors::new()
        }
    }

    fn push_value(ctxt: &mut Self::Context, field: ValueField<'v>) {
        match ctxt.context(field.name) {
            Ok(Either::Left(ctxt)) => A::push_value(ctxt, field.shift()),
            Ok(Either::Right(ctxt)) => B::push_value(ctxt, field.shift()),
            Err(e) => ctxt.errors.push(e),
        }
    }

    async fn push_data(ctxt: &mut Self::Context, field: DataField<'v, '_>) {
        match ctxt.context(field.name) {
            Ok(Either::Left(ctxt)) => A::push_data(ctxt, field.shift()).await,
            Ok(Either::Right(ctxt)) => B::push_data(ctxt, field.shift()).await,
            Err(e) => ctxt.errors.push(e),
        }
    }

    fn finalize(mut ctxt: Self::Context) -> Result<'v, Self> {
        match (A::finalize(ctxt.left), B::finalize(ctxt.right)) {
            (Ok(key), Ok(val)) if ctxt.errors.is_empty() => Ok((key, val)),
            (Ok(_), Ok(_)) => Err(ctxt.errors)?,
            (left, right) => {
                if let Err(e) = left { ctxt.errors.extend(e); }
                if let Err(e) = right { ctxt.errors.extend(e); }
                Err(ctxt.errors)?
            }
        }
    }
}
