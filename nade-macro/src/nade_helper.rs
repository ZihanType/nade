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
    argument::Argument, maybe_starts_with_dollar::MaybeStartsWithDollar, parameter::Parameter,
};

pub(crate) struct NadeHelper {
    arguments: Punctuated<Argument, Token![,]>,
    parameters: Punctuated<Parameter, Token![,]>,
    fn_path: MaybeStartsWithDollar<Path>,
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
        let fn_path = fn_path_paren.parse::<MaybeStartsWithDollar<Path>>()?;

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
    let paras_len = parameters.len();

    let mut fn_args = Vec::with_capacity(paras_len);
    let mut matched_args_indexes: Vec<usize> = Vec::with_capacity(args_len);

    for (para_idx, para) in parameters.into_iter().enumerate() {
        fn_args.push(match_one_parameter(
            &mut matched_args_indexes,
            para_idx,
            para,
            &arguments,
        )?);
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
                format_args!(
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

fn match_one_parameter(
    matched_args_indexes: &mut Vec<usize>,
    parameter_index: usize,
    parameter: Parameter,
    arguments: &Punctuated<Argument, Token![,]>,
) -> syn::Result<MaybeStartsWithDollar<Expr>> {
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
                            format_args!(
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
        let msg = format!(
            "parameter `{}` is specified both by named and positioned",
            parameter.to_token_stream()
        );

        macro_rules! err {
            ($span:expr) => {
                syn::Error::new($span, &msg)
            };
        }

        let mut e = err!(named);
        e.combine(err!(positioned));

        return Err(e);
    }

    if named.is_none() && positioned.is_none() && parameter.default.is_none() {
        return Err(syn::Error::new(
            Span::call_site(),
            format_args!(
                "parameter `{}` is not specified",
                parameter.to_token_stream()
            ),
        ));
    }

    let fn_arg = named
        .map(|(_, n)| MaybeStartsWithDollar::Normal(n.clone()))
        .unwrap_or_else(|| {
            positioned
                .map(|(_, p)| MaybeStartsWithDollar::Normal(p.clone()))
                .unwrap_or_else(|| parameter.default.unwrap().1)
        });

    Ok(fn_arg)
}
