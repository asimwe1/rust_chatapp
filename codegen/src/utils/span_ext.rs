use syntax::codemap::{Span, Spanned, BytePos};

pub trait SpanExt {
    /// Trim the span on the left and right by `length`.
    fn trim(self, length: u32) -> Span;

    /// Trim the span on the left by `length`.
    fn trim_left(self, length: usize) -> Span;

    /// Trim the span on the right by `length`.
    fn trim_right(self, length: usize) -> Span;

    // Trim from the right so that the span is `length` in size.
    fn shorten_to(self, to_length: usize) -> Span;

    // Trim from the left so that the span is `length` in size.
    fn shorten_upto(self, length: usize) -> Span;

    // Wrap `T` into a `Spanned<T>` with `self` as the span.
    fn wrap<T>(self, node: T) -> Spanned<T>;

    /// Expand the span on the left by `left` and right by `right`.
    fn expand(self, left: usize, right: usize) -> Span;
}

impl SpanExt for Span {
    fn trim_left(mut self, length: usize) -> Span {
        self.lo = self.lo + BytePos(length as u32);
        self
    }

    fn trim_right(mut self, length: usize) -> Span {
        self.hi = self.hi - BytePos(length as u32);
        self
    }

    fn shorten_to(mut self, to_length: usize) -> Span {
        self.hi = self.lo + BytePos(to_length as u32);
        self
    }

    fn shorten_upto(mut self, length: usize) -> Span {
        self.lo = self.hi - BytePos(length as u32);
        self
    }

    fn trim(mut self, length: u32) -> Span {
        self.lo = self.lo + BytePos(length);
        self.hi = self.hi - BytePos(length);
        self
    }

    fn wrap<T>(self, node: T) -> Spanned<T> {
        Spanned { node: node, span: self }
    }

    fn expand(mut self, left: usize, right: usize) -> Span {
        self.lo = self.lo + BytePos(left as u32);
        self.hi = self.lo + BytePos(right as u32);
        self
    }
}
