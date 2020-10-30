use std::{borrow::Cow, convert::TryInto, fmt::Display, ops::{RangeBounds, Bound}};

use rocket_http::ContentType;

use crate::{data::TempFile, form::error::{Error, Errors}};

pub fn eq<'v, A, B>(a: &A, b: B) -> Result<(), Errors<'v>>
    where A: PartialEq<B>
{
    if a != &b {
        Err(Error::validation("value does not match"))?
    }

    Ok(())
}

pub trait Len {
    fn len(&self) -> usize;

    fn len_u64(&self) -> u64 {
        self.len() as u64
    }
}

impl Len for str {
    fn len(&self) -> usize { self.len() }
}

impl Len for String {
    fn len(&self) -> usize { self.len() }
}

impl<T> Len for Vec<T> {
    fn len(&self) -> usize { <Vec<T>>::len(self) }
}

impl Len for TempFile<'_> {
    fn len(&self) -> usize { TempFile::len(self) as usize }

    fn len_u64(&self) -> u64 { TempFile::len(self) }
}

impl<K, V> Len for std::collections::HashMap<K, V> {
    fn len(&self) -> usize { <std::collections::HashMap<K, V>>::len(self) }
}

impl<T: Len + ?Sized> Len for &T {
    fn len(&self) -> usize {
        <T as Len>::len(self)
    }
}

pub fn len<'v, V, R>(value: V, range: R) -> Result<(), Errors<'v>>
    where V: Len, R: RangeBounds<u64>
{
    if !range.contains(&value.len_u64()) {
        let start = match range.start_bound() {
            Bound::Included(v) => Some(*v),
            Bound::Excluded(v) => Some(v.saturating_add(1)),
            Bound::Unbounded => None
        };

        let end = match range.end_bound() {
            Bound::Included(v) => Some(*v),
            Bound::Excluded(v) => Some(v.saturating_sub(1)),
            Bound::Unbounded => None,
        };

        Err((start, end))?
    }

    Ok(())
}

pub trait Contains<I> {
    fn contains(&self, item: I) -> bool;
}

impl<I, T: Contains<I>> Contains<I> for &T {
    fn contains(&self, item: I) -> bool {
        <T as Contains<I>>::contains(self, item)
    }
}

impl Contains<&str> for str {
    fn contains(&self, string: &str) -> bool {
        <str>::contains(self, string)
    }
}

impl Contains<&&str> for str {
    fn contains(&self, string: &&str) -> bool {
        <str>::contains(self, string)
    }
}

impl Contains<char> for str {
    fn contains(&self, c: char) -> bool {
        <str>::contains(self, c)
    }
}

impl Contains<&char> for str {
    fn contains(&self, c: &char) -> bool {
        <str>::contains(self, *c)
    }
}

impl Contains<&str> for &str {
    fn contains(&self, string: &str) -> bool {
        <str>::contains(self, string)
    }
}

impl Contains<&&str> for &str {
    fn contains(&self, string: &&str) -> bool {
        <str>::contains(self, string)
    }
}

impl Contains<char> for &str {
    fn contains(&self, c: char) -> bool {
        <str>::contains(self, c)
    }
}

impl Contains<&char> for &str {
    fn contains(&self, c: &char) -> bool {
        <str>::contains(self, *c)
    }
}

impl Contains<&str> for String {
    fn contains(&self, string: &str) -> bool {
        <str>::contains(self, string)
    }
}

impl Contains<&&str> for String {
    fn contains(&self, string: &&str) -> bool {
        <str>::contains(self, string)
    }
}

impl Contains<char> for String {
    fn contains(&self, c: char) -> bool {
        <str>::contains(self, c)
    }
}

impl Contains<&char> for String {
    fn contains(&self, c: &char) -> bool {
        <str>::contains(self, *c)
    }
}

impl<T: PartialEq> Contains<T> for Vec<T> {
    fn contains(&self, item: T) -> bool {
        <[T]>::contains(self, &item)
    }
}

impl<T: PartialEq> Contains<&T> for Vec<T> {
    fn contains(&self, item: &T) -> bool {
        <[T]>::contains(self, item)
    }
}

pub fn contains<'v, V, I>(value: V, item: I) -> Result<(), Errors<'v>>
    where V: for<'a> Contains<&'a I>, I: std::fmt::Debug
{
    if !value.contains(&item) {
        Err(Error::validation(format!("must contain {:?}", item)))?
    }

    Ok(())
}

pub fn omits<'v, V, I>(value: V, item: I) -> Result<(), Errors<'v>>
    where V: for<'a> Contains<&'a I>, I: std::fmt::Debug
{
    if value.contains(&item) {
        Err(Error::validation(format!("cannot contain {:?}", item)))?
    }

    Ok(())
}

pub fn range<'v, V, R>(value: &V, range: R) -> Result<(), Errors<'v>>
    where V: TryInto<isize> + Copy, R: RangeBounds<isize>
{
    if let Ok(v) = (*value).try_into() {
        if range.contains(&v) {
            return Ok(());
        }
    }

    let start = match range.start_bound() {
        Bound::Included(v) => Some(*v),
        Bound::Excluded(v) => Some(v.saturating_add(1)),
        Bound::Unbounded => None
    };

    let end = match range.end_bound() {
        Bound::Included(v) => Some(*v),
        Bound::Excluded(v) => Some(v.saturating_sub(1)),
        Bound::Unbounded => None,
    };


    Err((start, end))?
}

pub fn one_of<'v, V, I>(value: V, items: &[I]) -> Result<(), Errors<'v>>
    where V: for<'a> Contains<&'a I>, I: Display
{
    for item in items {
        if value.contains(item) {
            return Ok(());
        }
    }

    let choices = items.iter()
        .map(|item| item.to_string().into())
        .collect::<Vec<Cow<'v, str>>>();

    Err(choices)?
}

pub fn ext<'v>(file: &TempFile<'_>, ext: &str) -> Result<(), Errors<'v>> {
    if let Some(file_ct) = file.content_type() {
        if let Some(ext_ct) = ContentType::from_extension(ext) {
            if file_ct == &ext_ct {
                return Ok(());
            }

            let m = file_ct.extension()
                .map(|fext| format!("file type was .{} but must be .{}", fext, ext))
                .unwrap_or_else(|| format!("file type must be .{}", ext));

            Err(Error::validation(m))?
        }
    }

    Err(Error::validation(format!("invalid extension: expected {}", ext)))?
}
