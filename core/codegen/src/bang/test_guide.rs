use std::path::Path;
use std::error::Error;

use proc_macro::TokenStream;
use devise::{syn::{self, Ident, LitStr}, Result};

use crate::syn_ext::syn_to_diag;
use crate::proc_macro2::TokenStream as TokenStream2;

pub fn _macro(input: TokenStream) -> Result<TokenStream> {
    let root_glob = syn::parse::<LitStr>(input.into()).map_err(syn_to_diag)?;
    let modules = entry_to_modules(&root_glob)
        .map_err(|e| root_glob.span().unstable().error(format!("failed to read: {}", e)))?;

    Ok(quote_spanned!(root_glob.span() =>
        #[allow(dead_code)]
        #[allow(non_camel_case_types)]
        mod test_site_guide { #(#modules)* }
    ).into())
}

fn entry_to_modules(root_glob: &LitStr) -> std::result::Result<Vec<TokenStream2>, Box<dyn Error>> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("MANIFEST_DIR");
    let full_glob = Path::new(&manifest_dir).join(&root_glob.value()).display().to_string();

    let mut modules = vec![];
    for path in glob::glob(&full_glob).map_err(Box::new)? {
        let path = path.map_err(Box::new)?;
        let name = path.file_name()
            .and_then(|f| f.to_str())
            .map(|name| name.trim_matches(|c| char::is_numeric(c) || c == '-')
                .replace(|c| c == '-' || c == '.', "_"))
            .ok_or_else(|| "invalid file name")?;

        let ident = Ident::new(&name, root_glob.span());
        let full_path = Path::new(&manifest_dir).join(&path).display().to_string();
        modules.push(quote_spanned!(root_glob.span() =>
            #[doc(include = #full_path)]
            struct #ident;
        ))
    }

    Ok(modules)
}
