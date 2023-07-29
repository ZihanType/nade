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
        let pattern = input.call(Pat::parse_single)?;

        let default = match input.parse::<Option<Token![=]>>()? {
            Some(_) => Some(input.parse::<MaybeStartsWithDollar<Expr>>()?),
            None => None,
        };

        Ok(Parameter { pattern, default })
    }
}
