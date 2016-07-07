use super::{CATCH_STRUCT_PREFIX};
use utils::*;
use syntax::codemap::Span;
use syntax::tokenstream::TokenTree;
use syntax::ast::Expr;
use syntax::ext::base::{ExtCtxt, MacResult, MacEager};
use syntax::parse::token::Token;
use syntax::ptr::P;

#[allow(dead_code)]
const DEBUG: bool = false;

pub fn errors_macro(ecx: &mut ExtCtxt, _sp: Span, args: &[TokenTree])
        -> Box<MacResult + 'static> {
    let mut parser = ecx.new_parser_from_tts(args);
    let mut paths = parse_paths(&mut parser).unwrap_or_else(|mut e| {
        e.emit();
        vec![]
    });

    // Prefix each path terminator
    prefix_paths(CATCH_STRUCT_PREFIX, &mut paths);

    // Build up the P<Expr> for each path.
    let path_exprs: Vec<P<Expr>> = paths.iter().map(|p| {
        quote_expr!(ecx, rocket::Catcher::from(&$p))
    }).collect();

    // Now put them all in one vector and return the thing.
    let path_list = token_separate(ecx, &path_exprs, Token::Comma);
    let output = quote_expr!(ecx, vec![$path_list]).unwrap();
    MacEager::expr(P(output))
}
