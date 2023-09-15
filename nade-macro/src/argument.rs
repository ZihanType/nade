use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    Expr, Pat, Token,
};

pub(crate) enum Argument {
    Positioned {
        value: Expr,
    },
    Named {
        pattern: Pat,
        eq_token: Token![=],
        value: Expr,
    },
}

impl Parse for Argument {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let is_named = {
            let test = input.fork();
            test.call(Pat::parse_single).is_ok() && test.peek(Token![=])
        };

        let argument = if is_named {
            Argument::Named {
                pattern: input.call(Pat::parse_single)?,
                eq_token: input.parse::<Token![=]>()?,
                value: input.parse::<Expr>()?,
            }
        } else {
            Argument::Positioned {
                value: input.parse::<Expr>()?,
            }
        };

        Ok(argument)
    }
}

impl ToTokens for Argument {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Argument::Positioned { value } => value.to_tokens(tokens),
            Argument::Named {
                pattern,
                eq_token,
                value,
            } => {
                pattern.to_tokens(tokens);
                eq_token.to_tokens(tokens);
                value.to_tokens(tokens);
            }
        }
    }
}
