use proc_macro::Span;

use derive_utils::syn;
use derive_utils::ext::TypeExt;
use quote::ToTokens;

use self::syn::{Expr, Ident, LitStr, Path, Token, Type};
use self::syn::spanned::Spanned as SynSpanned;
use self::syn::parse::{self, Parse, ParseStream};
use self::syn::punctuated::Punctuated;

use indexmap::IndexMap;

#[derive(Debug)]
enum Arg {
    Unnamed(Expr),
    Named(Ident, Expr),
}

#[derive(Debug)]
pub enum Args {
    Unnamed(Vec<Expr>),
    Named(Vec<(Ident, Expr)>),
}

// For an invocation that looks like:
//  uri!("/mount/point", this::route: e1, e2, e3);
//       ^-------------| ^----------| ^---------|
//           uri_params.mount_point |    uri_params.arguments
//                      uri_params.route_path
//
#[derive(Debug)]
pub struct UriParams {
    pub mount_point: Option<LitStr>,
    pub route_path: Path,
    pub arguments: Option<Args>,
}

#[derive(Debug)]
pub struct FnArg {
    pub ident: Ident,
    pub ty: Type,
}

pub enum Validation {
    // Number expected, what we actually got.
    Unnamed(usize, usize),
    // (Missing, Extra, Duplicate)
    Named(Vec<Ident>, Vec<Ident>, Vec<Ident>),
    // Everything is okay.
    Ok(Vec<Expr>)
}

// `fn_args` are the URI arguments (excluding guards) from the original route's
// handler in the order they were declared in the URI (`<first>/<second>`).
// `uri` is the full URI used in the origin route's attribute
#[derive(Debug)]
pub struct InternalUriParams {
    pub uri: String,
    pub fn_args: Vec<FnArg>,
    pub uri_params: UriParams,
}

impl Arg {
    fn is_named(&self) -> bool {
        match *self {
            Arg::Named(..) => true,
            Arg::Unnamed(_) => false,
        }
    }

    fn unnamed(self) -> Expr {
        match self {
            Arg::Unnamed(expr) => expr,
            _ => panic!("Called Arg::unnamed() on an Arg::named!"),
        }
    }

    fn named(self) -> (Ident, Expr) {
        match self {
            Arg::Named(ident, expr) => (ident, expr),
            _ => panic!("Called Arg::named() on an Arg::Unnamed!"),
        }
    }
}

impl UriParams {
    /// The Span to use when referring to all of the arguments.
    pub fn args_span(&self) -> Span {
        match self.arguments {
            Some(ref args) => {
                let (first, last) = match args {
                    Args::Unnamed(ref exprs) => {
                        (
                            exprs.first().unwrap().span().unstable(),
                            exprs.last().unwrap().span().unstable()
                        )
                    },
                    Args::Named(ref pairs) => {
                        (
                            pairs.first().unwrap().0.span().unstable(),
                            pairs.last().unwrap().1.span().unstable()
                        )
                    },
                };
                first.join(last).expect("join spans")
            },
            None => self.route_path.span().unstable(),
        }
    }
}

impl Parse for Arg {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let has_key = input.peek2(Token![=]);
        if has_key {
            let ident = input.parse::<Ident>()?;
            input.parse::<Token![=]>()?;
            let expr = input.parse::<Expr>()?;
            Ok(Arg::Named(ident, expr))
        } else {
            let expr = input.parse::<Expr>()?;
            Ok(Arg::Unnamed(expr))
        }
    }
}

impl Parse for UriParams {
    // Parses the mount point, if any, route identifier, and arguments.
    fn parse(input: ParseStream) -> parse::Result<Self> {
        if input.is_empty() {
            return Err(input.error("call to `uri!` cannot be empty"));
        }

        // Parse the mount point and suffixing ',', if any.
        let mount_point = if input.peek(LitStr) {
            let string = input.parse::<LitStr>()?;
            let value = string.value();
            if value.contains('<') || !value.starts_with('/') {
                return Err(parse::Error::new(string.span(), "invalid mount point; mount points must be static, absolute URIs: `/example`"));
            }
            input.parse::<Token![,]>()?;
            Some(string)
        } else {
            None
        };

        // Parse the route identifier, which must always exist.
        let route_path = input.parse::<Path>()?;

        // If there are no arguments, finish early.
        if !input.peek(Token![:]) {
            let arguments = None;
            return Ok(Self { mount_point, route_path, arguments });
        }

        let colon = input.parse::<Token![:]>()?;

        // Parse arguments
        let args_start = input.cursor();
        let arguments: Punctuated<Arg, Token![,]> = input.parse_terminated(Arg::parse)?;

        // A 'colon' was used but there are no arguments.
        if arguments.is_empty() {
            return Err(parse::Error::new(colon.span(), "expected argument list after `:`"));
        }

        // Ensure that both types of arguments were not used at once.
        let (mut homogeneous_args, mut prev_named) = (true, None);
        for arg in &arguments {
            match prev_named {
                Some(prev_named) => homogeneous_args = prev_named == arg.is_named(),
                None => prev_named = Some(arg.is_named()),
            }
        }

        if !homogeneous_args {
            // TODO: This error isn't showing up with the right span.
            return Err(parse::Error::new(args_start.token_stream().span(), "named and unnamed parameters cannot be mixed"));
        }

        // Create the `Args` enum, which properly types one-kind-of-argument-ness.
        let args = if prev_named.unwrap() {
            Args::Named(arguments.into_iter().map(|arg| arg.named()).collect())
        } else {
            Args::Unnamed(arguments.into_iter().map(|arg| arg.unnamed()).collect())
        };

        let arguments = Some(args);
        Ok(Self { mount_point, route_path, arguments })
    }
}

impl Parse for FnArg {
    fn parse(input: ParseStream) -> parse::Result<FnArg> {
        let ident = input.parse::<Ident>()?;
        input.parse::<Token![:]>()?;
        let mut ty = input.parse::<Type>()?;
        ty.strip_lifetimes();
        Ok(FnArg { ident, ty })
    }
}

impl Parse for InternalUriParams {
    fn parse(input: ParseStream) -> parse::Result<InternalUriParams> {
        let uri = input.parse::<LitStr>()?.value();
        //let uri = parser.prev_span.wrap(uri_str);
        input.parse::<Token![,]>()?;

        let content;
        syn::parenthesized!(content in input);
        let fn_args: Punctuated<FnArg, Token![,]> = content.parse_terminated(FnArg::parse)?;
        let fn_args = fn_args.into_iter().collect();

        input.parse::<Token![,]>()?;
        let uri_params = input.parse::<UriParams>()?;
        Ok(InternalUriParams { uri, fn_args, uri_params })
    }
}

impl InternalUriParams {
    pub fn fn_args_str(&self) -> String {
        self.fn_args.iter()
            .map(|&FnArg { ref ident, ref ty }| format!("{}: {}", ident, ty.clone().into_token_stream().to_string().trim()))
            .collect::<Vec<_>>()
            .join(", ")
    }

    pub fn validate(&self) -> Validation {
        let unnamed = |args: &Vec<Expr>| -> Validation {
            let (expected, actual) = (self.fn_args.len(), args.len());
            if expected != actual { Validation::Unnamed(expected, actual) }
            else { Validation::Ok(args.clone()) }
        };

        match self.uri_params.arguments {
            None => unnamed(&vec![]),
            Some(Args::Unnamed(ref args)) => unnamed(args),
            Some(Args::Named(ref args)) => {
                let mut params: IndexMap<Ident, Option<Expr>> = self.fn_args.iter()
                    .map(|&FnArg { ref ident, .. }| (ident.clone(), None))
                    .collect();

                let (mut extra, mut dup) = (vec![], vec![]);
                for &(ref ident, ref expr) in args {
                    match params.get_mut(ident) {
                        Some(ref entry) if entry.is_some() => dup.push(ident.clone()),
                        Some(entry) => *entry = Some(expr.clone()),
                        None => extra.push(ident.clone()),
                    }
                }

                let (mut missing, mut exprs) = (vec![], vec![]);
                for (name, expr) in params {
                    match expr {
                        Some(expr) => exprs.push(expr),
                        None => missing.push(name)
                    }
                }

                if (extra.len() + dup.len() + missing.len()) == 0 {
                    Validation::Ok(exprs)
                } else {
                    Validation::Named(missing, extra, dup)
                }
            }
        }
    }
}

