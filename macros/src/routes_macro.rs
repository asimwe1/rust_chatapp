use super::{STRUCT_PREFIX};
use utils::{prepend_ident, token_separate};
use syntax::codemap::Span;
use syntax::ast::{Path, TokenTree, Expr};
use syntax::ext::base::{ExtCtxt, MacResult, MacEager};
use syntax::ext::build::AstBuilder;
use syntax::parse::parser::{Parser, PathParsingMode};
use syntax::parse::PResult;
use syntax::parse::token::Token;
use syntax::ptr::P;

const DEBUG: bool = false;

fn get_paths<'a>(parser: &mut Parser<'a>) -> PResult<'a, Vec<Path>> {
    if parser.eat(&Token::Eof) {
        return Ok(vec![]);
    }

    let mut results = Vec::new();
    loop {
        results.push(try!(parser.parse_path(PathParsingMode::NoTypesAllowed)));
        if !parser.eat(&Token::Comma) {
            try!(parser.expect(&Token::Eof));
            break;
        }
    }

    Ok(results)
}

pub fn routes_macro(ecx: &mut ExtCtxt, _sp: Span, args: &[TokenTree])
        -> Box<MacResult + 'static> {
    let mut parser = ecx.new_parser_from_tts(args);
    let mut paths = get_paths(&mut parser).unwrap_or_else(|mut e| {
        e.emit();
        vec![]
    });

    // Prefix each path terminator with STRUCT_PREFIX.
    for p in &mut paths {
        let last = p.segments.len() - 1;
        let last_seg = &mut p.segments[last];
        let new_ident = prepend_ident(STRUCT_PREFIX, &last_seg.identifier);
        last_seg.identifier = new_ident;
    }

    debug!("Found paths: {:?}", paths);
    // Build up the P<Expr> for each path.
    let path_exprs: Vec<P<Expr>> = paths.iter().map(|p| {
        quote_expr!(ecx, rocket::Route::from(&$p))
    }).collect();

    debug!("Path Exprs: {:?}", path_exprs);
    // Now put them all in one vector and return the thing.
    let path_list = token_separate(ecx, &path_exprs, Token::Comma);
    let output = quote_expr!(ecx, vec![$path_list]).unwrap();
    MacEager::expr(P(output))
}
