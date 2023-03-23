use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    Expr, Pat, Token,
};

use crate::maybe_starts_with_dollar::MaybeStartsWithDollar;

pub(crate) struct Parameter {
    pub(crate) pattern: Pat,
    pub(crate) default: Option<MaybeStartsWithDollar<Expr>>,
}

impl ToTokens for Parameter {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.pattern.to_tokens(tokens);
        if let Some(expr) = &self.default {
            <Token![=]>::default().to_tokens(tokens);
            expr.to_tokens(tokens);
        }
    }
}

impl Parse for Parameter {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let test = input.fork();
        let parameter = if test.call(Pat::parse_single).is_ok() && test.peek(Token![=]) {
            let pattern = input.call(Pat::parse_single)?;
            input.parse::<Token![=]>()?;
            let default = input.parse::<MaybeStartsWithDollar<Expr>>()?;
            Parameter {
                pattern,
                default: Some(default),
            }
        } else {
            let pattern = input.call(Pat::parse_single)?;
            Parameter {
                pattern,
                default: None,
            }
        };

        Ok(parameter)
    }
}
