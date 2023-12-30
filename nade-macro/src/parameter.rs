use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    Expr, Pat, Token, Type,
};

use crate::maybe_start_with_dollar::MaybeStartWithDollar;

pub(crate) struct Parameter {
    pub(crate) pat: Pat,
    colon_token: Token![:],
    ty: Type,
    pub(crate) default: Option<(Token![=], MaybeStartWithDollar<Expr>)>,
}

impl Parameter {
    pub(crate) fn new(
        pat: Pat,
        colon_token: Token![:],
        ty: Type,
        default: Option<MaybeStartWithDollar<Expr>>,
    ) -> Self {
        Self {
            pat,
            colon_token,
            ty,
            default: default.map(|expr| (<Token![=]>::default(), expr)),
        }
    }
}

impl ToTokens for Parameter {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.pat.to_tokens(tokens);
        self.colon_token.to_tokens(tokens);
        self.ty.to_tokens(tokens);
        if let Some((eq_token, expr)) = &self.default {
            eq_token.to_tokens(tokens);
            expr.to_tokens(tokens);
        }
    }
}

impl Parse for Parameter {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let pat = input.call(Pat::parse_single)?;
        let colon_token = input.parse::<Token![:]>()?;
        let ty = input.parse::<Type>()?;

        let default = if input.peek(Token![=]) {
            Some((input.parse()?, input.parse()?))
        } else {
            None
        };

        Ok(Parameter {
            pat,
            colon_token,
            ty,
            default,
        })
    }
}
