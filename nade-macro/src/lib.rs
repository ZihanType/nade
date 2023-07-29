mod argument;
mod maybe_starts_with_dollar;
mod nade;
mod nade_helper;
mod parameter;
mod parameter_doc;

use maybe_starts_with_dollar::MaybeStartsWithDollar;
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
        Some(
            MaybeStartsWithDollar::<Path>::try_from(attr)
                .map(|maybe| maybe.require_starts_with_dollar())
                .and_then(|i| i),
        )
    };

    module_path
        .transpose()
        .map(|module_path| nade::generate(module_path, &mut fun))
        .and_then(|i| i)
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
