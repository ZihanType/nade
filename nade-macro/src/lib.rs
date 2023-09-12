mod argument;
mod maybe_starts_with_dollar;
mod nade;
mod nade_helper;
mod parameter;
mod parameter_doc;

use maybe_starts_with_dollar::StartsWithDollar;
use nade_helper::NadeHelper;
use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{parse_macro_input, ItemFn, Path};

#[proc_macro_attribute]
pub fn nade(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut fun = parse_macro_input!(item as ItemFn);

    let module_path = if attr.is_empty() {
        None
    } else {
        Some(parse_macro_input!(attr as StartsWithDollar::<Path>))
    };

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
