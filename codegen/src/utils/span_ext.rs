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
    fn trim_left(self, length: usize) -> Span {
        self.with_lo(self.lo() + BytePos(length as u32))
    }

    fn trim_right(self, length: usize) -> Span {
        self.with_hi(self.hi() - BytePos(length as u32))
    }

    fn shorten_to(self, to_length: usize) -> Span {
        self.with_hi(self.lo() + BytePos(to_length as u32))
    }

    fn shorten_upto(self, length: usize) -> Span {
        self.with_lo(self.hi() - BytePos(length as u32))
    }

    fn trim(self, length: u32) -> Span {
        self.with_lo(self.lo() + BytePos(length))
            .with_hi(self.hi() - BytePos(length))
    }

    fn wrap<T>(self, node: T) -> Spanned<T> {
        Spanned { node: node, span: self }
    }

    fn expand(self, left: usize, right: usize) -> Span {
        self.with_lo(self.lo() + BytePos(left as u32))
            .with_hi(self.lo() + BytePos(right as u32))
    }
}
