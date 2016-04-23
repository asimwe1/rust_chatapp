use syntax::parse::{token};
use syntax::parse::token::Token;
use syntax::ast::{Path, Ident, MetaItem, MetaItemKind, LitKind, TokenTree};
use syntax::ext::base::{ExtCtxt};
use syntax::codemap::{Span, Spanned, BytePos, DUMMY_SP};
use syntax::ext::quote::rt::ToTokens;
use syntax::parse::PResult;
use syntax::parse::parser::{PathParsingMode, Parser};
use syntax::ptr::P;

use std::collections::{HashSet, HashMap};

macro_rules! debug {
    ($($message:tt)*) => ({
        if DEBUG {
            println!("{}:{}", file!(), line!());
            println!($($message)*);
            println!("");
        }
    })
}

pub fn prepend_ident<T: ToString>(other: T, ident: &Ident) -> Ident {
    let mut new_ident = other.to_string();
    new_ident.push_str(ident.name.to_string().as_str());
    token::str_to_ident(new_ident.as_str())
}

#[allow(dead_code)]
pub fn append_ident<T: ToString>(ident: &Ident, other: T) -> Ident {
    let mut new_ident = ident.name.to_string();
    new_ident.push_str(other.to_string().as_str());
    token::str_to_ident(new_ident.as_str())
}

#[inline]
pub fn wrap_span<T>(t: T, span: Span) -> Spanned<T> {
    Spanned {
        span: span,
        node: t,
    }
}

#[inline]
pub fn dummy_span<T>(t: T) -> Spanned<T> {
    Spanned {
        span: DUMMY_SP,
        node: t,
    }
}

#[inline]
pub fn dummy_kvspan<T>(t: T) -> KVSpanned<T> {
    KVSpanned {
        k_span: DUMMY_SP,
        v_span: DUMMY_SP,
        p_span: DUMMY_SP,
        node: t,
    }
}

#[derive(Debug, Clone)]
pub struct KVSpanned<T> {
    pub k_span: Span,
    pub v_span: Span,
    pub p_span: Span,
    pub node: T
}

impl<T: ToTokens> ToTokens for KVSpanned<T> {
    fn to_tokens(&self, cx: &ExtCtxt) -> Vec<TokenTree> {
        self.node.to_tokens(cx)
    }
}

impl<T> KVSpanned<T> {
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> KVSpanned<U> {
        KVSpanned {
            k_span: self.k_span,
            v_span: self.v_span,
            p_span: self.p_span,
            node: f(self.node),
        }
    }
}

pub fn get_key_values<'b>(ecx: &mut ExtCtxt, sp: Span, required: &[&str],
        optional: &[&str], kv_params: &'b [P<MetaItem>])
            -> HashMap<&'b str, KVSpanned<&'b str>> {
    let mut seen = HashSet::new();
    let mut kv_pairs = HashMap::new();

    // Collect all the kv pairs, keeping track of what we've seen.
    for param in kv_params {
        if let MetaItemKind::NameValue(ref name, ref value) = param.node {
            if required.contains(&&**name) || optional.contains(&&**name) {
                if seen.contains(&**name) {
                    let msg = format!("'{}' parameter appears twice.", &**name);
                    ecx.span_err(param.span, &msg);
                    continue;
                }

                seen.insert(&**name);
                if let LitKind::Str(ref string, _) = value.node {
                    let mut k_span = param.span;
                    k_span.hi = k_span.lo + BytePos(name.len() as u32);
                    kv_pairs.insert(&**name, KVSpanned {
                        node: &**string,
                        k_span: k_span,
                        p_span: param.span,
                        v_span: value.span,
                    });
                } else {
                    ecx.span_err(value.span, "Value must be a string.");
                }
            } else {
                let msg = format!("'{}' is not a valid parameter.", &**name);
                ecx.span_err(param.span, &msg);
            }
        } else {
            ecx.span_err(param.span, "Expected 'key = value', found:");
        }
    }

    // Now, trigger an error for missing `required` params.
    for req in required {
        if !seen.contains(req) {
            let m = format!("'{}' parameter is required but is missing.", req);
            ecx.span_err(sp, &m);
        }
    }

    kv_pairs
}

pub fn token_separate<T: ToTokens>(ecx: &ExtCtxt, things: &[T],
                                   token: Token) -> Vec<TokenTree> {
    let mut output: Vec<TokenTree> = vec![];
    for (i, thing) in things.iter().enumerate() {
        output.extend(thing.to_tokens(ecx));
        if i < things.len() - 1 {
            output.push(TokenTree::Token(DUMMY_SP, token.clone()));
        }
    }

    output
}

pub fn assert_meta_item_list(ecx: &ExtCtxt, meta_item: &MetaItem, s: &str) {
    if !meta_item.node.is_list() {
        let msg = format!("Incorrect use of macro. Expected: #[{}(...)]", s);
        ecx.span_fatal(meta_item.span, msg.as_str());
    }
}

pub trait MetaItemExt {
    fn is_list(&self) -> bool;
    fn get_list_items(&self) -> Option<&Vec<P<MetaItem>>>;
}

impl MetaItemExt for MetaItemKind {
    fn is_list(&self) -> bool {
        self.get_list_items().is_some()
    }

    fn get_list_items(&self) -> Option<&Vec<P<MetaItem>>> {
        match *self {
            MetaItemKind::List(_, ref params) => Some(params),
            _ => None
        }
    }
}

pub fn parse_paths<'a>(parser: &mut Parser<'a>) -> PResult<'a, Vec<Path>> {
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

pub fn prefix_paths(prefix: &str, paths: &mut Vec<Path>) {
    for p in paths {
        let last = p.segments.len() - 1;
        let last_seg = &mut p.segments[last];
        let new_ident = prepend_ident(prefix, &last_seg.identifier);
        last_seg.identifier = new_ident;
    }
}

// pub fn find_value_for(key: &str, kv_params: &[P<MetaItem>]) -> Option<String> {
//     for param in kv_params {
//         if let MetaItemKind::NameValue(ref name, ref value) = param.node {
//             if &**name == key {
//                 if let LitKind::Str(ref string, _) = value.node {
//                     return Some(String::from(&**string));
//                 }
//             }
//         }
//     }

//     None
// }
