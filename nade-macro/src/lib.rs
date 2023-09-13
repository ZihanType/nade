mod argument;
mod maybe_starts_with_dollar;
mod nade;
mod nade_helper;
mod parameter;
mod parameter_doc;
mod path_attribute;

use nade_helper::NadeHelper;
use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{parse_macro_input, ItemFn, Path};

use crate::maybe_starts_with_dollar::StartsWithDollar;

#[proc_macro_attribute]
pub fn nade(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut fun = parse_macro_input!(item as ItemFn);

    let mut module_path: Option<StartsWithDollar<Path>> = None;

    let module_path_parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("module_path") {
            if module_path.is_some() {
                return Err(meta.error("duplicate `module_path` argument"));
            }
            module_path = Some(meta.value()?.parse()?);
            Ok(())
        } else {
            Err(meta.error("expected `module_path`"))
        }
    });

    parse_macro_input!(attr with module_path_parser);

    nade::generate(module_path, &mut fun)
        .unwrap_or_else(|e| {
            let mut stream = e.to_compile_error();
            stream.extend(fun.to_token_stream());
            stream
        })
        .into()
}

#[doc(hidden)]
#[proc_macro]
pub fn nade_helper(input: TokenStream) -> TokenStream {
    let nade_helper = parse_macro_input!(input as NadeHelper);

    nade_helper::generate(nade_helper)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
