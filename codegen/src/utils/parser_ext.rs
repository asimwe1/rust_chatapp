use super::SpanExt;

use syntax::parse::parser::{PathStyle, Parser};
use syntax::parse::PResult;
use syntax::ast::{self, Path, StrStyle, Ident};
use syntax::parse::token::Token::{Eof, Comma};
use syntax::parse::common::SeqSep;
use syntax::codemap::Spanned;
use syntax::symbol::Symbol;

pub trait ParserExt<'a> {
    // Parse a comma-seperated list of paths: `a::b, b::c`.
    fn parse_paths(&mut self) -> PResult<'a, Vec<Path>>;

    // Just like `parse_str` but takes into account interpolated expressions.
    fn parse_str_lit(&mut self) -> PResult<'a, (Symbol, StrStyle)>;

    // Like `parse_ident` but also looks for an `ident` in a `Pat`.
    fn parse_ident_inc_pat(&mut self) -> PResult<'a, Spanned<Ident>>;
}

impl<'a> ParserExt<'a> for Parser<'a> {
    fn parse_paths(&mut self) -> PResult<'a, Vec<Path>> {
        self.parse_seq_to_end(&Eof,
                              SeqSep::trailing_allowed(Comma),
                              |p| p.parse_path(PathStyle::Mod))
    }

    fn parse_str_lit(&mut self) -> PResult<'a, (Symbol, StrStyle)> {
        self.parse_str()
            .or_else(|mut e| {
                let expr = self.parse_expr().map_err(|i| { e.cancel(); i })?;
                let string_lit = match expr.node {
                    ast::ExprKind::Lit(ref lit) => match lit.node {
                        ast::LitKind::Str(symbol, style) => (symbol, style),
                        _ => return Err(e)
                    }
                    _ => return Err(e)
                };

                e.cancel();
                Ok(string_lit)
            })
    }

    fn parse_ident_inc_pat(&mut self) -> PResult<'a, Spanned<Ident>> {
        self.parse_ident()
            .map(|ident| self.prev_span.wrap(ident))
            .or_else(|mut e| {
                let pat = self.parse_pat().map_err(|i| { e.cancel(); i })?;
                let ident = match pat.node {
                    ast::PatKind::Ident(_, ident, _) => ident,
                    _ => return Err(e)
                };

                e.cancel();
                Ok(ident)
            })
    }
}
