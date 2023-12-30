use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    Expr, Path, Token,
};

use crate::{
    argument::Argument, maybe_start_with_dollar::MaybeStartWithDollar, parameter::Parameter,
};

pub(crate) struct NadeHelper {
    arguments: Punctuated<Argument, Token![,]>,
    parameters: Punctuated<Parameter, Token![,]>,
    fn_path: MaybeStartWithDollar<Path>,
}

impl Parse for NadeHelper {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let arguments_paren;
        let parameters_paren;
        let fn_path_paren;
        parenthesized!(arguments_paren in input);
        parenthesized!(parameters_paren in input);
        parenthesized!(fn_path_paren in input);

        let arguments = arguments_paren.parse_terminated(Argument::parse, Token![,])?;
        let parameters = parameters_paren.parse_terminated(Parameter::parse, Token![,])?;
        let fn_path = fn_path_paren.parse::<MaybeStartWithDollar<Path>>()?;

        Ok(NadeHelper {
            arguments,
            parameters,
            fn_path,
        })
    }
}

pub(crate) fn generate(nade_helper: NadeHelper) -> syn::Result<TokenStream> {
    let NadeHelper {
        arguments,
        parameters,
        fn_path,
    } = nade_helper;

    let args_len = arguments.len();
    let params_len = parameters.len();

    let mut fn_args = Vec::with_capacity(params_len);
    let mut matched_args_indexes: Vec<usize> = Vec::with_capacity(args_len);

    for (param_idx, param) in parameters.iter().enumerate() {
        let arg = get_single_argument(&mut matched_args_indexes, param_idx, param, &arguments)?;
        fn_args.push(arg);
    }

    if let Some(e) = arguments
        .iter()
        .enumerate()
        .filter_map(|(idx, arg)| {
            if matched_args_indexes.contains(&idx) {
                None
            } else {
                Some(arg)
            }
        })
        .map(|arg| {
            syn::Error::new(
                arg.span(),
                format!(
                    "argument `{}` is not matched by any parameters",
                    arg.to_token_stream()
                ),
            )
        })
        .reduce(|mut a, b| {
            a.combine(b);
            a
        })
    {
        return Err(e);
    }

    let expand = quote! {
        #fn_path(#(#fn_args,)*)
    };

    Ok(expand)
}

fn get_single_argument<'a>(
    matched_args_indexes: &mut Vec<usize>,
    parameter_index: usize,
    parameter: &'a Parameter,
    arguments: &'a Punctuated<Argument, Token![,]>,
) -> syn::Result<MaybeStartWithDollar<&'a Expr>> {
    let mut named: Option<(Span, &Expr)> = None;
    let mut positioned: Option<(Span, &Expr)> = None;

    for (arg_idx, arg) in arguments.iter().enumerate() {
        let span = arg.span();

        match arg {
            Argument::Named { pattern, value, .. } => {
                if *pattern == parameter.pat {
                    if named.is_some() {
                        return Err(syn::Error::new(
                            span,
                            format!(
                                "parameter `{}` is specified multiple times by named",
                                parameter.to_token_stream()
                            ),
                        ));
                    }

                    named = Some((span, value));
                    matched_args_indexes.push(arg_idx);
                }
            }
            Argument::Positioned { value } => {
                if arg_idx == parameter_index {
                    positioned = Some((span, value));
                    matched_args_indexes.push(arg_idx);
                }
            }
        }
    }

    if let (Some((named, _)), Some((positioned, _))) = (named, positioned) {
        macro_rules! err {
            ($span:expr) => {
                syn::Error::new(
                    $span,
                    format!(
                        "parameter `{}` is specified both by named and positioned",
                        parameter.to_token_stream()
                    ),
                )
            };
        }

        let mut e = err!(named);
        e.combine(err!(positioned));

        return Err(e);
    }

    if named.is_none() && positioned.is_none() && parameter.default.is_none() {
        return Err(syn::Error::new(
            Span::call_site(),
            format!(
                "parameter `{}` is not specified",
                parameter.to_token_stream()
            ),
        ));
    }

    let fn_arg = named
        .map(|(_, n)| MaybeStartWithDollar::Normal(n))
        .unwrap_or_else(|| {
            positioned
                .map(|(_, p)| MaybeStartWithDollar::Normal(p))
                .unwrap_or_else(|| parameter.default.as_ref().unwrap().1.as_ref())
        });

    Ok(fn_arg)
}
