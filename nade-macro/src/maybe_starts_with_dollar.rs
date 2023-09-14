use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    Token,
};

pub(crate) enum MaybeStartsWithDollar<T> {
    StartsWithDollar(StartsWithDollar<T>),
    Normal(T),
}

impl<T: Parse> Parse for MaybeStartsWithDollar<T> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(if input.peek(Token![$]) {
            Self::StartsWithDollar(input.parse()?)
        } else {
            Self::Normal(input.parse()?)
        })
    }
}

impl<T: ToTokens> ToTokens for MaybeStartsWithDollar<T> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            MaybeStartsWithDollar::StartsWithDollar(e) => e.to_tokens(tokens),
            MaybeStartsWithDollar::Normal(e) => e.to_tokens(tokens),
        }
    }
}

impl<T> MaybeStartsWithDollar<T> {
    pub(crate) fn inner(&self) -> &T {
        match self {
            MaybeStartsWithDollar::StartsWithDollar(s) => &s.inner,
            MaybeStartsWithDollar::Normal(n) => n,
        }
    }
}

pub(crate) struct StartsWithDollar<T> {
    dollar_token: Token![$],
    pub(crate) inner: T,
}

impl<T: Parse> Parse for StartsWithDollar<T> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let dollar_token = input.parse::<Token![$]>()?;

        let lookahead = input.lookahead1();

        if lookahead.peek(Token![crate]) {
            Ok(Self {
                dollar_token,
                inner: input.parse()?,
            })
        } else {
            Err(lookahead.error())
        }
    }
}

impl<T: ToTokens> ToTokens for StartsWithDollar<T> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.dollar_token.to_tokens(tokens);
        self.inner.to_tokens(tokens);
    }
}
