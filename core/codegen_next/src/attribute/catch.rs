use proc_macro::TokenStream;
use derive_utils::{syn, Spanned, Result, FromMeta};
use proc_macro2::TokenStream as TokenStream2;
use proc_macro::Span;

use http_codegen::Status;
use syn_ext::{syn_to_diag, IdentExt, ReturnTypeExt};
use self::syn::{Attribute, parse::Parser};

crate const CATCH_FN_PREFIX: &str = "rocket_catch_fn_";
crate const CATCH_STRUCT_PREFIX: &str = "static_rocket_catch_info_for_";

/// The raw, parsed `#[catch(code)]` attribute.
#[derive(Debug, FromMeta)]
struct CatchAttribute {
    #[meta(naked)]
    status: Status
}

/// This structure represents the parsed `catch` attribute an associated items.
struct CatchParams {
    /// The status associated with the code in the `#[catch(code)]` attribute.
    status: Status,
    /// The function that was decorated with the `catch` attribute.
    function: syn::ItemFn,
}

fn parse_params(args: TokenStream2, input: TokenStream) -> Result<CatchParams> {
    let function: syn::ItemFn = syn::parse(input).map_err(syn_to_diag)
        .map_err(|diag| diag.help("`#[catch]` can only be used on functions"))?;

    let full_attr = quote!(#[catch(#args)]);
    let attrs = Attribute::parse_outer.parse2(full_attr).map_err(syn_to_diag)?;
    let attribute = match CatchAttribute::from_attrs("catch", &attrs) {
        Some(result) => result.map_err(|d| {
            d.help("`#[catch]` expects a single status integer, e.g.: #[catch(404)]")
        })?,
        None => return Err(Span::call_site().error("internal error: bad attribute"))
    };

    Ok(CatchParams { status: attribute.status, function })
}

pub fn _catch(args: TokenStream, input: TokenStream) -> Result<TokenStream> {
    // Parse and validate all of the user's input.
    let catch = parse_params(TokenStream2::from(args), input)?;

    // Gather everything we'll need to generate the catcher.
    let user_catcher_fn = &catch.function;
    let mut user_catcher_fn_name = catch.function.ident.clone();
    let generated_struct_name = user_catcher_fn_name.prepend(CATCH_STRUCT_PREFIX);
    let generated_fn_name = user_catcher_fn_name.prepend(CATCH_FN_PREFIX);
    let (vis, status) = (&catch.function.vis, &catch.status);
    let status_code = status.0.code;

    // Determine the number of parameters that will be passed in.
    let (fn_sig, inputs) = match catch.function.decl.inputs.len() {
        0 => (quote!(fn() -> _), quote!()),
        1 => (quote!(fn(&::rocket::Request) -> _), quote!(__req)),
        _ => return Err(catch.function.decl.inputs.span()
                .error("invalid number of arguments: must be zero or one")
                .help("catchers may optionally take an argument of type `&Request`"))
    };

    // Set the span of the function name to point to inputs so that a later type
    // coercion failure points to the user's catcher's handler input.
    user_catcher_fn_name.set_span(catch.function.decl.inputs.span().into());

    // This ensures that "Responder not implemented" points to the return type.
    let return_type_span = catch.function.decl.output.ty()
        .map(|ty| ty.span().into())
        .unwrap_or(Span::call_site().into());

    let catcher_response = quote_spanned!(return_type_span => {
        // Check the type signature.
        let __catcher: #fn_sig = #user_catcher_fn_name;
        // Generate the response.
        ::rocket::response::Responder::respond_to(__catcher(#inputs), __req)?
    });

    // Generate the catcher, keeping the user's input around.
    Ok(quote! {
        #user_catcher_fn

        #vis fn #generated_fn_name<'_b>(
            _: ::rocket::Error,
            __req: &'_b ::rocket::Request
        ) -> ::rocket::response::Result<'_b> {
            let response = #catcher_response;
            ::rocket::response::Response::build()
                .status(#status)
                .merge(response)
                .ok()
        }

        #[allow(non_upper_case_globals)]
        #vis static #generated_struct_name: ::rocket::StaticCatchInfo =
            ::rocket::StaticCatchInfo {
                code: #status_code,
                handler: #generated_fn_name,
            };
    }.into())
}

pub fn catch_attribute(args: TokenStream, input: TokenStream) -> TokenStream {
    match _catch(args, input) {
        Ok(tokens) => tokens,
        Err(diag) => {
            diag.emit();
            TokenStream::new()
        }
    }
}
