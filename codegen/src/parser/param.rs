use syntax::ast::Ident;
use syntax::codemap::{Span, Spanned};
use syntax::ext::base::ExtCtxt;
use utils::SpanExt;
use syntax::parse::PResult;

#[derive(Debug)]
pub enum Param {
    Single(Spanned<Ident>),
    Many(Spanned<Ident>),
}

impl Param {
    pub fn inner(&self) -> &Spanned<Ident> {
        match *self {
            Param::Single(ref ident) | Param::Many(ref ident) => ident,
        }
    }

    pub fn ident(&self) -> &Ident {
        match *self {
            Param::Single(ref ident) | Param::Many(ref ident) => &ident.node,
        }
    }

    pub fn parse_many<'a>(
        ecx: &ExtCtxt<'a>,
        mut string: &str,
        mut span: Span
    ) -> PResult<'a, Vec<Param>> {
        let err = |sp, msg| { Err(ecx.struct_span_err(sp, msg)) };

        let mut params = vec![];
        loop {
            // Find the start and end indexes for the next parameter, if any.
            let (start, end) = match string.find('<') {
                Some(i) => match string.find('>') {
                    Some(j) if j > i => (i, j),
                    Some(_) => return err(span, "malformed parameters"),
                    None => return err(span, "malformed parameters")
                },
                _ => return Ok(params)
            };

            // Calculate the parameter's ident and span.
            let param_span = span.trim_left(start).shorten_to(end + 1);
            let full_param = &string[(start + 1)..end];
            let (is_many, param) = if full_param.ends_with("..") {
                (true, &full_param[..(full_param.len() - 2)])
            } else {
                (false, full_param)
            };

            // Advance the string and span.
            string = &string[(end + 1)..];
            span = span.trim_left(end + 1);

            let spanned_ident = param_span.wrap(Ident::from_str(param));
            if is_many {
                params.push(Param::Many(spanned_ident))
            } else {
                params.push(Param::Single(spanned_ident))
            }
        }
    }
}
