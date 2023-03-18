use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Path, Token,
};

use crate::{
    argument::Argument,
    maybe_starts_with_dollar::{MaybeStartsWithDollar, Normal},
    parameter::Parameter,
};

pub(crate) struct Helper {
    arguments: Punctuated<Argument, Token![,]>,
    parameters: Punctuated<Parameter, Token![,]>,
    fn_path: MaybeStartsWithDollar<Path>,
}

impl Parse for Helper {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let arguments_paren;
        let parameters_paren;
        let fn_path_paren;
        parenthesized!(arguments_paren in input);
        parenthesized!(parameters_paren in input);
        parenthesized!(fn_path_paren in input);

        if !input.is_empty() {
            return Err(syn::Error::new(
                Span::call_site(),
                "unexpected token in `helper!(..)` macro",
            ));
        }

        let arguments = arguments_paren.parse_terminated(Argument::parse)?;
        let parameters = parameters_paren.parse_terminated(Parameter::parse)?;
        let fn_path = fn_path_paren.parse::<MaybeStartsWithDollar<Path>>()?;

        Ok(Helper {
            arguments,
            parameters,
            fn_path,
        })
    }
}

pub(crate) fn generate(helper: Helper) -> syn::Result<TokenStream> {
    let Helper {
        arguments,
        parameters,
        fn_path,
    } = helper;

    if arguments.len() > parameters.len() {
        return Err(syn::Error::new(
            Span::call_site(),
            "The number of arguments is more than the number of parameters",
        ));
    }

    let mut fn_args = Vec::with_capacity(parameters.len());
    let mut matched_args_indexes = Vec::with_capacity(arguments.len());
    for (para_idx, para) in parameters.into_iter().enumerate() {
        let mut named = None;
        let mut positioned = None;

        for (arg_idx, arg) in arguments.iter().enumerate() {
            match arg {
                Argument::Named { pattern, value } => {
                    if *pattern == para.pattern {
                        if named.is_some() {
                            return Err(syn::Error::new(
                                Span::call_site(),
                                format_args!(
                                    "The same parameter `{}` is specified multiple times by named",
                                    para.pattern.to_token_stream()
                                ),
                            ));
                        }

                        named = Some(value);
                        matched_args_indexes.push(arg_idx);
                    }
                }
                Argument::Positioned { value } => {
                    if arg_idx == para_idx {
                        positioned = Some(value);
                        matched_args_indexes.push(arg_idx);
                    }
                }
            }
        }

        if named.is_some() && positioned.is_some() {
            return Err(syn::Error::new(
                Span::call_site(),
                format_args!(
                    "The parameter `{}` is specified both by named and positioned",
                    para.pattern.to_token_stream()
                ),
            ));
        }

        if named.is_none() && positioned.is_none() && para.default.is_none() {
            return Err(syn::Error::new(
                Span::call_site(),
                format_args!(
                    "The parameter `{}` is not specified",
                    para.pattern.to_token_stream()
                ),
            ));
        }

        let fn_arg = named
            .map(|n| MaybeStartsWithDollar::Normal(Normal { inner: n.clone() }))
            .unwrap_or_else(|| {
                positioned
                    .map(|p| MaybeStartsWithDollar::Normal(Normal { inner: p.clone() }))
                    .unwrap_or_else(|| para.default.unwrap())
            });
        fn_args.push(fn_arg);
    }

    if matched_args_indexes.len() != arguments.len() {
        let not_matched_args_indexes = (0..arguments.len())
            .filter(|i| !matched_args_indexes.contains(i))
            .collect::<Vec<_>>();

        return Err(syn::Error::new(
            Span::call_site(),
            format_args!(
                "Some arguments are not matched by any parameters and their indexes are: {:?}",
                not_matched_args_indexes
            ),
        ));
    }

    let expand = quote! {
        #fn_path(#(#fn_args,)*)
    };

    Ok(expand)
}
