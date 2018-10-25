use std::fmt;

use smallvec::SmallVec;

use uri::UriDisplay;

pub struct Formatter<'i, 'f: 'i> {
    crate prefixes: SmallVec<[&'static str; 3]>,
    crate inner: &'i mut fmt::Formatter<'f>,
    crate previous: bool,
    crate fresh: bool
}

impl<'i, 'f: 'i> Formatter<'i, 'f> {
    pub fn write_raw<S: AsRef<str>>(&mut self, s: S) -> fmt::Result {
        let s = s.as_ref();
        if self.fresh && !self.prefixes.is_empty() {
            if self.previous {
                self.inner.write_str("&")?;
            }

            self.fresh = false;
            self.previous = true;

            for (i, prefix) in self.prefixes.iter().enumerate() {
                self.inner.write_str(prefix)?;
                if i < self.prefixes.len() - 1 {
                    self.inner.write_str(".")?;
                }
            }

            self.inner.write_str("=")?;
        }

        self.inner.write_str(s)
    }

    fn with_prefix<F>(&mut self, prefix: &str, f: F) -> fmt::Result
        where F: FnOnce(&mut Self) -> fmt::Result
    {
        self.fresh = true;

        // TODO: PROOF OF CORRECTNESS.
        let prefix: &'static str = unsafe { ::std::mem::transmute(prefix) };

        self.prefixes.push(prefix);
        let result = f(self);
        self.prefixes.pop();

        result
    }

    #[inline]
    pub fn write_seq_value<T: UriDisplay>(&mut self, value: T) -> fmt::Result {
        self.fresh = true;
        self.write_value(value)
    }

    #[inline]
    pub fn write_named_value<T: UriDisplay>(&mut self, name: &str, value: T) -> fmt::Result {
        self.with_prefix(name, |f| f.write_value(value))
    }

    #[inline]
    pub fn write_value<T: UriDisplay>(&mut self, value: T) -> fmt::Result {
        UriDisplay::fmt(&value, self)
    }
}

impl<'f, 'i: 'f> fmt::Write for Formatter<'f, 'i> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_raw(s)
    }
}
