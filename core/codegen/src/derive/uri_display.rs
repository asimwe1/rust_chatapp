use proc_macro::{Span, TokenStream};
use devise::*;

use derive::from_form::Form;

const NO_EMPTY_FIELDS: &str = "fieldless structs or variants are not supported";
const NO_NULLARY: &str = "nullary items are not supported";
const NO_EMPTY_ENUMS: &str = "empty enums are not supported";
const ONLY_ONE_UNNAMED: &str = "tuple structs or variants must have exactly one field";
const EXACTLY_ONE_FIELD: &str = "struct must have exactly one field";

fn validate_fields(fields: Fields, parent_span: Span) -> Result<()> {
    if fields.count() == 0 {
        return Err(parent_span.error(NO_EMPTY_FIELDS))
    } else if fields.are_unnamed() && fields.count() > 1 {
        return Err(fields.span().error(ONLY_ONE_UNNAMED));
    } else if fields.are_unit() {
        return Err(parent_span.error(NO_NULLARY));
    }

    Ok(())
}

fn validate_struct(gen: &DeriveGenerator, data: Struct) -> Result<()> {
    validate_fields(data.fields(), gen.input.span())
}

fn validate_enum(gen: &DeriveGenerator, data: Enum) -> Result<()> {
    if data.variants().count() == 0 {
        return Err(gen.input.span().error(NO_EMPTY_ENUMS));
    }

    for variant in data.variants() {
        validate_fields(variant.fields(), variant.span())?;
    }

    Ok(())
}

pub fn derive_uri_display_query(input: TokenStream) -> TokenStream {
    let display_trait = quote!(::rocket::http::uri::UriDisplay<::rocket::http::uri::Query>);
    let formatter = quote!(::rocket::http::uri::Formatter<::rocket::http::uri::Query>);
    DeriveGenerator::build_for(input, quote!(impl #display_trait))
        .generic_support(GenericSupport::Type | GenericSupport::Lifetime)
        .data_support(DataSupport::Struct | DataSupport::Enum)
        .validate_enum(validate_enum)
        .validate_struct(validate_struct)
        .map_type_generic(move |_, ident, _| quote!(#ident : #display_trait))
        .function(move |_, inner| quote! {
            fn fmt(&self, f: &mut #formatter) -> ::std::fmt::Result {
                #inner
                Ok(())
            }
        })
        .try_map_field(|_, field| {
            let span = field.span().into();
            let accessor = field.accessor();
            let tokens = if let Some(ref ident) = field.ident {
                let name = Form::from_attrs("form", &field.attrs)
                    .map(|result| result.map(|form| form.field.name))
                    .unwrap_or_else(|| Ok(ident.to_string()))?;

                quote_spanned!(span => f.write_named_value(#name, &#accessor)?;)
            } else {
                quote_spanned!(span => f.write_value(&#accessor)?;)
            };

            Ok(tokens)
        })
        .to_tokens()
}

pub fn derive_uri_display_path(input: TokenStream) -> TokenStream {
    let display_trait = quote!(::rocket::http::uri::UriDisplay<::rocket::http::uri::Path>);
    let formatter = quote!(::rocket::http::uri::Formatter<::rocket::http::uri::Path>);
    DeriveGenerator::build_for(input, quote!(impl #display_trait))
        .data_support(DataSupport::TupleStruct)
        .generic_support(GenericSupport::Type | GenericSupport::Lifetime)
        .map_type_generic(move |_, ident, _| quote!(#ident : #display_trait))
        .validate_fields(|_, fields| match fields.count() {
            1 => Ok(()),
            _ => Err(fields.span().error(EXACTLY_ONE_FIELD))
        })
        .function(move |_, inner| quote! {
            fn fmt(&self, f: &mut #formatter) -> ::std::fmt::Result {
                #inner
                Ok(())
            }
        })
        .map_field(|_, field| {
            let span = field.span().into();
            let accessor = field.accessor();
            quote_spanned!(span => f.write_value(&#accessor)?;)
        })
        .to_tokens()
}
