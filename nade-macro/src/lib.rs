mod argument;
mod helper;
mod maybe_starts_with_dollar;
mod nade;
mod parameter;
mod parameter_doc;

use helper::Helper;
use maybe_starts_with_dollar::MaybeStartsWithDollar;
use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{parse_macro_input, ItemFn, Path};

#[proc_macro_attribute]
pub fn nade(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut fun = parse_macro_input!(item as ItemFn);

    let module_path = if attr.is_empty() {
        None
    } else {
        let get_starts_with_dollar_path = || -> Result<_, _> {
            MaybeStartsWithDollar::<Path>::try_from(attr)?.starts_with_dollar()
        };
        match get_starts_with_dollar_path() {
            Ok(p) => Some(p),
            Err(e) => {
                let mut stream = e.to_compile_error();
                stream.extend(fun.to_token_stream());
                return stream.into();
            }
        }
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
pub fn helper(input: TokenStream) -> TokenStream {
    let helper = parse_macro_input!(input as Helper);

    helper::generate(helper)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
