use std::{slice::Iter, iter::Cloned};

use request::{FormItems, FormItem, Form, LenientForm, FromForm};

pub struct Query<'q>(pub &'q [FormItem<'q>]);

impl<'q> IntoIterator for Query<'q> {
    type Item = FormItem<'q>;
    type IntoIter = Cloned<Iter<'q, FormItem<'q>>>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter().cloned()
    }
}

pub trait FromQuery<'q>: Sized {
    type Error;

    fn from_query(q: Query<'q>) -> Result<Self, Self::Error>;
}

impl<'q, T: FromForm<'q>> FromQuery<'q> for Form<T> {
    type Error = T::Error;

    #[inline]
    fn from_query(q: Query<'q>) -> Result<Self, Self::Error> {
        T::from_form(&mut FormItems::from(q.0), true).map(Form)
    }
}

impl<'q, T: FromForm<'q>> FromQuery<'q> for LenientForm<T> {
    type Error = <T as FromForm<'q>>::Error;

    #[inline]
    fn from_query(q: Query<'q>) -> Result<Self, Self::Error> {
        T::from_form(&mut FormItems::from(q.0), false).map(LenientForm)
    }
}

impl<'q, T: FromQuery<'q>> FromQuery<'q> for Option<T> {
    type Error = !;

    #[inline]
    fn from_query(q: Query<'q>) -> Result<Self, Self::Error> {
        Ok(T::from_query(q).ok())
    }
}

impl<'q, T: FromQuery<'q>> FromQuery<'q> for Result<T, T::Error> {
    type Error = !;

    #[inline]
    fn from_query(q: Query<'q>) -> Result<Self, Self::Error> {
        Ok(T::from_query(q))
    }
}
